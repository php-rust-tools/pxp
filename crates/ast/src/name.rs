use std::fmt::Display;

use pxp_bytestring::ByteString;
use pxp_span::Span;
use pxp_token::TokenKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Copy, Hash, Eq, Serialize, Deserialize)]
pub enum NameQualification {
    Unqualified,
    Qualified,
    FullyQualified,
}

impl From<TokenKind> for NameQualification {
    fn from(kind: TokenKind) -> Self {
        match kind {
            TokenKind::QualifiedIdentifier => NameQualification::Qualified,
            TokenKind::FullyQualifiedIdentifier => NameQualification::FullyQualified,
            _ => NameQualification::Unqualified,
        }
    }
}

use crate::{Name, NameKind, NodeId, ResolvedName, SpecialName, SpecialNameKind, UnresolvedName};

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            NameKind::Special(s) => write!(f, "{}", s.symbol),
            NameKind::Unresolved(u) => write!(f, "{}", u.symbol),
            NameKind::Resolved(r) => write!(f, "{}", r.resolved),
        }
    }
}

impl Name {
    pub fn new(id: NodeId, kind: NameKind, span: Span) -> Self {
        Self { id, kind, span }
    }

    pub fn missing(id: NodeId, span: Span) -> Self {
        Self::new(
            id,
            NameKind::Resolved(ResolvedName {
                resolved: ByteString::empty(),
                original: ByteString::empty(),
            }),
            span,
        )
    }

    pub fn resolved(id: NodeId, symbol: ByteString, original: ByteString, span: Span) -> Self {
        Self::new(
            id,
            NameKind::Resolved(ResolvedName {
                resolved: symbol,
                original,
            }),
            span,
        )
    }

    pub fn unresolved(
        id: NodeId,
        symbol: ByteString,
        qualification: NameQualification,
        span: Span,
    ) -> Self {
        Self::new(
            id,
            NameKind::Unresolved(UnresolvedName {
                symbol,
                qualification,
            }),
            span,
        )
    }

    pub fn special(id: NodeId, kind: SpecialNameKind, symbol: ByteString, span: Span) -> Self {
        Self::new(id, NameKind::Special(SpecialName { kind, symbol }), span)
    }

    pub fn symbol(&self) -> &ByteString {
        match &self.kind {
            NameKind::Special(s) => &s.symbol,
            NameKind::Unresolved(u) => &u.symbol,
            NameKind::Resolved(r) => &r.resolved,
        }
    }

    pub fn is_special(&self) -> bool {
        matches!(self.kind, NameKind::Special(_))
    }

    pub fn is_unresolved(&self) -> bool {
        matches!(self.kind, NameKind::Unresolved(_))
    }

    pub fn is_resolved(&self) -> bool {
        matches!(self.kind, NameKind::Resolved(_))
    }

    pub fn to_resolved(&self) -> &ResolvedName {
        self.as_resolved().unwrap()
    }

    pub fn to_unresolved(&self) -> &UnresolvedName {
        self.as_unresolved().unwrap()
    }

    pub fn to_special(&self) -> &SpecialName {
        self.as_special().unwrap()
    }

    pub fn as_resolved(&self) -> Option<&ResolvedName> {
        match &self.kind {
            NameKind::Resolved(r) => Some(r),
            _ => None,
        }
    }

    pub fn as_unresolved(&self) -> Option<&UnresolvedName> {
        match &self.kind {
            NameKind::Unresolved(u) => Some(u),
            _ => None,
        }
    }

    pub fn as_special(&self) -> Option<&SpecialName> {
        match &self.kind {
            NameKind::Special(s) => Some(s),
            _ => None,
        }
    }
}

impl Display for SpecialName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            SpecialNameKind::Self_ => write!(f, "self"),
            SpecialNameKind::Static => write!(f, "static"),
            SpecialNameKind::Parent => write!(f, "parent"),
        }
    }
}

impl Display for UnresolvedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.symbol)
    }
}

impl Display for ResolvedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.original)
    }
}
