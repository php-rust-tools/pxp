use crate::internal::attributes;
use crate::internal::precedences::Associativity;
use crate::internal::precedences::Precedence;
use crate::Parser;
use crate::ParserDiagnostic;
use pxp_ast::Expression;
use pxp_ast::*;
use pxp_ast::{
    ArrayIndexExpression, CoalesceExpression, ConcatExpression, ConstantFetchExpression,
    ExpressionKind, FunctionCallExpression, FunctionClosureCreationExpression,
    InstanceofExpression, MagicConstantExpression, MethodCallExpression,
    MethodClosureCreationExpression, NullsafeMethodCallExpression, NullsafePropertyFetchExpression,
    PropertyFetchExpression, ReferenceExpression, ShortTernaryExpression,
    StaticMethodCallExpression, StaticMethodClosureCreationExpression,
    StaticPropertyFetchExpression, StaticVariableMethodCallExpression,
    StaticVariableMethodClosureCreationExpression, TernaryExpression,
};

use pxp_diagnostics::Severity;
use pxp_span::Span;
use pxp_span::Spanned;
use pxp_token::TokenKind;

use pxp_ast::BoolExpression;
use pxp_ast::CastExpression;
use pxp_ast::CloneExpression;
use pxp_ast::DieExpression;
use pxp_ast::EmptyExpression;
use pxp_ast::ErrorSuppressExpression;
use pxp_ast::EvalExpression;
use pxp_ast::ExitExpression;
use pxp_ast::IncludeExpression;
use pxp_ast::IncludeOnceExpression;
use pxp_ast::IssetExpression;
use pxp_ast::NewExpression;
use pxp_ast::ParenthesizedExpression;
use pxp_ast::PrintExpression;
use pxp_ast::RequireExpression;
use pxp_ast::RequireOnceExpression;
use pxp_ast::ThrowExpression;
use pxp_ast::UnsetExpression;
use pxp_ast::YieldExpression;
use pxp_ast::YieldFromExpression;

impl<'a> Parser<'a> {
    pub fn create(&mut self) -> Expression {
        self.for_precedence(Precedence::Lowest)
    }

    fn null_coalesce_precedence(&mut self) -> Expression {
        self.for_precedence(Precedence::NullCoalesce)
    }

    fn clone_or_new_precedence(&mut self) -> Expression {
        self.for_precedence(Precedence::CloneOrNew)
    }

