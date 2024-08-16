use crate::state::State;
use crate::ParserDiagnostic;
use pxp_ast::*;

use pxp_diagnostics::Severity;
use pxp_span::{Span, Spanned};
use pxp_token::TokenKind;

#[inline(always)]
pub fn class_group(state: &mut State, input: Vec<(Span, TokenKind)>) -> ClassModifierGroup {
    let modifiers = input
        .iter()
        .filter_map(|(span, token)| match token {
            TokenKind::Readonly => Some(ClassModifier::Readonly(*span)),
            TokenKind::Final => Some(ClassModifier::Final(*span)),
            TokenKind::Abstract => Some(ClassModifier::Abstract(*span)),
            _ => {
                state.diagnostic(
                    ParserDiagnostic::InvalidClassModifier,
                    Severity::Error,
                    *span,
                );

                None
            }
        })
        .collect::<Vec<ClassModifier>>();

    let group = ClassModifierGroup {
        id: state.id(),
        span: modifiers.span(),
        modifiers,
    };

    if group.has_abstract() && group.has_final() {
        let start = input.first().unwrap().0;
        let end = input.last().unwrap().0;
        let span = Span::new(start.start, end.end);

        state.diagnostic(
            ParserDiagnostic::CannotUseFinalWithAbstract,
            Severity::Error,
            span,
        );
    }

    group
}

#[inline(always)]
pub fn method_group(state: &mut State, input: Vec<(Span, TokenKind)>) -> MethodModifierGroup {
    let modifiers = input
        .iter()
        .filter_map(|(span, token)| match token {
            TokenKind::Final => Some(MethodModifier::Final(*span)),
            TokenKind::Abstract => Some(MethodModifier::Abstract(*span)),
            TokenKind::Private => Some(MethodModifier::Private(*span)),
            TokenKind::Protected => Some(MethodModifier::Protected(*span)),
            TokenKind::Public => Some(MethodModifier::Public(*span)),
            TokenKind::Static => Some(MethodModifier::Static(*span)),
            _ => {
                state.diagnostic(
                    ParserDiagnostic::InvalidMethodModifier,
                    Severity::Error,
                    *span,
                );

                None
            }
        })
        .collect::<Vec<MethodModifier>>();

    let group = MethodModifierGroup {
        id: state.id(),
        span: modifiers.span(),
        modifiers,
    };

    if group.has_abstract() && group.has_final() {
        let start = input.first().unwrap().0;
        let end = input.last().unwrap().0;
        let span = Span::new(start.start, end.end);

        state.diagnostic(
            ParserDiagnostic::CannotUseFinalWithAbstract,
            Severity::Error,
            span,
        );
    }

    group
}

#[inline(always)]
pub fn property_group(state: &mut State, input: Vec<(Span, TokenKind)>) -> PropertyModifierGroup {
    let modifiers = input
        .iter()
        .filter_map(|(span, token)| match token {
            TokenKind::Readonly => Some(PropertyModifier::Readonly(*span)),
            TokenKind::Static => Some(PropertyModifier::Static(*span)),
            TokenKind::Public => Some(PropertyModifier::Public(*span)),
            TokenKind::Protected => Some(PropertyModifier::Protected(*span)),
            TokenKind::Private => Some(PropertyModifier::Private(*span)),
            _ => {
                state.diagnostic(
                    ParserDiagnostic::InvalidPropertyModifier,
                    Severity::Error,
                    *span,
                );

                None
            }
        })
        .collect::<Vec<PropertyModifier>>();

    PropertyModifierGroup {
        id: state.id(),
        span: modifiers.span(),
        modifiers,
    }
}

#[inline(always)]
pub fn promoted_property_group(
    state: &mut State,
    input: Vec<(Span, TokenKind)>,
) -> PromotedPropertyModifierGroup {
    let modifiers = input
        .iter()
        .filter_map(|(span, token)| match token {
            TokenKind::Readonly => Some(PromotedPropertyModifier::Readonly(*span)),
            TokenKind::Private => Some(PromotedPropertyModifier::Private(*span)),
            TokenKind::Protected => Some(PromotedPropertyModifier::Protected(*span)),
            TokenKind::Public => Some(PromotedPropertyModifier::Public(*span)),
            _ => {
                state.diagnostic(
                    ParserDiagnostic::InvalidPropertyModifier,
                    Severity::Error,
                    *span,
                );

                None
            }
        })
        .collect::<Vec<PromotedPropertyModifier>>();

    PromotedPropertyModifierGroup {
        id: state.id(),
        span: modifiers.span(),
        modifiers,
    }
}

pub fn constant_group(state: &mut State, input: Vec<(Span, TokenKind)>) -> ConstantModifierGroup {
    let modifiers = input
        .iter()
        .filter_map(|(span, token)| match token {
            TokenKind::Protected => Some(ConstantModifier::Protected(*span)),
            TokenKind::Public => Some(ConstantModifier::Public(*span)),
            TokenKind::Private => Some(ConstantModifier::Private(*span)),
            TokenKind::Final => Some(ConstantModifier::Final(*span)),
            _ => {
                state.diagnostic(
                    ParserDiagnostic::InvalidConstantModifier,
                    Severity::Error,
                    *span,
                );

                None
            }
        })
        .collect::<Vec<ConstantModifier>>();

    let group = ConstantModifierGroup {
        id: state.id(),
        span: modifiers.span(),
        modifiers,
    };

    if group.has_final() && group.has_private() {
        let start = input.first().unwrap().0;
        let end = input.last().unwrap().0;
        let span = Span::new(start.start, end.end);

        state.diagnostic(
            ParserDiagnostic::CannotUseFinalWithPrivateOnConstant,
            Severity::Error,
            span,
        );
    }

    group
}

pub fn collect(state: &mut State) -> Vec<(Span, TokenKind)> {
    let mut collected: Vec<(Span, TokenKind)> = vec![];

    let collectable_tokens = [
        TokenKind::Private,
        TokenKind::Protected,
        TokenKind::Public,
        TokenKind::Final,
        TokenKind::Abstract,
        TokenKind::Static,
        TokenKind::Readonly,
    ];

    let mut current = state.stream.current().clone();
    let mut current_kind = current.kind;
    let mut current_span = current.span;

    while collectable_tokens.contains(&current_kind) {
        if let Some((span, _)) = collected.iter().find(|(_, kind)| kind == &current_kind) {
            state.diagnostic(ParserDiagnostic::DuplicateModifier, Severity::Error, *span);
        }

        // guard against multiple visibility modifiers, we don't care where these modifiers are used.
        if matches!(
            current_kind,
            TokenKind::Public | TokenKind::Protected | TokenKind::Private
        ) {
            if let Some((span, _)) = collected.iter().find(|(_, kind)| {
                matches!(
                    kind,
                    TokenKind::Public | TokenKind::Protected | TokenKind::Private
                ) && kind != &current_kind
            }) {
                state.diagnostic(
                    ParserDiagnostic::MultipleVisibilityModifiers,
                    Severity::Error,
                    *span,
                );
            }
        }

        collected.push((current_span, current_kind));

        state.stream.next();

        current = state.stream.current().clone();
        current_kind = current.kind;
        current_span = current.span;
    }

    collected
}
