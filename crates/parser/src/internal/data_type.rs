use crate::Parser;
use crate::ParserDiagnostic;
use pxp_ast::*;
use pxp_diagnostics::Severity;
use pxp_span::Span;
use pxp_token::TokenKind;
use pxp_type::Type;

impl<'a> Parser<'a> {
    pub fn parse_data_type(&mut self) -> DataType {
        let kind = if self.state.is_in_docblock() {
            self.parse_docblock_type()
        } else if self.current_kind() == TokenKind::Question {
            self.parse_nullable_type()
        } else if self.current_kind() == TokenKind::LeftParen {
            self.parse_dnf_type()
        } else {
            let ty = self.parse_simple_data_type();

            if self.current_kind() == TokenKind::Pipe {
                self.parse_union_type(ty, false)
            } else if self.current_kind() == TokenKind::Ampersand
                && !matches!(
                    self.peek_kind(),
                    TokenKind::Variable | TokenKind::Ellipsis | TokenKind::Ampersand
                )
            {
                self.parse_intersection_type(ty, false)
            } else {
                ty
            }
        };

        // FIXME: We need to create spans for types, but we don't have access to the previous token anymore.
        let span = Span::missing();

        DataType::new(self.state.id(), kind, span)
    }

    pub fn parse_optional_data_type(&mut self) -> Option<DataType> {
        let kind = if self.state.is_in_docblock() {
            self.parse_docblock_type()
        } else if self.current_kind() == TokenKind::Question {
            self.parse_nullable_type()
        } else if self.current_kind() == TokenKind::LeftParen {
            self.parse_dnf_type()
        } else {
            let ty = self.parse_optional_simple_data_type();

            match ty {
                Some(ty) => {
                    if self.current_kind() == TokenKind::Pipe {
                        self.parse_union_type(ty, false)
                    } else if self.current_kind() == TokenKind::Ampersand
                        && !matches!(
                            self.peek_kind(),
                            TokenKind::Variable | TokenKind::Ellipsis | TokenKind::Ampersand
                        )
                    {
                        self.parse_intersection_type(ty, false)
                    } else {
                        ty
                    }
                }
                None => return None,
            }
        };

        // FIXME: We need to create spans for types, but we don't have access to the previous token anymore.
        let span = Span::missing();

        Some(DataType::new(self.state.id(), kind, span))
    }

