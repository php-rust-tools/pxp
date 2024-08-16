use pxp_span::Span;

use crate::{NodeId, NodeKind};

#[derive(Debug, PartialEq, Clone)]
pub struct Node<'a> {
    pub id: NodeId,
    pub kind: NodeKind<'a>,
    pub span: Span,
}
