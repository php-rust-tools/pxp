use std::collections::VecDeque;

use pxp_ast::identifiers::SimpleIdentifier;
use pxp_ast::{attributes::AttributeGroup, NodeId};
use pxp_diagnostics::{Diagnostic, Severity};
use pxp_lexer::stream::TokenStream;
use pxp_span::Span;
use pxp_symbol::SymbolTable;

use crate::ParserDiagnostic;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NamespaceType {
    Braced,
    Unbraced,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Scope {
    Namespace(SimpleIdentifier),
    BracedNamespace(Option<SimpleIdentifier>),
}

#[derive(Debug)]
pub struct State<'a, 'b> {
    pub stack: VecDeque<Scope>,
    pub stream: &'a mut TokenStream<'a>,
    pub symbol_table: &'b mut SymbolTable,
    pub attributes: Vec<AttributeGroup>,
    pub namespace_type: Option<NamespaceType>,
    pub diagnostics: Vec<Diagnostic<ParserDiagnostic>>,
    pub id: NodeId,
}

impl<'a, 'b> State<'a, 'b> {
    pub fn new(tokens: &'a mut TokenStream<'a>, symbol_table: &'b mut SymbolTable) -> Self {
        Self {
            stack: VecDeque::with_capacity(32),
            stream: tokens,
            symbol_table,
            namespace_type: None,
            attributes: vec![],
            diagnostics: vec![],
            // Node IDs start from 1 to reserve 0 for invalid / missing / unknown nodes.
            id: 1,
        }
    }

    #[inline(always)]
    pub fn id(&mut self) -> usize {
        self.id += 1;
        self.id
    }

    pub fn attribute(&mut self, attr: AttributeGroup) {
        self.attributes.push(attr);
    }

    pub fn get_attributes(&mut self) -> Vec<AttributeGroup> {
        let mut attributes = vec![];

        std::mem::swap(&mut self.attributes, &mut attributes);

        attributes
    }

    /// Return the namespace type used in the current state
    ///
    /// The namespace type is retrieve from the last entered
    /// namespace scope.
    ///
    /// Note: even when a namespace scope is exited, the namespace type
    /// is retained, until the next namespace scope is entered.
    pub fn namespace_type(&self) -> Option<&NamespaceType> {
        self.namespace_type.as_ref()
    }

    pub fn namespace(&self) -> Option<&Scope> {
        self.stack.iter().next()
    }

    pub fn previous_scope(&self) -> Option<&Scope> {
        self.stack.get(self.stack.len() - 2)
    }

    pub fn diagnostic(&mut self, kind: ParserDiagnostic, severity: Severity, span: Span) {
        self.diagnostics.push(Diagnostic::new(kind, severity, span));
    }

    pub fn enter(&mut self, scope: Scope) {
        match &scope {
            Scope::Namespace(_) => {
                self.namespace_type = Some(NamespaceType::Unbraced);
            }
            Scope::BracedNamespace(_) => {
                self.namespace_type = Some(NamespaceType::Braced);
            }
        }

        self.stack.push_back(scope);
    }

    pub fn exit(&mut self) {
        self.stack.pop_back();
    }
}
