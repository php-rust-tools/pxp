use crate::{SimpleVariable, Variable};
use pxp_span::{Span, Spanned};
use pxp_symbol::Symbol;

impl SimpleVariable {
    pub fn missing(span: Span) -> Self {
        Self {
            symbol: Symbol::missing(),
            stripped: Symbol::missing(),
            span,
        }
    }
}

impl Spanned for Variable {
    fn span(&self) -> Span {
        match self {
            Self::SimpleVariable(simple) => simple.span,
            Self::VariableVariable(dynamic) => dynamic.span,
            Self::BracedVariableVariable(dynamic) => dynamic.span,
        }
    }
}
