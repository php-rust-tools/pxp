use std::fmt::Display;

use pxp_token::{Token, TokenKind};

#[derive(Debug, Clone)]
pub enum ParserDiagnostic {
    UnexpectedToken {
        token: Token,
    },
    ExpectedToken {
        expected: Vec<TokenKind>,
        found: Token,
    },
    InvalidSpreadOperator,
    InvalidTargetForAttributes,
    CannotMixKeyedAndUnkeyedListEntries,
    AbstractMethodInNonAbstractClass,
    CannotHaveMultipleDefaultArmsInMatch,
    MissingType,
    StandaloneTypeUsedInNullableType,
    StandaloneTypeUsedInUnionType,
    StandaloneTypeUsedInIntersectionType,
    NestedDisjunctiveNormalFormType,
    InvalidBackedEnumType,
    UnitEnumsCannotHaveCaseValues,
    BackedEnumCaseMustHaveValue,
    CannotUseReservedKeywordAsTypeName,
    CannotUseReservedKeywordAsLabel,
    CannotUseReservedKeywordAsConstantName,
    InvalidClassModifier,
    InvalidMethodModifier,
    InvalidPropertyModifier,
    InvalidConstantModifier,
    CannotUseFinalWithAbstract,
    CannotUseFinalWithPrivateOnConstant,
    DuplicateModifier,
    MultipleVisibilityModifiers,
    CannotMixBracketedAndUnbracketedNamespaceDeclarations,
    NestedNamespace,
    PromotedPropertyCannotBeVariadic,
    ForbiddenTypeUsedInProperty,
    ReadonlyPropertyMustHaveType,
    CannotUsePositionalArgumentAfterNamedArgument,
    PositionalArgumentsOnly,
    OnlyAllowedOneArgument,
    ArgumentRequired,
    StaticPropertyCannotBeReadonly,
    ReadonlyPropertyCannotHaveDefaultValue,
    TryMustHaveCatchOrFinally,
    DynamicVariableNotAllowed,
    UnexpectedEndOfFile,
    UnexpectedEndOfFileExpected {
        expected: Vec<TokenKind>,
    },
    MixedImportTypes,
    InvalidDocBodyIndentationLevel(usize),
    InvalidDocIndentation,
}