    fn for_precedence(&mut self, precedence: Precedence) -> Expression {
        let mut left = self.left(&precedence);

        loop {
            let current = self.state.current();
            let span = current.span;
            let kind = &current.kind;

            if matches!(current.kind, TokenKind::SemiColon | TokenKind::Eof) {
                break;
            }

            if self.is_postfix(kind) {
                let lpred = Precedence::postfix(kind);

                if lpred < precedence {
                    break;
                }

                left = self.postfix(left, kind);
                continue;
            }

            if self.is_infix(kind) {
                let rpred = Precedence::infix(kind);

                if rpred < precedence {
                    break;
                }

                if rpred == precedence && matches!(rpred.associativity(), Some(Associativity::Left))
                {
                    break;
                }

                if rpred == precedence && matches!(rpred.associativity(), Some(Associativity::Non))
                {
                    self.state.diagnostic(
                        ParserDiagnostic::UnexpectedToken {
                            token: current.to_owned(),
                        },
                        Severity::Error,
                        current.span,
                    );
                }

                self.state.next();

                let op = self.state.current();
                let start_span = op.span;
                let kind = match kind {
                    TokenKind::Question => {
                        // this happens due to a comment, or whitespaces between the  and the :
                        // we consider `foo()  : bar()` a ternary expression, with `then` being a noop
                        // however, this must behave like a short ternary at runtime.
                        if op.kind == TokenKind::Colon {
                            self.state.next();

                            let r#else = self.create();

                            ExpressionKind::Ternary(TernaryExpression {
                                id: self.state.id(),
                                span: Span::combine(left.span, r#else.span),
                                condition: Box::new(left),
                                question: span,
                                then: Box::new(Expression::noop(self.state.id(), start_span)),
                                colon: op.span,
                                r#else: Box::new(r#else),
                            })
                        } else {
                            let then = self.create();
                            let colon = self.skip_colon();
                            let r#else = self.create();

                            ExpressionKind::Ternary(TernaryExpression {
                                id: self.state.id(),
                                span: Span::combine(left.span, r#else.span),
                                condition: Box::new(left),
                                question: span,
                                then: Box::new(then),
                                colon,
                                r#else: Box::new(r#else),
                            })
                        }
                    }
                    TokenKind::QuestionColon => {
                        let r#else = self.create();
                        ExpressionKind::ShortTernary(ShortTernaryExpression {
                            id: self.state.id(),
                            span: Span::combine(left.span, r#else.span),
                            condition: Box::new(left),
                            question_colon: span,
                            r#else: Box::new(r#else),
                        })
                    }
                    TokenKind::Equals if op.kind == TokenKind::Ampersand => {
                        self.state.next();

                        // FIXME: You should only be allowed to assign a referencable variable,
                        //        here, not any old expression.
                        let right = Box::new(self.for_precedence(rpred));
                        let right_span = right.span;
                        let span = Span::combine(left.span, right_span);
                        let reference_span = Span::combine(op.span, right_span);

                        ExpressionKind::AssignmentOperation(AssignmentOperationExpression {
                            id: self.state.id(),
                            span,
                            kind: AssignmentOperationKind::Assign {
                                id: self.state.id(),
                                left: Box::new(left),
                                equals: span,
                                right: Box::new(Expression::new(
                                    self.state.id(),
                                    ExpressionKind::Reference(ReferenceExpression {
                                        id: self.state.id(),
                                        span: reference_span,
                                        ampersand: op.span,
                                        right,
                                    }),
                                    Span::new(start_span.start, right_span.end),
                                    CommentGroup::default(),
                                )),
                            },
                        })
                    }
                    TokenKind::Instanceof if op.kind == TokenKind::Self_ => {
                        let self_span = op.span;
                        self.state.next();
                        let right = Expression::new(
                            self.state.id(),
                            ExpressionKind::Self_(SelfExpression {
                                id: self.state.id(),
                                span: self_span,
                            }),
                            self_span,
                            CommentGroup::default(),
                        );
                        let span = Span::combine(left.span, right.span);

                        ExpressionKind::Instanceof(InstanceofExpression {
                            id: self.state.id(),
                            span,
                            left: Box::new(left),
                            instanceof: span,
                            right: Box::new(right),
                        })
                    }
                    TokenKind::Instanceof if op.kind == TokenKind::Parent => {
                        self.state.next();
                        let right = Expression::new(
                            self.state.id(),
                            ExpressionKind::Parent(ParentExpression {
                                id: self.state.id(),
                                span: op.span,
                            }),
                            op.span,
                            CommentGroup::default(),
                        );
                        let span = Span::combine(left.span, right.span);

                        ExpressionKind::Instanceof(InstanceofExpression {
                            id: self.state.id(),
                            span,
                            left: Box::new(left),
                            instanceof: span,
                            right: Box::new(right),
                        })
                    }
                    TokenKind::Instanceof if op.kind == TokenKind::Static => {
                        let instanceof = span;
                        self.state.next();
                        let right = Expression::new(
                            self.state.id(),
                            ExpressionKind::Static(StaticExpression {
                                id: self.state.id(),
                                span: op.span,
                            }),
                            op.span,
                            CommentGroup::default(),
                        );

                        ExpressionKind::Instanceof(InstanceofExpression {
                            id: self.state.id(),
                            span: Span::combine(left.span, right.span),
                            left: Box::new(left),
                            instanceof,
                            right: Box::new(right),
                        })
                    }
                    TokenKind::Instanceof if op.kind == TokenKind::Enum => {
                        let enum_span = op.span;
                        self.state.next();

                        let right = Expression::new(
                            self.state.id(),
                            ExpressionKind::Identifier(Identifier::SimpleIdentifier(
                                SimpleIdentifier::new(
                                    self.state.id(),
                                    op.symbol.to_bytestring(),
                                    enum_span,
                                ),
                            )),
                            enum_span,
                            CommentGroup::default(),
                        );

                        ExpressionKind::Instanceof(InstanceofExpression {
                            id: self.state.id(),
                            span: Span::combine(left.span, right.span),
                            left: Box::new(left),
                            instanceof: span,
                            right: Box::new(right),
                        })
                    }
                    TokenKind::Instanceof if op.kind == TokenKind::From => {
                        let from_span = op.span;
                        self.state.next();
                        let right = Expression::new(
                            self.state.id(),
                            ExpressionKind::Identifier(Identifier::SimpleIdentifier(
                                SimpleIdentifier::new(
                                    self.state.id(),
                                    op.symbol.to_bytestring(),
                                    op.span,
                                ),
                            )),
                            Span::new(start_span.start, from_span.end),
                            CommentGroup::default(),
                        );

                        ExpressionKind::Instanceof(InstanceofExpression {
                            id: self.state.id(),
                            span: Span::combine(left.span, right.span),
                            left: Box::new(left),
                            instanceof: span,
                            right: Box::new(right),
                        })
                    }
                    _ => {
                        let op_span = span;
                        let left = Box::new(left);
                        let right = Box::new(self.for_precedence(rpred));
                        let span = Span::combine(left.span, right.span);

                        match kind {
                            TokenKind::Plus => {
                                ExpressionKind::ArithmeticOperation(ArithmeticOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: ArithmeticOperationKind::Addition {
                                        id: self.state.id(),
                                        left,
                                        plus: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::Minus => {
                                ExpressionKind::ArithmeticOperation(ArithmeticOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: ArithmeticOperationKind::Subtraction {
                                        id: self.state.id(),
                                        left,
                                        minus: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::Asterisk => {
                                ExpressionKind::ArithmeticOperation(ArithmeticOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: ArithmeticOperationKind::Multiplication {
                                        id: self.state.id(),
                                        left,
                                        asterisk: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::Slash => {
                                ExpressionKind::ArithmeticOperation(ArithmeticOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: ArithmeticOperationKind::Division {
                                        id: self.state.id(),
                                        left,
                                        slash: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::Percent => {
                                ExpressionKind::ArithmeticOperation(ArithmeticOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: ArithmeticOperationKind::Modulo {
                                        id: self.state.id(),
                                        left,
                                        percent: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::Pow => {
                                ExpressionKind::ArithmeticOperation(ArithmeticOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: ArithmeticOperationKind::Exponentiation {
                                        id: self.state.id(),
                                        left,
                                        pow: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::Equals => {
                                ExpressionKind::AssignmentOperation(AssignmentOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: AssignmentOperationKind::Assign {
                                        id: self.state.id(),
                                        left,
                                        equals: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::PlusEquals => {
                                ExpressionKind::AssignmentOperation(AssignmentOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: AssignmentOperationKind::Addition {
                                        id: self.state.id(),
                                        left,
                                        plus_equals: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::MinusEquals => {
                                ExpressionKind::AssignmentOperation(AssignmentOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: AssignmentOperationKind::Subtraction {
                                        id: self.state.id(),
                                        left,
                                        minus_equals: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::AsteriskEquals => {
                                ExpressionKind::AssignmentOperation(AssignmentOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: AssignmentOperationKind::Multiplication {
                                        id: self.state.id(),
                                        left,
                                        asterisk_equals: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::SlashEquals => {
                                ExpressionKind::AssignmentOperation(AssignmentOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: AssignmentOperationKind::Division {
                                        id: self.state.id(),
                                        left,
                                        slash_equals: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::PercentEquals => {
                                ExpressionKind::AssignmentOperation(AssignmentOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: AssignmentOperationKind::Modulo {
                                        id: self.state.id(),
                                        left,
                                        percent_equals: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::PowEquals => {
                                ExpressionKind::AssignmentOperation(AssignmentOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: AssignmentOperationKind::Exponentiation {
                                        id: self.state.id(),
                                        left,
                                        pow_equals: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::AmpersandEquals => {
                                ExpressionKind::AssignmentOperation(AssignmentOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: AssignmentOperationKind::BitwiseAnd {
                                        id: self.state.id(),
                                        left,
                                        ampersand_equals: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::PipeEquals => {
                                ExpressionKind::AssignmentOperation(AssignmentOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: AssignmentOperationKind::BitwiseOr {
                                        id: self.state.id(),
                                        left,
                                        pipe_equals: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::CaretEquals => {
                                ExpressionKind::AssignmentOperation(AssignmentOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: AssignmentOperationKind::BitwiseXor {
                                        id: self.state.id(),
                                        left,
                                        caret_equals: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::LeftShiftEquals => {
                                ExpressionKind::AssignmentOperation(AssignmentOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: AssignmentOperationKind::LeftShift {
                                        id: self.state.id(),
                                        left,
                                        left_shift_equals: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::RightShiftEquals => {
                                ExpressionKind::AssignmentOperation(AssignmentOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: AssignmentOperationKind::RightShift {
                                        id: self.state.id(),
                                        left,
                                        right_shift_equals: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::DoubleQuestionEquals => {
                                ExpressionKind::AssignmentOperation(AssignmentOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: AssignmentOperationKind::Coalesce {
                                        id: self.state.id(),
                                        left,
                                        coalesce_equals: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::DotEquals => {
                                ExpressionKind::AssignmentOperation(AssignmentOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: AssignmentOperationKind::Concat {
                                        id: self.state.id(),
                                        left,
                                        dot_equals: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::Ampersand => {
                                ExpressionKind::BitwiseOperation(BitwiseOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: BitwiseOperationKind::And {
                                        id: self.state.id(),
                                        left,
                                        and: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::Pipe => {
                                ExpressionKind::BitwiseOperation(BitwiseOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: BitwiseOperationKind::Or {
                                        id: self.state.id(),
                                        left,
                                        or: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::Caret => {
                                ExpressionKind::BitwiseOperation(BitwiseOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: BitwiseOperationKind::Xor {
                                        id: self.state.id(),
                                        left,
                                        xor: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::LeftShift => {
                                ExpressionKind::BitwiseOperation(BitwiseOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: BitwiseOperationKind::LeftShift {
                                        id: self.state.id(),
                                        left,
                                        left_shift: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::RightShift => {
                                ExpressionKind::BitwiseOperation(BitwiseOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: BitwiseOperationKind::RightShift {
                                        id: self.state.id(),
                                        left,
                                        right_shift: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::DoubleEquals => {
                                ExpressionKind::ComparisonOperation(ComparisonOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: ComparisonOperationKind::Equal {
                                        id: self.state.id(),
                                        left,
                                        double_equals: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::TripleEquals => {
                                ExpressionKind::ComparisonOperation(ComparisonOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: ComparisonOperationKind::Identical {
                                        id: self.state.id(),
                                        left,
                                        triple_equals: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::BangEquals => {
                                ExpressionKind::ComparisonOperation(ComparisonOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: ComparisonOperationKind::NotEqual {
                                        id: self.state.id(),
                                        left,
                                        bang_equals: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::AngledLeftRight => {
                                ExpressionKind::ComparisonOperation(ComparisonOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: ComparisonOperationKind::AngledNotEqual {
                                        id: self.state.id(),
                                        left,
                                        angled_left_right: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::BangDoubleEquals => {
                                ExpressionKind::ComparisonOperation(ComparisonOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: ComparisonOperationKind::NotIdentical {
                                        id: self.state.id(),
                                        left,
                                        bang_double_equals: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::LessThan => {
                                ExpressionKind::ComparisonOperation(ComparisonOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: ComparisonOperationKind::LessThan {
                                        id: self.state.id(),
                                        left,
                                        less_than: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::GreaterThan => {
                                ExpressionKind::ComparisonOperation(ComparisonOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: ComparisonOperationKind::GreaterThan {
                                        id: self.state.id(),
                                        left,
                                        greater_than: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::LessThanEquals => {
                                ExpressionKind::ComparisonOperation(ComparisonOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: ComparisonOperationKind::LessThanOrEqual {
                                        id: self.state.id(),
                                        left,
                                        less_than_equals: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::GreaterThanEquals => {
                                ExpressionKind::ComparisonOperation(ComparisonOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: ComparisonOperationKind::GreaterThanOrEqual {
                                        id: self.state.id(),
                                        left,
                                        greater_than_equals: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::Spaceship => {
                                ExpressionKind::ComparisonOperation(ComparisonOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: ComparisonOperationKind::Spaceship {
                                        id: self.state.id(),
                                        left,
                                        spaceship: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::BooleanAnd => {
                                ExpressionKind::LogicalOperation(LogicalOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: LogicalOperationKind::And {
                                        id: self.state.id(),
                                        left,
                                        double_ampersand: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::BooleanOr => {
                                ExpressionKind::LogicalOperation(LogicalOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: LogicalOperationKind::Or {
                                        id: self.state.id(),
                                        left,
                                        double_pipe: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::LogicalAnd => {
                                ExpressionKind::LogicalOperation(LogicalOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: LogicalOperationKind::LogicalAnd {
                                        id: self.state.id(),
                                        left,
                                        and: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::LogicalOr => {
                                ExpressionKind::LogicalOperation(LogicalOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: LogicalOperationKind::LogicalOr {
                                        id: self.state.id(),
                                        left,
                                        or: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::LogicalXor => {
                                ExpressionKind::LogicalOperation(LogicalOperationExpression {
                                    id: self.state.id(),
                                    span,
                                    kind: LogicalOperationKind::LogicalXor {
                                        id: self.state.id(),
                                        left,
                                        xor: op_span,
                                        right,
                                    },
                                })
                            }
                            TokenKind::Dot => ExpressionKind::Concat(ConcatExpression {
                                id: self.state.id(),
                                span,
                                left,
                                dot: op_span,
                                right,
                            }),
                            TokenKind::Instanceof => {
                                ExpressionKind::Instanceof(InstanceofExpression {
                                    id: self.state.id(),
                                    span,
                                    left,
                                    instanceof: op_span,
                                    right,
                                })
                            }
                            _ => unreachable!(),
                        }
                    }
                };

                let end_span = self.state.previous().span;

                left = Expression::new(
                    self.state.id(),
                    kind,
                    Span::new(start_span.start, end_span.end),
                    CommentGroup::default(),
                );

                self.maybe_shift_assignment_operands(&mut left);

                continue;
            }

            break;
        }

        left
    }

    fn should_shift_assignment_operands(&self, expr: &Expression) -> bool {
        let is_assignment = matches!(expr.kind, ExpressionKind::AssignmentOperation(_));

        if !is_assignment {
            return false;
        }

        let ExpressionKind::AssignmentOperation(AssignmentOperationExpression { kind, .. }) =
            &expr.kind
        else {
            unreachable!()
        };

        matches!(
            kind.left().kind,
            ExpressionKind::ComparisonOperation(_)
                | ExpressionKind::BitwiseOperation(_)
                | ExpressionKind::ArithmeticOperation(_)
                | ExpressionKind::LogicalOperation(_)
        )
    }

    // This is a workaround for a problem somebody reported, but something that
    // I've found in other PHP parsers too.
    //
    // Given the following code:
    //     true !== $a = true
    // The precedence system interprets this as:
    //     (true !== $a) = true
    // which isn't a valid assignment target.
    //
    // This seems to be a downfall of the precedence system and a side effect
    // of PHP treating assignment as an expression rather than a statement.
    //
    // I found a similar piece of logic in the `microsoft/tolerant-php-parser` project,
    // where they check to see if the expression being created is an assignment operation
    // and if the left-hand side if a binary operation.
    //
    // If it is, then they shift the operands around to fake parentheses, so that the expression
    // is instead interpreted as:
    //     true !== ($a = true)
    //
    // This is a real mega hack, but it seems to work and should be the only place where
    // we need to do this sort of trickery-bobbery.
    fn maybe_shift_assignment_operands(&self, expr: &mut Expression) {
        if !self.should_shift_assignment_operands(expr) {
            return;
        }

        // At this point, we know that the left-hand side of the expression is an assignment.
        let ExpressionKind::AssignmentOperation(AssignmentOperationExpression { id, kind, .. }) =
            &expr.kind
        else {
            unreachable!()
        };

        // Given the following AST:
        // AssignmentOperation {
        //     left: ComparisonOperation {
        //         left: true,
        //         op: !==,
        //         right: $a
        //     },
        //     right: true
        // }
        //
        // We need to transform it into:
        // ComparisonOperation {
        //     left: true,
        //     op: !==,
        //     right: AssignmentOperation {
        //         left: $a,
        //         right: true
        //     }
        // }

        // So we first need to get the left-hand side of the assignment.
        // Which in the example above will be the ComparisonOperation.
        let assignment_left = kind.left();

        // We also need the right-hand side of the assignment since
        // that will be our new right-hand side too.
        let assignment_right = kind.right();

        // Then we need to get the right-hand side of the comparison, since
        // this is the real assignment target.
        let real_assignment_target = match &assignment_left.kind {
            ExpressionKind::ComparisonOperation(ComparisonOperationExpression { kind, .. }) => {
                Some(kind.right())
            }
            ExpressionKind::BitwiseOperation(BitwiseOperationExpression { kind, .. }) => {
                Some(kind.right())
            }
            ExpressionKind::ArithmeticOperation(ArithmeticOperationExpression { kind, .. }) => {
                kind.right()
            }
            ExpressionKind::LogicalOperation(LogicalOperationExpression { kind, .. }) => {
                Some(kind.right())
            }
            _ => todo!(),
        };

        if real_assignment_target.is_none() {
            // This is a bit of a hack, but we can't really do anything about it.
            // If we can't find the real assignment target, then we can't shift the operands.
            return;
        }

        // Then we can create the new right-hand side of the comparison, which will
        // be an assignment expression.
        //
        // This is a bit lengthy since we need to match against the existing assignment to
        // make sure it's the right type of assignment.
        let new_right = Expression::new(
            expr.id,
            ExpressionKind::AssignmentOperation(AssignmentOperationExpression {
                id: *id,
                span: Span::default(),
                kind: match kind {
                    AssignmentOperationKind::Assign { id, equals, .. } => {
                        AssignmentOperationKind::Assign {
                            id: *id,
                            left: Box::new(real_assignment_target.cloned().unwrap()),
                            equals: *equals,
                            right: Box::new(assignment_right.clone()),
                        }
                    }
                    AssignmentOperationKind::Addition {
                        id, plus_equals, ..
                    } => AssignmentOperationKind::Addition {
                        id: *id,
                        left: Box::new(real_assignment_target.cloned().unwrap()),
                        plus_equals: *plus_equals,
                        right: Box::new(assignment_right.clone()),
                    },
                    AssignmentOperationKind::Subtraction {
                        id, minus_equals, ..
                    } => AssignmentOperationKind::Subtraction {
                        id: *id,
                        left: Box::new(real_assignment_target.cloned().unwrap()),
                        minus_equals: *minus_equals,
                        right: Box::new(assignment_right.clone()),
                    },
                    AssignmentOperationKind::Multiplication {
                        id,
                        asterisk_equals,
                        ..
                    } => AssignmentOperationKind::Multiplication {
                        id: *id,
                        left: Box::new(real_assignment_target.cloned().unwrap()),
                        asterisk_equals: *asterisk_equals,
                        right: Box::new(assignment_right.clone()),
                    },
                    AssignmentOperationKind::Division {
                        id, slash_equals, ..
                    } => AssignmentOperationKind::Division {
                        id: *id,
                        left: Box::new(real_assignment_target.cloned().unwrap()),
                        slash_equals: *slash_equals,
                        right: Box::new(assignment_right.clone()),
                    },
                    AssignmentOperationKind::Modulo {
                        id, percent_equals, ..
                    } => AssignmentOperationKind::Modulo {
                        id: *id,
                        left: Box::new(real_assignment_target.cloned().unwrap()),
                        percent_equals: *percent_equals,
                        right: Box::new(assignment_right.clone()),
                    },
                    AssignmentOperationKind::Exponentiation { id, pow_equals, .. } => {
                        AssignmentOperationKind::Exponentiation {
                            id: *id,
                            left: Box::new(real_assignment_target.cloned().unwrap()),
                            pow_equals: *pow_equals,
                            right: Box::new(assignment_right.clone()),
                        }
                    }
                    AssignmentOperationKind::Concat { id, dot_equals, .. } => {
                        AssignmentOperationKind::Concat {
                            id: *id,
                            left: Box::new(real_assignment_target.cloned().unwrap()),
                            dot_equals: *dot_equals,
                            right: Box::new(assignment_right.clone()),
                        }
                    }
                    AssignmentOperationKind::BitwiseAnd {
                        id,
                        ampersand_equals,
                        ..
                    } => AssignmentOperationKind::BitwiseAnd {
                        id: *id,
                        left: Box::new(real_assignment_target.cloned().unwrap()),
                        ampersand_equals: *ampersand_equals,
                        right: Box::new(assignment_right.clone()),
                    },
                    AssignmentOperationKind::BitwiseOr {
                        id, pipe_equals, ..
                    } => AssignmentOperationKind::BitwiseOr {
                        id: *id,
                        left: Box::new(real_assignment_target.cloned().unwrap()),
                        pipe_equals: *pipe_equals,
                        right: Box::new(assignment_right.clone()),
                    },
                    AssignmentOperationKind::BitwiseXor {
                        id, caret_equals, ..
                    } => AssignmentOperationKind::BitwiseXor {
                        id: *id,
                        left: Box::new(real_assignment_target.cloned().unwrap()),
                        caret_equals: *caret_equals,
                        right: Box::new(assignment_right.clone()),
                    },
                    AssignmentOperationKind::LeftShift {
                        id,
                        left_shift_equals,
                        ..
                    } => AssignmentOperationKind::LeftShift {
                        id: *id,
                        left: Box::new(real_assignment_target.cloned().unwrap()),
                        left_shift_equals: *left_shift_equals,
                        right: Box::new(assignment_right.clone()),
                    },
                    AssignmentOperationKind::RightShift {
                        id,
                        right_shift_equals,
                        ..
                    } => AssignmentOperationKind::RightShift {
                        id: *id,
                        left: Box::new(real_assignment_target.cloned().unwrap()),
                        right_shift_equals: *right_shift_equals,
                        right: Box::new(assignment_right.clone()),
                    },
                    AssignmentOperationKind::Coalesce {
                        id,
                        coalesce_equals,
                        ..
                    } => AssignmentOperationKind::Coalesce {
                        id: *id,
                        left: Box::new(real_assignment_target.cloned().unwrap()),
                        coalesce_equals: *coalesce_equals,
                        right: Box::new(assignment_right.clone()),
                    },
                },
            }),
            Span::default(),
            CommentGroup::default(),
        );

        // Then we need to create the new binary operation, which will replace
        // the existing assignment operation.
        let mut new_expression = assignment_left.clone();

        match &mut new_expression.kind {
            ExpressionKind::ComparisonOperation(ComparisonOperationExpression { kind, .. }) => {
                kind.set_right(Box::new(new_right))
            }
            ExpressionKind::BitwiseOperation(BitwiseOperationExpression { kind, .. }) => {
                kind.set_right(Box::new(new_right))
            }
            ExpressionKind::ArithmeticOperation(ArithmeticOperationExpression { kind, .. }) => {
                kind.set_right(Box::new(new_right))
            }
            ExpressionKind::LogicalOperation(LogicalOperationExpression { kind, .. }) => {
                kind.set_right(Box::new(new_right))
            }
            _ => unreachable!(),
        };

        *expr = new_expression;
    }

    pub fn attributes(&mut self) -> Expression {
        self.gather_attributes();

        let current = self.state.current();

        match &current.kind {
            TokenKind::Static if self.state.peek().kind == TokenKind::Function => {
                self.anonymous_function()
            }
            TokenKind::Static if self.state.peek().kind == TokenKind::Fn => {
                self.arrow_function()
            }
            TokenKind::Function => self.anonymous_function(),
            TokenKind::Fn => self.arrow_function(),
            _ => {
                self.state.diagnostic(
                    ParserDiagnostic::InvalidTargetForAttributes,
                    Severity::Error,
                    current.span,
                );

                Expression::missing(self.state.id(), current.span)
            }
        }
    }

    fn left(&mut self, precedence: &Precedence) -> Expression {
        if self.state.is_eof() {
            self.state.diagnostic(
                ParserDiagnostic::UnexpectedEndOfFile,
                Severity::Error,
                self.state.current().span,
            );

            return Expression::missing(self.state.id(), self.state.current().span);
        }

        let current = self.state.current();
        let peek = self.state.peek();

        match (&current.kind, &peek.kind) {
            (TokenKind::Attribute, _) => self.attributes(),

            (TokenKind::Static, TokenKind::Fn) => self.arrow_function(),

            (TokenKind::Static, TokenKind::Function) => self.anonymous_function(),

            (TokenKind::Fn, _) => self.arrow_function(),

            (TokenKind::Function, _) => self.anonymous_function(),

            (TokenKind::Eval, TokenKind::LeftParen) => {
                let start_span = self.state.current().span;
                let eval = self.state.current().span;
                self.state.next();

                let argument = Box::new(self.single_argument(true, true).unwrap());
                let end_span = self.state.previous().span;

                Expression::new(
                    self.state.id(),
                    ExpressionKind::Eval(EvalExpression {
                        id: self.state.id(),
                        span: Span::combine(start_span, end_span),
                        eval,
                        argument,
                    }),
                    Span::new(start_span.start, end_span.end),
                    CommentGroup::default(),
                )
            }

            (TokenKind::Empty, TokenKind::LeftParen) => {
                let start_span = self.state.current().span;
                let empty = self.state.current().span;
                self.state.next();

                let argument = Box::new(self.single_argument(true, true).unwrap());
                let end_span = self.state.previous().span;
                let span = Span::combine(start_span, end_span);

                Expression::new(
                    self.state.id(),
                    ExpressionKind::Empty(EmptyExpression {
                        id: self.state.id(),
                        span,
                        empty,
                        argument,
                    }),
                    span,
                    CommentGroup::default(),
                )
            }

            (TokenKind::Die, _) => {
                let start_span = self.state.current().span;
                let die = self.state.current().span;
                self.state.next();

                let argument = self.single_argument(false, true).map(Box::new);

                let end_span = self.state.previous().span;
                let span = Span::combine(start_span, end_span);

                Expression::new(
                    self.state.id(),
                    ExpressionKind::Die(DieExpression {
                        id: self.state.id(),
                        span,
                        die,
                        argument,
                    }),
                    span,
                    CommentGroup::default(),
                )
            }

            (TokenKind::Exit, _) => {
                let start_span = self.state.current().span;
                let exit = self.state.current().span;
                self.state.next();

                let argument = self.single_argument(false, true).map(Box::new);

                let end_span = self.state.previous().span;
                let span = Span::combine(start_span, end_span);

                Expression::new(
                    self.state.id(),
                    ExpressionKind::Exit(ExitExpression {
                        id: self.state.id(),
                        span,
                        exit,
                        argument,
                    }),
                    span,
                    CommentGroup::default(),
                )
            }

            (TokenKind::Isset, TokenKind::LeftParen) => {
                let start_span = self.state.current().span;
                let isset = self.state.current().span;
                self.state.next();
                let arguments = self.argument_list();
                let end_span = self.state.previous().span;
                let span = Span::combine(start_span, end_span);

                Expression::new(
                    self.state.id(),
                    ExpressionKind::Isset(IssetExpression {
                        id: self.state.id(),
                        span,
                        isset,
                        arguments,
                    }),
                    span,
                    CommentGroup::default(),
                )
            }

            (TokenKind::Unset, TokenKind::LeftParen) => {
                let start_span = self.state.current().span;
                let unset = self.state.current().span;
                self.state.next();
                let arguments = self.argument_list();
                let end_span = self.state.previous().span;
                let span = Span::combine(start_span, end_span);

                Expression::new(
                    self.state.id(),
                    ExpressionKind::Unset(UnsetExpression {
                        id: self.state.id(),
                        span,
                        unset,
                        arguments,
                    }),
                    span,
                    CommentGroup::default(),
                )
            }

            (TokenKind::Print, _) => {
                let start_span = self.state.current().span;
                let print = self.state.current().span;
                self.state.next();

                let mut value = None;
                let mut argument = None;

                if let Some(arg) = self.single_argument(false, true) {
                    argument = Some(Box::new(arg));
                } else {
                    value = Some(Box::new(self.create()));
                }

                let end_span = self.state.previous().span;
                let span = Span::combine(start_span, end_span);

                Expression::new(
                    self.state.id(),
                    ExpressionKind::Print(PrintExpression {
                        id: self.state.id(),
                        span,
                        print,
                        value,
                        argument,
                    }),
                    span,
                    CommentGroup::default(),
                )
            }

            (
                TokenKind::True
                | TokenKind::False
                | TokenKind::Null
                | TokenKind::Readonly
                | TokenKind::Self_
                | TokenKind::Parent
                | TokenKind::Enum
                | TokenKind::From,
                TokenKind::LeftParen,
            ) => {
                let name = self.name_maybe_soft_reserved(UseKind::Function);
                let span = name.span;

                let lhs = Expression::new(
                    self.state.id(),
                    ExpressionKind::Name(name),
                    span,
                    CommentGroup::default(),
                );

                self.postfix(lhs, &TokenKind::LeftParen)
            }

            (TokenKind::Enum | TokenKind::From, TokenKind::DoubleColon) => {
                let name = self.full_name_including_self();
                let span = name.span;

                let lhs = Expression::new(
                    self.state.id(),
                    ExpressionKind::Name(name),
                    span,
                    CommentGroup::default(),
                );

                self.postfix(lhs, &TokenKind::DoubleColon)
            }

            (TokenKind::List, _) => self.list_expression(),

            (TokenKind::New, TokenKind::Class | TokenKind::Attribute) => {
                self.parse_anonymous(None)
            }

            (TokenKind::Throw, _) => {
                let start_span = self.state.current().span;
                self.state.next();
                let exception = self.for_precedence(Precedence::Lowest);
                let exception_span = exception.span;
                let span = Span::combine(start_span, exception_span);

                Expression::new(
                    self.state.id(),
                    ExpressionKind::Throw(ThrowExpression {
                        id: self.state.id(),
                        span,
                        value: Box::new(exception),
                    }),
                    span,
                    CommentGroup::default(),
                )
            }

            (TokenKind::Yield, _) => {
                let start_span = self.state.current().span;
                self.state.next();
                if self.state.current().kind == TokenKind::SemiColon
                    || self.state.current().kind == TokenKind::RightParen
                {
                    Expression::new(
                        self.state.id(),
                        ExpressionKind::Yield(YieldExpression {
                            id: self.state.id(),
                            r#yield: start_span,
                            span: start_span,
                            key: None,
                            value: None,
                        }),
                        start_span,
                        CommentGroup::default(),
                    )
                } else {
                    let mut from = Span::default();

                    if self.state.current().kind == TokenKind::From {
                        from = self.state.current().span;
                        self.state.next();
                    }

                    let mut key = None;
                    let mut value = Box::new(self.create());

                    if self.state.current().kind == TokenKind::DoubleArrow && !from.is_empty() {
                        self.state.next();
                        key = Some(value.clone());
                        value = Box::new(self.create());
                    }

                    let end_span = self.state.previous().span;
                    let span = Span::new(start_span.start, end_span.end);

                    if !from.is_empty() {
                        Expression::new(
                            self.state.id(),
                            ExpressionKind::YieldFrom(YieldFromExpression {
                                id: self.state.id(),
                                r#yield: start_span,
                                from,
                                span,
                                value,
                            }),
                            span,
                            CommentGroup::default(),
                        )
                    } else {
                        Expression::new(
                            self.state.id(),
                            ExpressionKind::Yield(YieldExpression {
                                id: self.state.id(),
                                span,
                                r#yield: start_span,
                                key,
                                value: Some(value),
                            }),
                            span,
                            CommentGroup::default(),
                        )
                    }
                }
            }

            (TokenKind::Clone, _) => {
                let start_span = self.state.current().span;
                self.state.next();

                let target = self.for_precedence(Precedence::CloneOrNew);

                let end_span = self.state.previous().span;
                let span = Span::new(start_span.start, end_span.end);

                Expression::new(
                    self.state.id(),
                    ExpressionKind::Clone(CloneExpression {
                        id: self.state.id(),
                        span,
                        clone: start_span,
                        target: Box::new(target),
                    }),
                    span,
                    CommentGroup::default(),
                )
            }

            (TokenKind::True, _) => {
                let span = self.state.current().span;
                let value = self.state.current().clone();
                self.state.next();

                Expression::new(
                    self.state.id(),
                    ExpressionKind::Bool(BoolExpression {
                        id: self.state.id(),
                        span,
                        value: value.to_owned(),
                    }),
                    span,
                    CommentGroup::default(),
                )
            }

            (TokenKind::False, _) => {
                let span = self.state.current().span;
                let value = self.state.current().clone();
                self.state.next();

                Expression::new(
                    self.state.id(),
                    ExpressionKind::Bool(BoolExpression {
                        id: self.state.id(),
                        span,
                        value: value.to_owned(),
                    }),
                    span,
                    CommentGroup::default(),
                )
            }

            (TokenKind::Null, _) => {
                let span = self.state.current().span;
                self.state.next();

                Expression::new(
                    self.state.id(),
                    ExpressionKind::Null(span),
                    span,
                    CommentGroup::default(),
                )
            }

            (TokenKind::LiteralInteger, _) => {
                let span = self.state.current().span;
                let current = self.state.current();

                if let TokenKind::LiteralInteger = &current.kind {
                    self.state.next();

                    Expression::new(
                        self.state.id(),
                        ExpressionKind::Literal(Literal::new(
                            self.state.id(),
                            LiteralKind::Integer,
                            current.to_owned(),
                            span,
                        )),
                        span,
                        CommentGroup::default(),
                    )
                } else {
                    unreachable!("{}:{}", file!(), line!());
                }
            }

            (TokenKind::LiteralFloat, _) => {
                let span = self.state.current().span;
                let current = self.state.current();

                if let TokenKind::LiteralFloat = &current.kind {
                    self.state.next();

                    Expression::new(
                        self.state.id(),
                        ExpressionKind::Literal(Literal::new(
                            self.state.id(),
                            LiteralKind::Float,
                            current.to_owned(),
                            span,
                        )),
                        span,
                        CommentGroup::default(),
                    )
                } else {
                    unreachable!("{}:{}", file!(), line!());
                }
            }

            (TokenKind::LiteralSingleQuotedString | TokenKind::LiteralDoubleQuotedString, _) => {
                let span = self.state.current().span;
                let current = self.state.current();

                if let TokenKind::LiteralSingleQuotedString = &current.kind {
                    self.state.next();

                    Expression::new(
                        self.state.id(),
                        ExpressionKind::Literal(Literal::new(
                            self.state.id(),
                            LiteralKind::String,
                            current.to_owned(),
                            span,
                        )),
                        span,
                        CommentGroup::default(),
                    )
                } else if let TokenKind::LiteralDoubleQuotedString = &current.kind {
                    self.state.next();

                    Expression::new(
                        self.state.id(),
                        ExpressionKind::Literal(Literal::new(
                            self.state.id(),
                            LiteralKind::String,
                            current.to_owned(),
                            span,
                        )),
                        span,
                        CommentGroup::default(),
                    )
                } else {
                    unreachable!("{}:{}", file!(), line!());
                }
            }

            (TokenKind::StringPart, _) => self.interpolated(),

            (TokenKind::StartHeredoc, _) => self.heredoc(),

            (TokenKind::StartNowdoc, _) => self.nowdoc(),

            (TokenKind::Backtick, _) => self.shell_exec(),

            (
                TokenKind::Identifier
                | TokenKind::QualifiedIdentifier
                | TokenKind::FullyQualifiedIdentifier,
                _,
            ) => {
                let name = self.full_name(
                    match self.state.peek().kind {
                        TokenKind::LeftParen => UseKind::Function,
                        TokenKind::DoubleColon => UseKind::Normal,
                        _ => UseKind::Const,
                    },
                );

                let span = name.span;

                Expression::new(
                    self.state.id(),
                    ExpressionKind::Name(name),
                    span,
                    CommentGroup::default(),
                )
            }

            (TokenKind::Static, _) => {
                let span = self.state.current().span;
                self.state.next();
                let expression = Expression::new(
                    self.state.id(),
                    ExpressionKind::Static(StaticExpression {
                        id: self.state.id(),
                        span,
                    }),
                    span,
                    CommentGroup::default(),
                );

                self.postfix(expression, &TokenKind::DoubleColon)
            }

            (TokenKind::Self_, _) => {
                let span = self.state.current().span;
                self.state.next();

                Expression::new(
                    self.state.id(),
                    ExpressionKind::Self_(SelfExpression {
                        id: self.state.id(),
                        span,
                    }),
                    span,
                    CommentGroup::default(),
                )
            }

            (TokenKind::Parent, _) => {
                let span = self.state.current().span;
                self.state.next();

                Expression::new(
                    self.state.id(),
                    ExpressionKind::Parent(ParentExpression {
                        id: self.state.id(),
                        span,
                    }),
                    span,
                    CommentGroup::default(),
                )
            }

            (TokenKind::LeftParen, _) => {
                let start = self.state.current().span;
                self.state.next();

                let expr = self.create();

                let end = self.skip_right_parenthesis();
                let span = Span::combine(start, end);

                Expression::new(
                    self.state.id(),
                    ExpressionKind::Parenthesized(ParenthesizedExpression {
                        id: self.state.id(),
                        span,
                        start,
                        expr: Box::new(expr),
                        end,
                    }),
                    span,
                    CommentGroup::default(),
                )
            }

            (TokenKind::Match, _) => self.match_expression(),

            (TokenKind::Array, _) => self.array_expression(),

            (TokenKind::LeftBracket, _) => self.short_array_expression(),

            (TokenKind::New, _) => {
                let new = self.state.current().span;

                self.state.next();

                if self.state.current().kind == TokenKind::Class
                    || self.state.current().kind == TokenKind::Attribute
                {
                    return self.parse_anonymous(Some(new));
                };

                let target = match self.state.current().kind {
                    TokenKind::Self_ => {
                        let token = self.state.current();

                        self.state.next();

                        Expression::new(
                            self.state.id(),
                            ExpressionKind::Name(Name::special(
                                self.state.id(),
                                SpecialNameKind::Self_(token.span),
                                token.symbol.to_bytestring(),
                                token.span,
                            )),
                            token.span,
                            CommentGroup::default(),
                        )
                    }
                    TokenKind::Static => {
                        let token = self.state.current();

                        self.state.next();

                        Expression::new(
                            self.state.id(),
                            ExpressionKind::Name(Name::special(
                                self.state.id(),
                                SpecialNameKind::Static(token.span),
                                token.symbol.to_bytestring(),
                                token.span,
                            )),
                            token.span,
                            CommentGroup::default(),
                        )
                    }
                    TokenKind::Parent => {
                        let token = self.state.current();

                        self.state.next();

                        Expression::new(
                            self.state.id(),
                            ExpressionKind::Name(Name::special(
                                self.state.id(),
                                SpecialNameKind::Parent(token.span),
                                token.symbol.to_bytestring(),
                                token.span,
                            )),
                            token.span,
                            CommentGroup::default(),
                        )
                    }
                    TokenKind::FullyQualifiedIdentifier => {
                        let token = self.state.current();

                        let span = token.span;
                        let symbol = token.symbol.to_bytestring();
                        let resolved = self.state.strip_leading_namespace_qualifier(&symbol);

                        self.state.next();

                        Expression::new(
                            self.state.id(),
                            ExpressionKind::Name(Name::resolved(
                                self.state.id(),
                                resolved,
                                symbol,
                                span,
                            )),
                            span,
                            CommentGroup::default(),
                        )
                    }
                    TokenKind::Identifier
                    | TokenKind::QualifiedIdentifier
                    | TokenKind::Enum
                    | TokenKind::From => {
                        let token = self.state.current();

                        self.state.next();

                        Expression::new(
                            self.state.id(),
                            ExpressionKind::Name(
                                self.state.maybe_resolve_identifier(&token, UseKind::Normal),
                            ),
                            token.span,
                            CommentGroup::default(),
                        )
                    }
                    _ => self.clone_or_new_precedence(),
                };

                let arguments = if self.state.current().kind == TokenKind::LeftParen {
                    Some(self.argument_list())
                } else {
                    None
                };

                let span = Span::combine(new, self.state.previous().span);

                Expression::new(
                    self.state.id(),
                    ExpressionKind::New(NewExpression {
                        id: self.state.id(),
                        span,
                        target: Box::new(target),
                        new,
                        arguments,
                    }),
                    span,
                    CommentGroup::default(),
                )
            }

            (TokenKind::DirConstant, _) => {
                let span = self.state.current().span;
                self.state.next();

                Expression::new(
                    self.state.id(),
                    ExpressionKind::MagicConstant(MagicConstantExpression {
                        id: self.state.id(),
                        span,
                        kind: MagicConstantKind::Directory,
                    }),
                    span,
                    CommentGroup::default(),
                )
            }

            (TokenKind::FileConstant, _) => {
                let span = self.state.current().span;
                self.state.next();

                Expression::new(
                    self.state.id(),
                    ExpressionKind::MagicConstant(MagicConstantExpression {
                        id: self.state.id(),
                        span,
                        kind: MagicConstantKind::File,
                    }),
                    span,
                    CommentGroup::default(),
                )
            }

            (TokenKind::LineConstant, _) => {
                let span = self.state.current().span;
                self.state.next();

                Expression::new(
                    self.state.id(),
                    ExpressionKind::MagicConstant(MagicConstantExpression {
                        id: self.state.id(),
                        span,
                        kind: MagicConstantKind::Line,
                    }),
                    span,
                    CommentGroup::default(),
                )
            }

            (TokenKind::FunctionConstant, _) => {
                let span = self.state.current().span;
                self.state.next();

                Expression::new(
                    self.state.id(),
                    ExpressionKind::MagicConstant(MagicConstantExpression {
                        id: self.state.id(),
                        span,
                        kind: MagicConstantKind::Function,
                    }),
                    span,
                    CommentGroup::default(),
                )
            }

            (TokenKind::ClassConstant, _) => {
                let span = self.state.current().span;
                self.state.next();

                Expression::new(
                    self.state.id(),
                    ExpressionKind::MagicConstant(MagicConstantExpression {
                        id: self.state.id(),
                        span,
                        kind: MagicConstantKind::Class,
                    }),
                    span,
                    CommentGroup::default(),
                )
            }

            (TokenKind::MethodConstant, _) => {
                let span = self.state.current().span;
                self.state.next();

                Expression::new(
                    self.state.id(),
                    ExpressionKind::MagicConstant(MagicConstantExpression {
                        id: self.state.id(),
                        span,
                        kind: MagicConstantKind::Method,
                    }),
                    span,
                    CommentGroup::default(),
                )
            }

            (TokenKind::NamespaceConstant, _) => {
                let span = self.state.current().span;
                self.state.next();

                Expression::new(
                    self.state.id(),
                    ExpressionKind::MagicConstant(MagicConstantExpression {
                        id: self.state.id(),
                        span,
                        kind: MagicConstantKind::Namespace,
                    }),
                    span,
                    CommentGroup::default(),
                )
            }

            (TokenKind::TraitConstant, _) => {
                let span = self.state.current().span;
                self.state.next();

                Expression::new(
                    self.state.id(),
                    ExpressionKind::MagicConstant(MagicConstantExpression {
                        id: self.state.id(),
                        span,
                        kind: MagicConstantKind::Trait,
                    }),
                    span,
                    CommentGroup::default(),
                )
            }

            (TokenKind::CompilerHaltOffsetConstant, _) => {
                let span = self.state.current().span;
                self.state.next();

                Expression::new(
                    self.state.id(),
                    ExpressionKind::MagicConstant(MagicConstantExpression {
                        id: self.state.id(),
                        span,
                        kind: MagicConstantKind::CompilerHaltOffset,
                    }),
                    span,
                    CommentGroup::default(),
                )
            }

            (
                TokenKind::Include
                | TokenKind::IncludeOnce
                | TokenKind::Require
                | TokenKind::RequireOnce,
                _,
            ) => {
                let start_span = self.state.current().span;
                let current = self.state.current();
                let keyword_span = current.span;

                self.state.next();

                let path = self.create();
                let span = Span::combine(start_span, path.span);
                let path = Box::new(path);

                let kind = match current.kind {
                    TokenKind::Include => ExpressionKind::Include(IncludeExpression {
                        id: self.state.id(),
                        span,
                        include: keyword_span,
                        path,
                    }),
                    TokenKind::IncludeOnce => ExpressionKind::IncludeOnce(IncludeOnceExpression {
                        id: self.state.id(),
                        span,
                        include_once: keyword_span,
                        path,
                    }),
                    TokenKind::Require => ExpressionKind::Require(RequireExpression {
                        id: self.state.id(),
                        span,
                        require: keyword_span,
                        path,
                    }),
                    TokenKind::RequireOnce => ExpressionKind::RequireOnce(RequireOnceExpression {
                        id: self.state.id(),
                        span,
                        require_once: keyword_span,
                        path,
                    }),
                    _ => unreachable!(),
                };

                let end_span = self.state.previous().span;

                Expression::new(
                    self.state.id(),
                    kind,
                    Span::new(start_span.start, end_span.end),
                    CommentGroup::default(),
                )
            }

            (
                TokenKind::StringCast
                | TokenKind::BinaryCast
                | TokenKind::ObjectCast
                | TokenKind::BoolCast
                | TokenKind::BooleanCast
                | TokenKind::IntCast
                | TokenKind::IntegerCast
                | TokenKind::FloatCast
                | TokenKind::DoubleCast
                | TokenKind::RealCast
                | TokenKind::UnsetCast
                | TokenKind::ArrayCast,
                _,
            ) => {
                let current = self.state.current();

                let span = current.span;
                let kind = current.clone().into();

                self.state.next();

                let rhs = self.for_precedence(Precedence::Prefix);
                let rhs_span = rhs.span;

                Expression::new(
                    self.state.id(),
                    ExpressionKind::Cast(CastExpression {
                        id: self.state.id(),
                        span,
                        kind,
                        value: Box::new(rhs),
                    }),
                    Span::new(span.start, rhs_span.end),
                    CommentGroup::default(),
                )
            }

            (
                TokenKind::Decrement | TokenKind::Increment | TokenKind::Minus | TokenKind::Plus,
                _,
            ) => {
                let start_span = self.state.current().span;
                let current = self.state.current();

                let op_span = current.span;
                let op = current.kind;

                self.state.next();

                let right = Box::new(self.for_precedence(Precedence::Prefix));
                let right_span = right.span;
                let span = Span::combine(start_span, right_span);

                let expr = match op {
                    TokenKind::Minus => {
                        ExpressionKind::ArithmeticOperation(ArithmeticOperationExpression {
                            id: self.state.id(),
                            span,
                            kind: ArithmeticOperationKind::Negative {
                                id: self.state.id(),
                                minus: op_span,
                                right,
                            },
                        })
                    }
                    TokenKind::Plus => {
                        ExpressionKind::ArithmeticOperation(ArithmeticOperationExpression {
                            id: self.state.id(),
                            span,
                            kind: ArithmeticOperationKind::Positive {
                                id: self.state.id(),
                                plus: op_span,
                                right,
                            },
                        })
                    }
                    TokenKind::Decrement => {
                        ExpressionKind::ArithmeticOperation(ArithmeticOperationExpression {
                            id: self.state.id(),
                            span,
                            kind: ArithmeticOperationKind::PreDecrement {
                                id: self.state.id(),
                                decrement: op_span,
                                right,
                            },
                        })
                    }
                    TokenKind::Increment => {
                        ExpressionKind::ArithmeticOperation(ArithmeticOperationExpression {
                            id: self.state.id(),
                            span,
                            kind: ArithmeticOperationKind::PreIncrement {
                                id: self.state.id(),
                                increment: op_span,
                                right,
                            },
                        })
                    }
                    _ => unreachable!(),
                };

                Expression::new(
                    self.state.id(),
                    expr,
                    Span::new(start_span.start, right_span.end),
                    CommentGroup::default(),
                )
            }

            (TokenKind::Bang, _) => {
                let start_span = self.state.current().span;
                let bang = self.state.current().span;

                self.state.next();

                let rhs = self.for_precedence(Precedence::Bang);
                let end_span = rhs.span;
                let span = Span::combine(start_span, end_span);

                Expression::new(
                    self.state.id(),
                    ExpressionKind::LogicalOperation(LogicalOperationExpression {
                        id: self.state.id(),
                        span,
                        kind: LogicalOperationKind::Not {
                            id: self.state.id(),
                            bang,
                            right: Box::new(rhs),
                        },
                    }),
                    Span::new(start_span.start, end_span.end),
                    CommentGroup::default(),
                )
            }

            (TokenKind::At, _) => {
                let span = self.state.current().span;

                self.state.next();

                let rhs = self.for_precedence(Precedence::Prefix);
                let end_span = rhs.span;
                let span = Span::combine(span, end_span);

                Expression::new(
                    self.state.id(),
                    ExpressionKind::ErrorSuppress(ErrorSuppressExpression {
                        id: self.state.id(),
                        span,
                        at: span,
                        expr: Box::new(rhs),
                    }),
                    Span::new(span.start, end_span.end),
                    CommentGroup::default(),
                )
            }

            (TokenKind::BitwiseNot, _) => {
                let span = self.state.current().span;

                self.state.next();

                let right = Box::new(self.for_precedence(Precedence::Prefix));
                let end_span = right.span;
                let span = Span::combine(span, end_span);

                Expression::new(
                    self.state.id(),
                    ExpressionKind::BitwiseOperation(BitwiseOperationExpression {
                        span,
                        kind: BitwiseOperationKind::Not {
                            id: self.state.id(),
                            not: span,
                            right,
                        },
                        id: self.state.id(),
                    }),
                    Span::new(span.start, end_span.end),
                    CommentGroup::default(),
                )
            }

            (TokenKind::Dollar | TokenKind::DollarLeftBrace | TokenKind::Variable, _) => {
                let span = self.state.current().span;

                Expression::new(
                    self.state.id(),
                    ExpressionKind::Variable(self.dynamic_variable()),
                    span,
                    CommentGroup::default(),
                )
            }

            _ => self.unexpected_token(precedence),
        }
    }

    fn unexpected_token(&mut self, _: &Precedence) -> Expression {
        let current = self.state.current().to_owned();
        let is_right_brace = current.kind == TokenKind::RightBrace;
        let span = current.span;

        self.state.diagnostic(
            ParserDiagnostic::UnexpectedToken {
                token: current,
            },
            Severity::Error,
            span,
        );

        // This is a common case where we don't want to consume the right-brace as it might close a structure.
        if is_right_brace {
            self.state.next();
        }

        Expression::missing(self.state.id(), span)
    }

    fn postfix(&mut self, lhs: Expression, op: &TokenKind) -> Expression {
        let start_span = self.state.current().span;
        let kind = match op {
            TokenKind::DoubleQuestion => {
                let double_question = self.state.current().span;
                self.state.next();

                let rhs = self.null_coalesce_precedence();
                let span = Span::combine(lhs.span, rhs.span);

                ExpressionKind::Coalesce(CoalesceExpression {
                    id: self.state.id(),
                    span,
                    lhs: Box::new(lhs),
                    double_question,
                    rhs: Box::new(rhs),
                })
            }
            TokenKind::LeftParen => {
                // `(...)` closure creation
                if self.state.lookahead(0).kind == TokenKind::Ellipsis
                    && self.state.lookahead(1).kind == TokenKind::RightParen
                {
                    let start = self.skip(TokenKind::LeftParen);
                    let ellipsis = self.skip(TokenKind::Ellipsis);
                    let end = self.skip(TokenKind::RightParen);
                    let span = Span::combine(start, end);

                    let placeholder = ArgumentPlaceholder {
                        id: self.state.id(),
                        span,
                        comments: self.state.comments(),
                        left_parenthesis: start,
                        ellipsis,
                        right_parenthesis: end,
                    };

                    let span = Span::combine(lhs.span, span);

                    ExpressionKind::FunctionClosureCreation(FunctionClosureCreationExpression {
                        id: self.state.id(),
                        span,
                        target: Box::new(lhs),
                        placeholder,
                    })
                } else {
                    let arguments = self.argument_list();
                    let span = Span::combine(lhs.span, arguments.span);

                    ExpressionKind::FunctionCall(FunctionCallExpression {
                        id: self.state.id(),
                        span,
                        target: Box::new(lhs),
                        arguments,
                    })
                }
            }
            TokenKind::LeftBracket => {
                let left_bracket = self.skip_left_bracket();
                
                let index = if self.state.is(TokenKind::RightBracket) {
                    None
                } else {
                    Some(Box::new(self.create()))
                };

                let right_bracket = self.skip_right_bracket();
                let span = Span::combine(lhs.span, right_bracket);

                ExpressionKind::ArrayIndex(ArrayIndexExpression {
                    id: self.state.id(),
                    span,
                    array: Box::new(lhs),
                    left_bracket,
                    index,
                    right_bracket,
                })
            }
            TokenKind::DoubleColon => {
                let double_colon = self.skip_double_colon();
                let current = self.state.current();

                let property = match current.kind {
                    TokenKind::Variable | TokenKind::Dollar | TokenKind::DollarLeftBrace => {
                        ExpressionKind::Variable(self.dynamic_variable())
                    }
                    _ if self.is_identifier_maybe_reserved(&self.state.current().kind) => {
                        ExpressionKind::Identifier(Identifier::SimpleIdentifier(
                            self.identifier_maybe_reserved(),
                        ))
                    }
                    TokenKind::LeftBrace => {
                        let start = current.span;

                        self.state.next();

                        let expr = Box::new(self.create());
                        let end = self.skip_right_brace();
                        let span = Span::new(start.start, end.end);

                        ExpressionKind::Identifier(Identifier::DynamicIdentifier(
                            DynamicIdentifier {
                                id: self.state.id(),
                                span,
                                expr,
                            },
                        ))
                    }
                    TokenKind::Class => {
                        self.state.next();

                        let symbol = current.symbol.to_bytestring();

                        ExpressionKind::Identifier(Identifier::SimpleIdentifier(
                            SimpleIdentifier::new(self.state.id(), symbol, current.span),
                        ))
                    }
                    _ => {
                        self.state.diagnostic(
                            ParserDiagnostic::ExpectedToken {
                                expected: vec![
                                    TokenKind::LeftBrace,
                                    TokenKind::Dollar,
                                    TokenKind::Identifier,
                                ],
                                found: current.to_owned(),
                            },
                            Severity::Error,
                            current.span,
                        );

                        self.state.next();

                        ExpressionKind::Missing(MissingExpression {
                            id: 0,
                            span: current.span,
                        })
                    }
                };

                let lhs = Box::new(lhs);

                if self.state.current().kind == TokenKind::LeftParen {
                    if self.state.lookahead(0).kind == TokenKind::Ellipsis
                        && self.state.lookahead(1).kind == TokenKind::RightParen
                    {
                        let start = self.skip(TokenKind::LeftParen);
                        let ellipsis = self.skip(TokenKind::Ellipsis);
                        let end = self.skip(TokenKind::RightParen);
                        let span = Span::combine(start, end);

                        let placeholder = ArgumentPlaceholder {
                            id: self.state.id(),
                            span,
                            comments: self.state.comments(),
                            left_parenthesis: start,
                            ellipsis,
                            right_parenthesis: end,
                        };

                        match property {
                            ExpressionKind::Identifier(identifier) => {
                                ExpressionKind::StaticMethodClosureCreation(
                                    StaticMethodClosureCreationExpression {
                                        id: self.state.id(),
                                        span: Span::combine(lhs.span, placeholder.span),
                                        target: lhs,
                                        double_colon,
                                        method: identifier,
                                        placeholder,
                                    },
                                )
                            }
                            ExpressionKind::Variable(variable) => {
                                ExpressionKind::StaticVariableMethodClosureCreation(
                                    StaticVariableMethodClosureCreationExpression {
                                        id: self.state.id(),
                                        span: Span::combine(lhs.span, placeholder.span),
                                        target: lhs,
                                        double_colon,
                                        method: variable,
                                        placeholder,
                                    },
                                )
                            }
                            _ => unreachable!(),
                        }
                    } else {
                        let arguments = self.argument_list();

                        match property {
                            ExpressionKind::Identifier(identifier) => {
                                ExpressionKind::StaticMethodCall(StaticMethodCallExpression {
                                    id: self.state.id(),
                                    span: Span::combine(lhs.span, arguments.span),
                                    target: lhs,
                                    double_colon,
                                    method: identifier,
                                    arguments,
                                })
                            }
                            ExpressionKind::Variable(variable) => {
                                ExpressionKind::StaticVariableMethodCall(
                                    StaticVariableMethodCallExpression {
                                        id: self.state.id(),
                                        span: Span::combine(lhs.span, arguments.span),
                                        target: lhs,
                                        double_colon,
                                        method: variable,
                                        arguments,
                                    },
                                )
                            }
                            _ => unreachable!(),
                        }
                    }
                } else {
                    match property {
                        ExpressionKind::Identifier(identifier) => {
                            ExpressionKind::ConstantFetch(ConstantFetchExpression {
                                id: self.state.id(),
                                span: Span::combine(lhs.span, identifier.span()),
                                target: lhs,
                                double_colon,
                                constant: identifier,
                            })
                        }
                        ExpressionKind::Variable(variable) => {
                            ExpressionKind::StaticPropertyFetch(StaticPropertyFetchExpression {
                                id: self.state.id(),
                                span: Span::combine(lhs.span, variable.span()),
                                target: lhs,
                                double_colon,
                                property: variable,
                            })
                        }
                        _ => {
                            let span = Span::combine(lhs.span, double_colon);

                            ExpressionKind::ConstantFetch(ConstantFetchExpression {
                                id: self.state.id(),
                                span,
                                target: lhs,
                                double_colon,
                                constant: Identifier::missing(
                                    self.state.id(),
                                    Span::flat(double_colon.end),
                                ),
                            })
                        }
                    }
                }
            }
            TokenKind::Arrow | TokenKind::QuestionArrow => {
                let span = self.state.current().span;
                self.state.next();

                let property = match self.state.current().kind {
                    TokenKind::Variable | TokenKind::Dollar | TokenKind::DollarLeftBrace => {
                        let start_span = self.state.current().span;
                        let kind = ExpressionKind::Variable(self.dynamic_variable());
                        let end_span = self.state.previous().span;

                        Expression::new(
                            self.state.id(),
                            kind,
                            Span::new(start_span.start, end_span.end),
                            CommentGroup::default(),
                        )
                    }
                    _ if self.is_identifier_maybe_reserved(&self.state.current().kind) => {
                        let start_span = self.state.current().span;
                        let kind = ExpressionKind::Identifier(Identifier::SimpleIdentifier(
                            self.identifier_maybe_reserved(),
                        ));
                        let end_span = self.state.previous().span;

                        Expression::new(
                            self.state.id(),
                            kind,
                            Span::new(start_span.start, end_span.end),
                            CommentGroup::default(),
                        )
                    }
                    TokenKind::LeftBrace => {
                        let start = self.state.current().span;
                        self.state.next();

                        let name = create();

                        let end = self.skip_right_brace();
                        let span = Span::new(start.start, end.end);

                        Expression::new(
                            self.state.id(),
                            ExpressionKind::Identifier(Identifier::DynamicIdentifier(
                                DynamicIdentifier {
                                    id: self.state.id(),
                                    span,
                                    expr: Box::new(name),
                                },
                            )),
                            Span::new(start.start, end.end),
                            CommentGroup::default(),
                        )
                    }
                    _ => {
                        let span = self.state.current().span;

                        self.state.diagnostic(
                            ParserDiagnostic::ExpectedToken {
                                expected: vec![
                                    TokenKind::LeftBrace,
                                    TokenKind::Dollar,
                                    TokenKind::Identifier,
                                ],
                                found: self.state.current().to_owned(),
                            },
                            Severity::Error,
                            span,
                        );

                        Expression::missing(self.state.id(), span)
                    }
                };

                if self.state.current().kind == TokenKind::LeftParen {
                    if op == &TokenKind::QuestionArrow {
                        let arguments = self.argument_list();

                        ExpressionKind::NullsafeMethodCall(NullsafeMethodCallExpression {
                            id: self.state.id(),
                            span: Span::combine(lhs.span, arguments.span),
                            target: Box::new(lhs),
                            method: Box::new(property),
                            question_arrow: span,
                            arguments,
                        })
                    } else {
                        // `(...)` closure creation
                        if self.state.lookahead(0).kind == TokenKind::Ellipsis
                            && self.state.lookahead(1).kind == TokenKind::RightParen
                        {
                            let start = self.skip(TokenKind::LeftParen);
                            let ellipsis = self.skip(TokenKind::Ellipsis);
                            let end = self.skip(TokenKind::RightParen);
                            let span = Span::combine(start, end);

                            let placeholder = ArgumentPlaceholder {
                                id: self.state.id(),
                                span,
                                comments: self.state.comments(),
                                left_parenthesis: start,
                                ellipsis,
                                right_parenthesis: end,
                            };

                            ExpressionKind::MethodClosureCreation(MethodClosureCreationExpression {
                                id: self.state.id(),
                                span: Span::combine(lhs.span, placeholder.span),
                                target: Box::new(lhs),
                                method: Box::new(property),
                                arrow: span,
                                placeholder,
                            })
                        } else {
                            let arguments = self.argument_list();

                            ExpressionKind::MethodCall(MethodCallExpression {
                                id: self.state.id(),
                                span: Span::combine(lhs.span, arguments.span),
                                target: Box::new(lhs),
                                method: Box::new(property),
                                arrow: span,
                                arguments,
                            })
                        }
                    }
                } else if op == &TokenKind::QuestionArrow {
                    ExpressionKind::NullsafePropertyFetch(NullsafePropertyFetchExpression {
                        id: self.state.id(),
                        span: Span::combine(lhs.span, property.span),
                        target: Box::new(lhs),
                        question_arrow: span,
                        property: Box::new(property),
                    })
                } else {
                    ExpressionKind::PropertyFetch(PropertyFetchExpression {
                        id: self.state.id(),
                        span: Span::combine(lhs.span, property.span),
                        target: Box::new(lhs),
                        arrow: span,
                        property: Box::new(property),
                    })
                }
            }
            TokenKind::Increment => {
                let op = self.state.current().span;
                self.state.next();

                ExpressionKind::ArithmeticOperation(ArithmeticOperationExpression {
                    id: self.state.id(),
                    span: Span::combine(lhs.span, op),
                    kind: ArithmeticOperationKind::PostIncrement {
                        id: self.state.id(),
                        left: Box::new(lhs),
                        increment: op,
                    },
                })
            }
            TokenKind::Decrement => {
                let op = self.state.current().span;
                self.state.next();

                ExpressionKind::ArithmeticOperation(ArithmeticOperationExpression {
                    id: self.state.id(),
                    span: Span::combine(lhs.span, op),
                    kind: ArithmeticOperationKind::PostDecrement {
                        id: self.state.id(),
                        left: Box::new(lhs),
                        decrement: op,
                    },
                })
            }
            _ => unreachable!(),
        };

        let end_span = self.state.previous().span;

        Expression::new(
            self.state.id(),
            kind,
            Span::new(start_span.start, end_span.end),
            CommentGroup::default(),
        )
    }

    fn is_infix(&self, t: &TokenKind) -> bool {
        matches!(
            t,
            TokenKind::Pow
                | TokenKind::RightShiftEquals
                | TokenKind::LeftShiftEquals
                | TokenKind::CaretEquals
                | TokenKind::AmpersandEquals
                | TokenKind::PipeEquals
                | TokenKind::PercentEquals
                | TokenKind::PowEquals
                | TokenKind::LogicalAnd
                | TokenKind::LogicalOr
                | TokenKind::LogicalXor
                | TokenKind::Spaceship
                | TokenKind::LeftShift
                | TokenKind::RightShift
                | TokenKind::Ampersand
                | TokenKind::Pipe
                | TokenKind::Caret
                | TokenKind::Percent
                | TokenKind::Instanceof
                | TokenKind::Asterisk
                | TokenKind::Slash
                | TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Dot
                | TokenKind::LessThan
                | TokenKind::GreaterThan
                | TokenKind::LessThanEquals
                | TokenKind::GreaterThanEquals
                | TokenKind::DoubleEquals
                | TokenKind::TripleEquals
                | TokenKind::BangEquals
                | TokenKind::BangDoubleEquals
                | TokenKind::AngledLeftRight
                | TokenKind::Question
                | TokenKind::QuestionColon
                | TokenKind::BooleanAnd
                | TokenKind::BooleanOr
                | TokenKind::Equals
                | TokenKind::PlusEquals
                | TokenKind::MinusEquals
                | TokenKind::DotEquals
                | TokenKind::DoubleQuestionEquals
                | TokenKind::AsteriskEquals
                | TokenKind::SlashEquals
        )
    }

    #[inline(always)]
    fn is_postfix(&self, t: &TokenKind) -> bool {
        matches!(
            t,
            TokenKind::Increment
                | TokenKind::Decrement
                | TokenKind::LeftParen
                | TokenKind::LeftBracket
                | TokenKind::Arrow
                | TokenKind::QuestionArrow
                | TokenKind::DoubleColon
                | TokenKind::DoubleQuestion
        )
    }
}
