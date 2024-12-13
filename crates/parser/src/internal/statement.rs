use pxp_ast::{ClosingTagStatement, DeclareBody, DeclareBodyBlock, DeclareBodyBraced, DeclareBodyExpression, DeclareBodyNoop, DeclareEntry, DeclareEntryGroup, DeclareStatement, EchoOpeningTagStatement, EchoStatement, ExpressionStatement, FullOpeningTagStatement, GlobalStatement, HaltCompilerStatement, InlineHtmlStatement, ReturnStatement, ShortOpeningTagStatement, Statement, StatementKind, StaticStatement, StaticVar, Variable};
use pxp_span::{Span, Spanned};
use pxp_token::{OpenTagKind, TokenKind};

use crate::Parser;

impl<'a> Parser<'a> {
    pub(crate) fn parse_top_level_statement(&mut self) -> Statement {
        match self.current_kind() {
            TokenKind::Namespace | TokenKind::Use | TokenKind::Const | TokenKind::HaltCompiler => {
                let comments = self.state.comments();
                let kind = match self.current_kind() {
                    TokenKind::Namespace => self.parse_namespace(),
                    TokenKind::Use => self.parse_use_statement(),
                    TokenKind::Const => StatementKind::Constant(self.parse_constant()),
                    TokenKind::HaltCompiler => {
                        let start = self.next();

                        let (span, content) = if let TokenKind::InlineHtml = self.current_kind() {
                            let content = self.next_but_first(|parser| parser.current().to_owned());

                            (Span::combine(start, content.span), Some(content))
                        } else {
                            (start, None)
                        };

                        StatementKind::HaltCompiler(HaltCompilerStatement {
                            id: self.state.id(),
                            span,
                            content,
                        })
                    }
                    _ => unreachable!(),
                };

                let span = kind.span();

                Statement::new(
                    self.state.id(),
                    kind,
                    span,
                    comments,
                )
            }
            _ => self.parse_statement(),
        }
    }