impl Display for ParserDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserDiagnostic::UnexpectedToken { token } => {
                write!(f, "unexpected token {}", token.kind)
            }
            ParserDiagnostic::ExpectedToken { expected, found } => {
                if expected.len() == 1 {
                    write!(
                        f,
                        "unexpected token {}, expected {}",
                        found.kind,
                        expected.first().unwrap()
                    )
                } else {
                    write!(
                        f,
                        "unexpected token {}, expected one of {}",
                        found.kind,
                        expected
                            .iter()
                            .map(|kind| format!("{}", kind))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            }
            ParserDiagnostic::InvalidSpreadOperator => write!(f, "cannot use spread operator here"),
            ParserDiagnostic::InvalidTargetForAttributes => {
                write!(f, "invalid target for attributes")
            }
            ParserDiagnostic::CannotMixKeyedAndUnkeyedListEntries => {
                write!(f, "cannot mix keyed and unkeyed list entries")
            }
            ParserDiagnostic::AbstractMethodInNonAbstractClass => {
                write!(f, "cannot declare abstract method in non-abstract class")
            }
            ParserDiagnostic::CannotHaveMultipleDefaultArmsInMatch => {
                write!(f, "cannot have multiple default arms in match")
            }
            ParserDiagnostic::MissingType => write!(f, "missing type"),
            ParserDiagnostic::StandaloneTypeUsedInNullableType => {
                write!(f, "cannot use standalone type in nullable type")
            }
            ParserDiagnostic::StandaloneTypeUsedInUnionType => {
                write!(f, "cannot use standalone type in union type")
            }
            ParserDiagnostic::StandaloneTypeUsedInIntersectionType => {
                write!(f, "cannot use standalone type in intersection type")
            }
            ParserDiagnostic::NestedDisjunctiveNormalFormType => {
                write!(f, "DNF types cannot be nested")
            }
            ParserDiagnostic::InvalidBackedEnumType => {
                write!(f, "invalid backed enum type, must be `string` or `int`")
            }
            ParserDiagnostic::UnitEnumsCannotHaveCaseValues => {
                write!(f, "unit enums cannot have case values")
            }
            ParserDiagnostic::BackedEnumCaseMustHaveValue => {
                write!(f, "backed enum case must have value")
            }
            ParserDiagnostic::CannotUseReservedKeywordAsTypeName => {
                write!(f, "cannot use reserved keyword as type name")
            }
            ParserDiagnostic::CannotUseReservedKeywordAsLabel => {
                write!(f, "cannot use reserved keyword as label")
            }
            ParserDiagnostic::CannotUseReservedKeywordAsConstantName => {
                write!(f, "cannot use reserved keyword as constant name")
            }
            ParserDiagnostic::InvalidClassModifier => write!(f, "invalid class modifier"),
            ParserDiagnostic::InvalidMethodModifier => write!(f, "invalid method modifier"),
            ParserDiagnostic::InvalidPropertyModifier => write!(f, "invalid property modifier"),
            ParserDiagnostic::InvalidConstantModifier => write!(f, "invalid constant modifier"),
            ParserDiagnostic::CannotUseFinalWithAbstract => {
                write!(f, "cannot use final and abstract together")
            }
            ParserDiagnostic::CannotUseFinalWithPrivateOnConstant => write!(
                f,
                "private constant cannot be final as it is not visible to other classes"
            ),
            ParserDiagnostic::DuplicateModifier => write!(f, "duplicate modifier"),
            ParserDiagnostic::MultipleVisibilityModifiers => {
                write!(f, "cannot have multiple visibility modifiers")
            }
            ParserDiagnostic::CannotMixBracketedAndUnbracketedNamespaceDeclarations => write!(
                f,
                "cannot mix bracketed and unbracketed namespace declarations"
            ),
            ParserDiagnostic::NestedNamespace => write!(f, "cannot nest namespaces"),
            ParserDiagnostic::PromotedPropertyCannotBeVariadic => {
                write!(f, "promoted property cannot be variadic")
            }
            ParserDiagnostic::ForbiddenTypeUsedInProperty => {
                write!(f, "forbidden type used in property")
            }
            ParserDiagnostic::ReadonlyPropertyMustHaveType => {
                write!(f, "readonly property must have type")
            }
            ParserDiagnostic::CannotUsePositionalArgumentAfterNamedArgument => {
                write!(f, "cannot use positional argument after named argument")
            }
            ParserDiagnostic::PositionalArgumentsOnly => {
                write!(f, "only positional arguments are allowed")
            }
            ParserDiagnostic::OnlyAllowedOneArgument => write!(f, "only one argument is allowed"),
            ParserDiagnostic::ArgumentRequired => write!(f, "argument required"),
            ParserDiagnostic::StaticPropertyCannotBeReadonly => {
                write!(f, "static property cannot be readonly")
            }
            ParserDiagnostic::ReadonlyPropertyCannotHaveDefaultValue => {
                write!(f, "readonly property cannot have default value")
            }
            ParserDiagnostic::TryMustHaveCatchOrFinally => {
                write!(f, "try must have catch or finally")
            }
            ParserDiagnostic::DynamicVariableNotAllowed => {
                write!(f, "dynamic variable not allowed")
            }
            ParserDiagnostic::UnexpectedEndOfFile => write!(f, "unexpected end of file"),
            ParserDiagnostic::UnexpectedEndOfFileExpected { expected } => {
                if expected.len() == 1 {
                    write!(
                        f,
                        "unexpected end of file, expected {}",
                        expected.first().unwrap()
                    )
                } else {
                    write!(
                        f,
                        "unexpected end of file, expected one of {}",
                        expected
                            .iter()
                            .map(|kind| format!("{}", kind))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            }
            ParserDiagnostic::MixedImportTypes => write!(f, "cannot mix import types"),
            ParserDiagnostic::InvalidDocBodyIndentationLevel(level) => write!(f, "heredoc / nowdoc body indentation level [{}] is invalid", level),
            ParserDiagnostic::InvalidDocIndentation => write!(f, "heredoc / nowdoc body indentation is invalid"),
        }
    }
}
