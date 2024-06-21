use pxp_span::{Span, Spanned};

use crate::generated::*;

impl Spanned for Statement {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for StatementKind {
    fn span(&self) -> Span {
        match self {
            StatementKind::FullOpeningTag(node) => node.span(),
            StatementKind::ShortOpeningTag(node) => node.span(),
            StatementKind::EchoOpeningTag(node) => node.span(),
            StatementKind::ClosingTag(node) => node.span(),
            StatementKind::InlineHtml(node) => node.span(),
            StatementKind::Label(node) => node.span(),
            StatementKind::Goto(node) => node.span(),
            StatementKind::HaltCompiler(node) => node.span(),
            StatementKind::Static(node) => node.span(),
            StatementKind::DoWhile(node) => node.span(),
            StatementKind::While(node) => node.span(),
            StatementKind::For(node) => node.span(),
            StatementKind::Foreach(node) => node.span(),
            StatementKind::Break(node) => node.span(),
            StatementKind::Continue(node) => node.span(),
            StatementKind::Constant(node) => node.span(),
            StatementKind::Function(node) => node.span(),
            StatementKind::Class(node) => node.span(),
            StatementKind::Trait(node) => node.span(),
            StatementKind::Interface(node) => node.span(),
            StatementKind::If(node) => node.span(),
            StatementKind::Switch(node) => node.span(),
            StatementKind::Echo(node) => node.span(),
            StatementKind::Expression(node) => node.span(),
            StatementKind::Return(node) => node.span(),
            StatementKind::Namespace(node) => node.span(),
            StatementKind::Use(node) => node.span(),
            StatementKind::GroupUse(node) => node.span(),
            StatementKind::Comment(node) => node.span(),
            StatementKind::Try(node) => node.span(),
            StatementKind::UnitEnum(node) => node.span(),
            StatementKind::BackedEnum(node) => node.span(),
            StatementKind::Block(node) => node.span(),
            StatementKind::Global(node) => node.span(),
            StatementKind::Declare(node) => node.span(),
            StatementKind::Noop(node) => node.span(),
        }
    }
}

impl Spanned for Expression {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for ExpressionKind {
    fn span(&self) -> Span {
        match self {
            ExpressionKind::Missing => Span::default(),
            ExpressionKind::Eval(node) => node.span(),
            ExpressionKind::Empty(node) => node.span(),
            ExpressionKind::Die(node) => node.span(),
            ExpressionKind::Exit(node) => node.span(),
            ExpressionKind::Isset(node) => node.span(),
            ExpressionKind::Unset(node) => node.span(),
            ExpressionKind::Print(node) => node.span(),
            ExpressionKind::Literal(node) => node.span(),
            ExpressionKind::ArithmeticOperation(node) => node.span(),
            ExpressionKind::AssignmentOperation(node) => node.span(),
            ExpressionKind::BitwiseOperation(node) => node.span(),
            ExpressionKind::ComparisonOperation(node) => node.span(),
            ExpressionKind::LogicalOperation(node) => node.span(),
            ExpressionKind::Concat(node) => node.span(),
            ExpressionKind::Instanceof(node) => node.span(),
            ExpressionKind::Reference(node) => node.span(),
            ExpressionKind::Parenthesized(node) => node.span(),
            ExpressionKind::ErrorSuppress(node) => node.span(),
            ExpressionKind::Identifier(node) => node.span(),
            ExpressionKind::Variable(node) => node.span(),
            ExpressionKind::Include(node) => node.span(),
            ExpressionKind::IncludeOnce(node) => node.span(),
            ExpressionKind::Require(node) => node.span(),
            ExpressionKind::RequireOnce(node) => node.span(),
            ExpressionKind::FunctionCall(node) => node.span(),
            ExpressionKind::FunctionClosureCreation(node) => node.span(),
            ExpressionKind::MethodCall(node) => node.span(),
            ExpressionKind::MethodClosureCreation(node) => node.span(),
            ExpressionKind::NullsafeMethodCall(node) => node.span(),
            ExpressionKind::StaticMethodCall(node) => node.span(),
            ExpressionKind::StaticVariableMethodCall(node) => node.span(),
            ExpressionKind::StaticMethodClosureCreation(node) => node.span(),
            ExpressionKind::StaticVariableMethodClosureCreation(node) => node.span(),
            ExpressionKind::PropertyFetch(node) => node.span(),
            ExpressionKind::NullsafePropertyFetch(node) => node.span(),
            ExpressionKind::StaticPropertyFetch(node) => node.span(),
            ExpressionKind::ConstantFetch(node) => node.span(),
            ExpressionKind::Static => Span::default(),
            ExpressionKind::Self_ => Span::default(),
            ExpressionKind::Parent => Span::default(),
            ExpressionKind::ShortArray(node) => node.span(),
            ExpressionKind::Array(node) => node.span(),
            ExpressionKind::List(node) => node.span(),
            ExpressionKind::Closure(node) => node.span(),
            ExpressionKind::ArrowFunction(node) => node.span(),
            ExpressionKind::New(node) => node.span(),
            ExpressionKind::InterpolatedString(node) => node.span(),
            ExpressionKind::Heredoc(node) => node.span(),
            ExpressionKind::Nowdoc(node) => node.span(),
            ExpressionKind::ShellExec(node) => node.span(),
            ExpressionKind::AnonymousClass(node) => node.span(),
            ExpressionKind::Bool(node) => node.span(),
            ExpressionKind::ArrayIndex(node) => node.span(),
            ExpressionKind::Null => Span::default(),
            ExpressionKind::MagicConstant(node) => node.span(),
            ExpressionKind::ShortTernary(node) => node.span(),
            ExpressionKind::Ternary(node) => node.span(),
            ExpressionKind::Coalesce(node) => node.span(),
            ExpressionKind::Clone(node) => node.span(),
            ExpressionKind::Match(node) => node.span(),
            ExpressionKind::Throw(node) => node.span(),
            ExpressionKind::Yield(node) => node.span(),
            ExpressionKind::YieldFrom(node) => node.span(),
            ExpressionKind::Cast(node) => node.span(),
            ExpressionKind::Name(node) => node.span(),
            ExpressionKind::Noop => Span::default(),
        }
    }
}

impl Spanned for InlineHtmlStatement {
    fn span(&self) -> Span {
        self.html.span()
    }
}

impl Spanned for FullOpeningTagStatement {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for ShortOpeningTagStatement {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for EchoOpeningTagStatement {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for ClosingTagStatement {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for ExpressionStatement {
    fn span(&self) -> Span {
        Span::new(self.expression.span().start, self.ending.span().end)
    }
}