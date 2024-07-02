use crate::expressions;
use crate::internal::attributes;
use crate::internal::data_type;
use crate::internal::identifiers;
use crate::internal::modifiers;
use crate::internal::utils;
use crate::internal::variables;
use crate::state::State;
use crate::ParserDiagnostic;
use pxp_ast::*;

use pxp_diagnostics::Severity;
use pxp_span::Span;
use pxp_span::Spanned;
use pxp_token::TokenKind;

pub fn function_parameter_list(state: &mut State) -> FunctionParameterList {
    let comments = state.stream.comments();
    let left_parenthesis = utils::skip_left_parenthesis(state);
    let parameters = utils::comma_separated(
        state,
        &|state| {
            attributes::gather_attributes(state);

            let ty = data_type::optional_data_type(state);

            let mut current = state.stream.current();
            let ampersand = if current.kind == TokenKind::Ampersand {
                state.stream.next();
                current = state.stream.current();
                Some(current.span)
            } else {
                None
            };

            let ellipsis = if current.kind == TokenKind::Ellipsis {
                state.stream.next();

                Some(current.span)
            } else {
                None
            };

            // 2. Then expect a variable.
            let var = variables::simple_variable(state);

            let mut default = None;
            if state.stream.current().kind == TokenKind::Equals {
                state.stream.next();
                default = Some(expressions::create(state));
            }

            FunctionParameter {
                span: if ty.is_some() {
                    Span::combine(ty.span(), var.span)
                } else {
                    var.span
                },
                comments: state.stream.comments(),
                name: var,
                attributes: state.get_attributes(),
                data_type: ty,
                ellipsis,
                default,
                ampersand,
            }
        },
        TokenKind::RightParen,
    );

    let right_parenthesis = utils::skip_right_parenthesis(state);

    FunctionParameterList {
        span: Span::combine(left_parenthesis, right_parenthesis),
        comments,
        left_parenthesis,
        parameters,
        right_parenthesis,
    }
}

pub fn constructor_parameter_list(state: &mut State) -> ConstructorParameterList {
    let comments = state.stream.comments();

    let left_parenthesis = utils::skip_left_parenthesis(state);
    let parameters = utils::comma_separated::<ConstructorParameter>(
        state,
        &|state| {
            attributes::gather_attributes(state);

            let modifiers = modifiers::collect(state);
            let modifiers = modifiers::promoted_property_group(state, modifiers);

            let ty = data_type::optional_data_type(state);

            let mut current = state.stream.current();
            let ampersand = if matches!(current.kind, TokenKind::Ampersand) {
                state.stream.next();

                current = state.stream.current();

                Some(current.span)
            } else {
                None
            };

            let (ellipsis, var) = if matches!(current.kind, TokenKind::Ellipsis) {
                state.stream.next();
                let var = variables::simple_variable(state);
                if !modifiers.is_empty() {
                    state.diagnostic(
                        ParserDiagnostic::PromotedPropertyCannotBeVariadic,
                        Severity::Error,
                        current.span,
                    );
                }

                (Some(current.span), var)
            } else {
                (None, variables::simple_variable(state))
            };

            // 2. Then expect a variable.

            if !modifiers.is_empty() {
                match &ty {
                    Some(ty) => {
                        if ty.includes_callable() || ty.is_bottom() {
                            state.diagnostic(
                                ParserDiagnostic::ForbiddenTypeUsedInProperty,
                                Severity::Error,
                                ty.get_span(),
                            );
                        }
                    }
                    None => {
                        if let Some(modifier) = modifiers.get_readonly() {
                            state.diagnostic(
                                ParserDiagnostic::ReadonlyPropertyMustHaveType,
                                Severity::Error,
                                modifier.span(),
                            );
                        }
                    }
                }
            }

            let mut default = None;
            if state.stream.current().kind == TokenKind::Equals {
                state.stream.next();
                default = Some(expressions::create(state));
            }

            ConstructorParameter {
                span: if ty.is_some() {
                    Span::combine(ty.span(), var.span)
                } else {
                    var.span
                },
                comments: state.stream.comments(),
                name: var,
                attributes: state.get_attributes(),
                data_type: ty,
                ellipsis,
                default,
                modifiers,
                ampersand,
            }
        },
        TokenKind::RightParen,
    );

    let right_parenthesis = utils::skip_right_parenthesis(state);

    ConstructorParameterList {
        span: Span::combine(left_parenthesis, right_parenthesis),
        comments,
        left_parenthesis,
        parameters,
        right_parenthesis,
    }
}