    pub(crate) fn parse_statement(&mut self) -> Statement {
        let start = self.current_span();
        let comments = self.state.comments();

        let has_attributes = self.gather_attributes();
        let current = self.current();
        let peek = self.peek();

        let statement = if has_attributes {
            match self.current_kind() {
                TokenKind::Abstract => self.parse_class(),
                TokenKind::Readonly if peek.kind != TokenKind::LeftParen => self.parse_class(),
                TokenKind::Final => self.parse_class(),
                TokenKind::Class => self.parse_class(),
                TokenKind::Interface => self.parse_interface(),
                TokenKind::Trait => self.parse_trait(),
                TokenKind::Enum
                    if !matches!(
                        peek.kind,
                        TokenKind::LeftParen | TokenKind::DoubleColon | TokenKind::Colon,
                    ) =>
                {
                    self.parse_enum()
                }
                TokenKind::Function
                    if self.is_identifier_maybe_soft_reserved(&peek.kind)
                        || peek.kind == TokenKind::Ampersand =>
                {
                    if peek.kind == TokenKind::Ampersand {
                        if !self.is_identifier_maybe_soft_reserved(&state.lookahead(1).kind)
                        {
                            let expression = self.attributes();
                            let ending = self.skip_ending();
                            let ending_span = ending.span();

                            let span = Span::combine(start, ending_span);
                            let kind = StatementKind::Expression(ExpressionStatement {
                                id: self.state.id(),
                                span,
                                expression,
                                ending,
                            });

                            return Statement::new(self.state.id(), kind, span, comments);
                        }

                        self.parse_function()
                    } else {
                        self.parse_function()
                    }
                }
                _ => {
                    let start = current.span;
                    let expression = self.attributes();
                    let ending = self.skip_ending();
                    let ending_span = ending.span();

                    StatementKind::Expression(ExpressionStatement {
                        id: self.state.id(),
                        span: Span::combine(start, ending_span),
                        expression,
                        ending,
                    })
                }
            }
        } else {
            match &current.kind {
                TokenKind::OpenTag(OpenTagKind::Echo) => {
                    let span = current.span;
                    self.next();

                    StatementKind::EchoOpeningTag(EchoOpeningTagStatement {
                        id: self.state.id(),
                        span,
                    })
                }
                TokenKind::OpenTag(OpenTagKind::Full) => {
                    let span = current.span;
                    self.next();

                    StatementKind::FullOpeningTag(FullOpeningTagStatement {
                        id: self.state.id(),
                        span,
                    })
                }
                TokenKind::OpenTag(OpenTagKind::Short) => {
                    let span = current.span;
                    self.next();

                    StatementKind::ShortOpeningTag(ShortOpeningTagStatement {
                        id: self.state.id(),
                        span,
                    })
                }
                TokenKind::CloseTag => {
                    let span = current.span;
                    self.next();

                    StatementKind::ClosingTag(ClosingTagStatement {
                        id: self.state.id(),
                        span,
                    })
                }
                TokenKind::Abstract => self.parse_class(),
                TokenKind::Readonly if peek.kind != TokenKind::LeftParen => {
                    self.parse_class()
                }
                TokenKind::Final => self.parse_class(),
                TokenKind::Class => self.parse_class(),
                TokenKind::Interface => self.parse_interface(),
                TokenKind::Trait => self.parse_trait(),
                TokenKind::Enum
                    if !matches!(
                        peek.kind,
                        TokenKind::LeftParen | TokenKind::DoubleColon | TokenKind::Colon,
                    ) =>
                {
                    self.parse_enum()
                }
                TokenKind::Function
                    if self.is_identifier_maybe_soft_reserved(&peek.kind)
                        || peek.kind == TokenKind::Ampersand =>
                {
                    if peek.kind == TokenKind::Ampersand {
                        if !self.is_identifier_maybe_soft_reserved(&state.lookahead(1).kind)
                        {
                            let expression = self.attributes();
                            let ending = self.skip_ending();
                            let ending_span = ending.span();

                            let span = Span::combine(start, ending_span);

                            let kind = StatementKind::Expression(ExpressionStatement {
                                id: self.state.id(),
                                span,
                                expression,
                                ending,
                            });

                            return Statement::new(self.state.id(), kind, span, comments);
                        }

                        self.parse_function()
                    } else {
                        self.parse_function()
                    }
                }
                TokenKind::Goto => self.parse_goto_statement(),
                token
                    if self.is_identifier_maybe_reserved(token)
                        && peek.kind == TokenKind::Colon =>
                {
                    self.parse_label_statement()
                }
                TokenKind::Declare => {
                    let declare = self.skip(TokenKind::Declare);

                    let entries = {
                        let start = self.skip_left_parenthesis();
                        let mut entries = Vec::new();
                        loop {
                            let key = self.parse_identifier();
                            let start = key.span;
                            let equals = self.skip(TokenKind::Equals);
                            let value = self.parse_literal();
                            let end = value.span;

                            entries.push(DeclareEntry {
                                id: self.state.id(),
                                span: Span::combine(start, end),
                                key,
                                equals,
                                value,
                            });

                            if self.current_kind() == TokenKind::Comma {
                                self.next();
                            } else {
                                break;
                            }
                        }

                        let end = self.skip_right_parenthesis();
                        let span = Span::combine(start, end);

                        DeclareEntryGroup {
                            id: self.state.id(),
                            span,
                            left_parenthesis: start,
                            entries,
                            right_parenthesis: end,
                        }
                    };

                    let body = match self.current_kind() {
                        TokenKind::SemiColon => {
                            let span = self.skip_semicolon();

                            DeclareBody::Noop(DeclareBodyNoop {
                                id: self.state.id(),
                                span,
                                semicolon: span,
                            })
                        }
                        TokenKind::LeftBrace => {
                            let start = self.skip_left_brace();
                            let statements = self.parse_multiple_statements_until(
                                &TokenKind::RightBrace,
                            );
                            let end = self.skip_right_brace();

                            DeclareBody::Braced(DeclareBodyBraced {
                                id: self.state.id(),
                                span: Span::combine(start, end),
                                left_brace: start,
                                statements,
                                right_brace: end,
                            })
                        }
                        TokenKind::Colon => {
                            let start = self.skip_colon();
                            let statements = self.parse_multiple_statements_until(
                                &TokenKind::EndDeclare,
                            );
                            let enddeclare = self.skip(TokenKind::EndDeclare);
                            let semicolon = self.skip_semicolon();

                            DeclareBody::Block(DeclareBodyBlock {
                                id: self.state.id(),
                                span: Span::combine(start, semicolon),
                                colon: start,
                                statements,
                                enddeclare,
                                semicolon,
                            })
                        }
                        _ => {
                            let expression = self.create();
                            let end = self.skip_semicolon();
                            let span = Span::combine(expression.span(), end.span());

                            DeclareBody::Expression(DeclareBodyExpression {
                                id: self.state.id(),
                                span,
                                expression,
                                semicolon: end,
                            })
                        }
                    };

                    let span = Span::combine(declare, body.span());

                    StatementKind::Declare(DeclareStatement {
                        id: self.state.id(),
                        span,
                        declare,
                        entries,
                        body,
                    })
                }
                TokenKind::Global => {
                    let global = current.span;
                    self.next();

                    let mut variables = vec![];
                    // `loop` instead of `while` as we don't allow for extra commas.
                    loop {
                        variables.push(self.parse_dynamic_variable());

                        if self.current_kind() == TokenKind::Comma {
                            self.next();
                        } else {
                            break;
                        }
                    }

                    let semicolon = self.skip_semicolon();
                    let span = Span::combine(global, semicolon);

                    StatementKind::Global(GlobalStatement {
                        id: self.state.id(),
                        span,
                        global,
                        variables,
                        semicolon,
                    })
                }
                TokenKind::Static if matches!(peek.kind, TokenKind::Variable) => {
                    self.next();

                    let mut vars = vec![];

                    // `loop` instead of `while` as we don't allow for extra commas.
                    loop {
                        let var = self.parse_simple_variable();
                        let mut default = None;

                        if self.current_kind() == TokenKind::Equals {
                            self.next();

                            default = Some(self.create());
                        }

                        let span = if let Some(default) = &default {
                            Span::combine(var.span, default.span)
                        } else {
                            var.span
                        };

                        vars.push(StaticVar {
                            id: self.state.id(),
                            span,
                            var: Variable::SimpleVariable(var),
                            default,
                        });

                        if self.current_kind() == TokenKind::Comma {
                            self.next();
                        } else {
                            break;
                        }
                    }

                    let semicolon = self.skip_semicolon();
                    let span = Span::combine(current.span, semicolon);

                    StatementKind::Static(StaticStatement {
                        id: self.state.id(),
                        span,
                        vars,
                        semicolon,
                    })
                }
                TokenKind::InlineHtml => {
                    let html = self.current().to_owned();
                    self.next();

                    StatementKind::InlineHtml(InlineHtmlStatement {
                        id: self.state.id(),
                        span: html.span,
                        html,
                    })
                }
                TokenKind::Do => self.parse_do_while_statement(),
                TokenKind::While => self.parse_while_statement(),
                TokenKind::For => self.parse_for_statement(),
                TokenKind::Foreach => self.parse_foreach_statement(),
                TokenKind::Continue => self.parse_continue_statement(),
                TokenKind::Break => self.parse_break_statement(),
                TokenKind::Switch => self.parse_switch_statement(),
                TokenKind::If => self.parse_if_statement(),
                TokenKind::Try => self.parse_try_block(),
                TokenKind::LeftBrace => self.parse_block_statement(),
                TokenKind::SemiColon => {
                    let start = current.span;

                    self.next();

                    StatementKind::Noop(start)
                }
                TokenKind::Echo => {
                    let echo = current.span;
                    self.next();

                    let mut values = Vec::new();
                    // FIXME: We should check for a semi-colon here and produce a better error,
                    //        currently the error says that the semi-colon is unexpected (which it is)
                    //        but a more appropriate error would be that the expression is missing and
                    //        that the semi-colon is fine where it is (or at least not the real problem).
                    loop {
                        values.push(self.create());

                        if self.current_kind() == TokenKind::Comma {
                            self.next();
                        } else {
                            break;
                        }
                    }

                    let ending = self.skip_ending();
                    let end = ending.span();

                    StatementKind::Echo(EchoStatement {
                        id: self.state.id(),
                        span: Span::combine(echo, end),
                        echo,
                        values,
                        ending,
                    })
                }
                TokenKind::Return => {
                    let r#return = current.span;

                    self.next();

                    let value = if matches!(
                        self.current_kind(),
                        TokenKind::SemiColon | TokenKind::CloseTag
                    ) {
                        None
                    } else {
                        Some(self.create())
                    };

                    let ending = self.skip_ending();
                    let end = ending.span();

                    StatementKind::Return(ReturnStatement {
                        id: self.state.id(),
                        span: Span::combine(r#return, end),
                        r#return,
                        value,
                        ending,
                    })
                }
                _ => {
                    let expression = self.create();
                    let ending = self.skip_ending();

                    StatementKind::Expression(ExpressionStatement {
                        id: self.state.id(),
                        span: Span::combine(expression.span, ending.span()),
                        expression,
                        ending,
                    })
                }
            }
        };

        let span = statement.span();

        Statement::new(self.state.id(), statement, span, comments)
    }
}