    // Special type parsing logic for DocBlock comments, heavily based on the phpstan/phpdoc-parser package.
    fn parse_docblock_type(&mut self) -> Type<Name> {
        match self.current_kind() {
            TokenKind::Question => self.parse_docblock_nullable(),
            _ => {
                let r#type = self.parse_docblock_atomic();

                if r#type == Type::Missing {
                    return Type::Missing;
                }

                match self.current_kind() {
                    TokenKind::Pipe => self.parse_docblock_union(r#type),
                    TokenKind::Ampersand => self.parse_docblock_intersection(r#type),
                    _ => r#type,
                }
            }
        }
    }

    fn parse_docblock_nullable(&mut self) -> Type<Name> {
        self.next();

        let inner = self.parse_docblock_atomic();

        if inner == Type::Missing {
            return Type::Missing;
        }

        Type::Nullable(Box::new(inner))
    }

    fn parse_docblock_union(&mut self, lhs: Type<Name>) -> Type<Name> {
        let mut types = vec![lhs];

        while let TokenKind::Pipe = self.current_kind() {
            self.next();

            // FIXME: Warn about invalid types inside of union.
            types.push(self.parse_docblock_atomic());
        }

        Type::Union(types)
    }

    fn parse_docblock_subparse_union(&mut self, lhs: Type<Name>) -> Type<Name> {
        let mut types = vec![lhs];

        while let TokenKind::Pipe = self.current_kind() {
            self.next();

            self.skip_doc_eol();
            // FIXME: Warn about invalid types inside of union.
            types.push(self.parse_docblock_atomic());
            self.skip_doc_eol();
        }

        Type::Union(types)
    }

    fn parse_docblock_intersection(&mut self, lhs: Type<Name>) -> Type<Name> {
        let mut types = vec![lhs];

        while let TokenKind::Ampersand = self.current_kind() {
            self.next();

            // FIXME: Warn about invalid types inside of intersection.
            types.push(self.parse_docblock_atomic());
        }

        Type::Intersection(types)
    }

    fn parse_docblock_subparse_intersection(&mut self, lhs: Type<Name>) -> Type<Name> {
        let mut types = vec![lhs];

        while let TokenKind::Ampersand = self.current_kind() {
            self.next();

            self.skip_doc_eol();
            // FIXME: Warn about invalid types inside of intersection.
            types.push(self.parse_docblock_atomic());
            self.skip_doc_eol();
        }

        Type::Intersection(types)
    }

    fn parse_docblock_missing_type(&mut self) -> Type<Name> {
        self.diagnostic(
            ParserDiagnostic::MissingType,
            Severity::Warning,
            self.current_span(),
        );

        Type::Missing
    }

    fn parse_docblock_atomic(&mut self) -> Type<Name> {
        match self.current_kind() {
            TokenKind::LeftParen => {
                self.next();
                self.skip_doc_eol();

                let inner = self.parse_docblock_subparse();

                if inner == Type::Missing {
                    return self.parse_docblock_missing_type();
                }

                self.skip_doc_eol();

                if self.current_kind() != TokenKind::RightParen {
                    self.diagnostic(
                        ParserDiagnostic::ExpectedTokenExFound {
                            expected: vec![TokenKind::RightParen],
                        },
                        Severity::Warning,
                        self.current_span(),
                    );
                } else {
                    self.next();
                }

                if self.current_kind() == TokenKind::LeftBracket {
                    self.parse_docblock_array_or_offset_access(inner)
                } else {
                    inner
                }
            }
            TokenKind::Variable if self.current_symbol() == b"$this" => {
                self.next();

                if self.current_kind() == TokenKind::LeftBracket {
                    self.parse_docblock_array_or_offset_access(Type::This)
                } else {
                    Type::This
                }
            }
            _ => {
                let r#type = self
                    .parse_optional_simple_data_type()
                    .unwrap_or_else(|| self.parse_docblock_missing_type());

                if r#type == Type::Missing {
                    return Type::Missing;
                }

                // FIXME: Check for ! T:: here.
                let current = self.current();

                if current.kind == TokenKind::LessThan {
                    let mut r#type = self.parse_docblock_generic(r#type);

                    if self.current_kind() == TokenKind::LeftBracket {
                        r#type = self.parse_docblock_array_or_offset_access(r#type);
                    }

                    r#type
                } else if current.kind == TokenKind::LeftParen {
                    todo!("parse docblock callable type");
                } else if current.kind == TokenKind::LeftBracket {
                    self.parse_docblock_array_or_offset_access(r#type)
                } else {
                    r#type
                }
            }
        }
    }

    fn parse_docblock_generic(&mut self, lhs: Type<Name>) -> Type<Name> {
        self.next();
        let mut generic_types = vec![];
        let mut is_first = true;

        while is_first || self.current_kind() == TokenKind::Comma {
            if self.current_kind() == TokenKind::Comma {
                self.next();
            }

            self.skip_doc_eol();

            if !is_first && self.current_kind() == TokenKind::GreaterThan {
                break;
            }

            is_first = false;

            // FIXME: Parse variance keywords and wildcards here too.
            generic_types.push(self.parse_docblock_type());

            self.skip_doc_eol();
        }

        if self.current_kind() == TokenKind::GreaterThan {
            self.next();
        } else {
            self.diagnostic(
                ParserDiagnostic::ExpectedTokenExFound {
                    expected: vec![TokenKind::GreaterThan],
                },
                Severity::Warning,
                self.current_span(),
            );
        }

        Type::NamedWithGenerics(Box::new(lhs), generic_types)
    }

    fn parse_docblock_array_or_offset_access(&mut self, lhs: Type<Name>) -> Type<Name> {
        let mut r#type = lhs;

        while let TokenKind::LeftBracket = self.current_kind() {
            self.next();

            // FIXME: Add offset type parsing here.

            if self.current_kind() == TokenKind::RightBracket {
                self.next();

                r#type = Type::TypedArray(Box::new(Type::array_key_types()), Box::new(r#type));
            }
        }

        r#type
    }

    fn parse_docblock_subparse(&mut self) -> Type<Name> {
        match self.current_kind() {
            TokenKind::Question => self.parse_docblock_nullable(),
            TokenKind::Variable => todo!(),
            _ => {
                let r#type = self.parse_docblock_atomic();

                if r#type == Type::Missing {
                    return Type::Missing;
                }

                if self.current_kind() == TokenKind::Identifier && self.current_symbol() == b"is" {
                    todo!("parse docblock conditional type");
                }

                self.skip_doc_eol();

                if self.current_kind() == TokenKind::Pipe {
                    self.parse_docblock_subparse_union(r#type)
                } else if self.current_kind() == TokenKind::Ampersand {
                    self.parse_docblock_subparse_intersection(r#type)
                } else {
                    r#type
                }
            }
        }
    }

    fn parse_dnf_type(&mut self) -> Type<Name> {
        // (A|B|..)&C.. or (A&B&..)|C..
        self.next();
        let ty = self.parse_simple_data_type();

        match self.current_kind() {
            TokenKind::Pipe => {
                let union = self.parse_union_type(ty, true);

                self.skip_right_parenthesis();

                self.parse_intersection_type(union, false)
            }
            TokenKind::Ampersand => {
                let intersection = self.parse_intersection_type(ty, true);

                self.skip_right_parenthesis();

                self.parse_union_type(intersection, false)
            }
            _ => {
                self.diagnostic(
                    ParserDiagnostic::UnexpectedToken {
                        token: self.current().to_owned(),
                    },
                    Severity::Error,
                    self.current_span(),
                );

                Type::Missing
            }
        }
    }

    fn parse_optional_simple_data_type(&mut self) -> Option<Type<Name>> {
        match self.current_kind() {
            TokenKind::Array => {
                self.next();

                Some(Type::Array)
            }
            TokenKind::Callable => {
                self.next();

                Some(Type::Callable)
            }
            TokenKind::Null => {
                self.next();

                Some(Type::Null)
            }
            TokenKind::True => {
                self.next();

                Some(Type::True)
            }
            TokenKind::False => {
                self.next();

                Some(Type::False)
            }
            TokenKind::Static => {
                self.next();

                Some(Type::StaticReference)
            }
            TokenKind::Self_ => {
                self.next();

                Some(Type::SelfReference)
            }
            TokenKind::Parent => {
                self.next();

                Some(Type::ParentReference)
            }
            TokenKind::Enum | TokenKind::From => {
                self.next();

                let id = self.state.id();

                Some(Type::Named(self.maybe_resolve_identifier(
                    id,
                    &self.current(),
                    UseKind::Normal,
                )))
            }
            TokenKind::Identifier => {
                let id = self.current_symbol_as_bytestring();
                self.next();

                let name = &id[..];
                let lowered_name = name.to_ascii_lowercase();
                match lowered_name.as_slice() {
                    b"void" => Some(Type::Void),
                    b"never" => Some(Type::Never),
                    b"float" => Some(Type::Float),
                    b"bool" => Some(Type::Boolean),
                    b"int" => Some(Type::Integer),
                    b"string" => Some(Type::String),
                    b"object" => Some(Type::Object),
                    b"mixed" => Some(Type::Mixed),
                    b"iterable" => Some(Type::Iterable),
                    b"null" => Some(Type::Null),
                    b"true" => Some(Type::True),
                    b"false" => Some(Type::False),
                    b"array" => Some(Type::Array),
                    b"callable" => Some(Type::Callable),
                    _ => {
                        let id = self.state.id();

                        Some(Type::Named(self.maybe_resolve_identifier(
                            id,
                            &self.current(),
                            UseKind::Normal,
                        )))
                    }
                }
            }
            TokenKind::FullyQualifiedIdentifier => {
                let span = self.next();
                let symbol = self.current_symbol_as_bytestring();
                let resolved = self.state.strip_leading_namespace_qualifier(&symbol);

                Some(Type::Named(Name::resolved(
                    self.state.id(),
                    resolved,
                    symbol,
                    span,
                )))
            }
            TokenKind::QualifiedIdentifier => {
                self.next();

                let id = self.state.id();
                let name = self.maybe_resolve_identifier(id, &self.current(), UseKind::Normal);

                Some(Type::Named(name))
            }
            _ => None,
        }
    }

    fn parse_simple_data_type(&mut self) -> Type<Name> {
        match self.parse_optional_simple_data_type() {
            Some(ty) => ty,
            None => {
                self.diagnostic(
                    ParserDiagnostic::MissingType,
                    Severity::Error,
                    self.current_span(),
                );

                Type::Missing
            }
        }
    }

    fn parse_nullable_type(&mut self) -> Type<Name> {
        let span = self.next();

        let ty = self.parse_simple_data_type();

        if ty.standalone() {
            self.diagnostic(
                ParserDiagnostic::StandaloneTypeUsedInNullableType,
                Severity::Error,
                span,
            );
        }

        Type::Nullable(Box::new(ty))
    }

    fn parse_union_type(&mut self, other: Type<Name>, within_dnf: bool) -> Type<Name> {
        if other.standalone() {
            self.diagnostic(
                ParserDiagnostic::StandaloneTypeUsedInUnionType,
                Severity::Error,
                self.current_span(),
            );
        }

        let mut types = vec![other];
        let mut last_pipe = self.skip(TokenKind::Pipe);

        loop {
            let current = self.current();
            let ty = if current.kind == TokenKind::LeftParen {
                if within_dnf {
                    // don't allow nesting.
                    //
                    // examples on how we got here:
                    //
                    // v-- get_intersection_type: within_dnf = false
                    //     v-- get_union_type: within_dnf = true
                    //      v-- error
                    // F&(A|(D&S))
                    //
                    // v-- get_intersection_type: within_dnf = false
                    //     v-- get_union_type: within_dnf = true
                    //        v-- error
                    // F&(A|B|(D&S))
                    self.diagnostic(
                        ParserDiagnostic::NestedDisjunctiveNormalFormType,
                        Severity::Error,
                        current.span,
                    );
                }

                self.next();

                let other = self.parse_simple_data_type();
                let ty = self.parse_intersection_type(other, true);

                self.skip_right_parenthesis();

                ty
            } else {
                let ty = self.parse_simple_data_type();
                if ty.standalone() {
                    self.diagnostic(
                        ParserDiagnostic::StandaloneTypeUsedInUnionType,
                        Severity::Error,
                        last_pipe,
                    );
                }

                ty
            };

            types.push(ty);

            if self.current_kind() == TokenKind::Pipe {
                last_pipe = self.skip(TokenKind::Pipe);
            } else {
                break;
            }
        }

        Type::Union(types)
    }

    fn parse_intersection_type(&mut self, other: Type<Name>, within_dnf: bool) -> Type<Name> {
        if other.standalone() {
            self.diagnostic(
                ParserDiagnostic::StandaloneTypeUsedInIntersectionType,
                Severity::Error,
                self.current_span(),
            );
        }

        let mut types = vec![other];

        let mut last_ampersand = self.skip(TokenKind::Ampersand);

        loop {
            let current = self.current();
            let ty = if current.kind == TokenKind::LeftParen {
                if within_dnf {
                    // don't allow nesting.
                    //
                    // examples on how we got here:
                    //
                    //  v-- get_union_type: within_dnf = false
                    //     v-- get_intersection_type: within_dnf = true
                    //      v-- error
                    // F|(A&(D|S))
                    //
                    //  v-- get_union_type: within_dnf = false
                    //     v-- get_intersection_type: within_dnf = true
                    //        v-- error
                    // F|(A&B&(D|S))

                    self.diagnostic(
                        ParserDiagnostic::NestedDisjunctiveNormalFormType,
                        Severity::Error,
                        current.span,
                    );
                }

                self.next();

                let other = self.parse_simple_data_type();
                let ty = self.parse_union_type(other, true);

                self.skip_right_parenthesis();

                ty
            } else {
                let ty = self.parse_simple_data_type();
                if ty.standalone() {
                    self.diagnostic(
                        ParserDiagnostic::StandaloneTypeUsedInIntersectionType,
                        Severity::Error,
                        last_ampersand,
                    );
                }

                ty
            };

            types.push(ty);

            if self.current_kind() == TokenKind::Ampersand
                && !matches!(
                    self.peek_kind(),
                    TokenKind::Variable | TokenKind::Ellipsis | TokenKind::Ampersand
                )
            {
                last_ampersand = self.skip(TokenKind::Ampersand);
            } else {
                break;
            }
        }

        Type::Intersection(types)
    }
}