pub fn argument_list(state: &mut State) -> ArgumentList {
    let comments = state.stream.comments();
    let start = utils::skip_left_parenthesis(state);

    let mut arguments = Vec::new();
    let mut has_used_named_arguments = false;

    while !state.stream.is_eof() && state.stream.current().kind != TokenKind::RightParen {
        let span = state.stream.current().span;
        let (named, argument) = argument(state);
        if named {
            has_used_named_arguments = true;
        } else if has_used_named_arguments {
            state.diagnostic(
                ParserDiagnostic::CannotUsePositionalArgumentAfterNamedArgument,
                Severity::Error,
                span,
            );
        }

        arguments.push(argument);

        if state.stream.current().kind == TokenKind::Comma {
            state.stream.next();
        } else {
            break;
        }
    }

    let end = utils::skip_right_parenthesis(state);

    ArgumentList {
        span: Span::combine(start, end),
        comments,
        left_parenthesis: start,
        right_parenthesis: end,
        arguments,
    }
}

pub fn single_argument(
    state: &mut State,
    required: bool,
    only_positional: bool,
) -> Option<SingleArgument> {
    let comments = state.stream.comments();

    if state.stream.current().kind != TokenKind::LeftParen {
        return None;
    }

    let start = utils::skip_left_parenthesis(state);

    let mut first_argument = None;

    while !state.stream.is_eof() && state.stream.current().kind != TokenKind::RightParen {
        let span = state.stream.current().span;
        let (named, argument) = argument(state);
        if only_positional && named {
            state.diagnostic(
                ParserDiagnostic::PositionalArgumentsOnly,
                Severity::Error,
                span,
            );
        }

        if first_argument.is_some() {
            state.diagnostic(
                ParserDiagnostic::OnlyAllowedOneArgument,
                Severity::Error,
                span,
            );
        }

        first_argument = Some(argument);

        if state.stream.current().kind == TokenKind::Comma {
            state.stream.next();
        } else {
            break;
        }
    }

    if required && first_argument.is_none() {
        state.diagnostic(
            ParserDiagnostic::ArgumentRequired,
            Severity::Error,
            state.stream.current().span,
        );
    }

    let end = utils::skip_right_parenthesis(state);

    Some(SingleArgument {
        span: Span::combine(start, end),
        comments,
        left_parenthesis: start,
        right_parenthesis: end,
        argument: first_argument,
    })
}

fn argument(state: &mut State) -> (bool, Argument) {
    if identifiers::is_identifier_maybe_reserved(&state.stream.current().kind)
        && state.stream.peek().kind == TokenKind::Colon
    {
        let name = identifiers::identifier_maybe_reserved(state);
        let colon = utils::skip(state, TokenKind::Colon);
        let ellipsis = if state.stream.current().kind == TokenKind::Ellipsis {
            Some(utils::skip(state, TokenKind::Ellipsis))
        } else {
            None
        };
        let value = expressions::create(state);

        return (
            true,
            Argument::Named(NamedArgument {
                span: Span::combine(name.span, value.span),
                comments: state.stream.comments(),
                name,
                colon,
                ellipsis,
                value,
            }),
        );
    }

    let ellipsis = if state.stream.current().kind == TokenKind::Ellipsis {
        Some(utils::skip(state, TokenKind::Ellipsis))
    } else {
        None
    };

    let value = expressions::create(state);

    (
        false,
        Argument::Positional(PositionalArgument {
            span: value.span,
            comments: state.stream.comments(),
            ellipsis,
            value,
        }),
    )
}
