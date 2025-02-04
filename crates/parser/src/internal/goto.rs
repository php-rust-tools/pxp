use crate::Parser;
use pxp_ast::StatementKind;
use pxp_ast::*;
use pxp_span::Span;
use pxp_token::TokenKind;

impl<'a> Parser<'a> {
    pub fn parse_label_statement(&mut self) -> StatementKind {
        let comments = self.comments();
        let label = self.parse_label_identifier();
        let colon = self.skip_colon();

        StatementKind::Label(Box::new(LabelStatement {
            id: self.id(),
            span: Span::combine(label.span, colon),
            comments,
            label,
            colon,
        }))
    }

    pub fn parse_goto_statement(&mut self) -> StatementKind {
        let comments = self.comments();
        let keyword = self.skip(TokenKind::Goto);
        let label = self.parse_label_identifier();
        let semicolon = self.skip_semicolon();

        StatementKind::Goto(Box::new(GotoStatement {
            id: self.id(),
            span: Span::combine(keyword, semicolon),
            comments,
            keyword,
            label,
            semicolon,
        }))
    }
}
