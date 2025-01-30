use std::fmt::{Debug, Display};

use pxp_bytestring::ByteString;
use pxp_span::Span;
use serde::{Deserialize, Serialize};
use strum::EnumIs;

#[derive(Debug, PartialEq, Eq, Clone, Hash, Default, EnumIs, Serialize, Deserialize)]
pub enum Type<N: Debug + Display> {
    Named(N),
    Generic(Box<Type<N>>, Vec<GenericTypeArgument<N>>),
    Nullable(Box<Type<N>>),
    Union(Vec<Type<N>>),
    Intersection(Vec<Type<N>>),
    Void,
    Null,
    True,
    False,
    Never,
    Float,
    Boolean,
    Integer,
    NonNegativeInteger,
    ClassString,
    String,
    LiteralString(ByteString),
    NumericString,
    NonEmptyString,
    Empty,
    List,
    NonEmptyList,
    Array,
    NonEmptyArray,
    Object,
    #[default]
    Mixed,
    NonEmptyMixed,
    Callable,
    CallableString,
    CallableSignature(Box<Type<N>>, Vec<CallableParameter<N>>, Box<Type<N>>),
    Iterable,
    StaticReference,
    SelfReference,
    ParentReference,
    ArrayKey,
    TypedArray(Box<Type<N>>, Box<Type<N>>),
    Shaped {
        base: Box<Type<N>>,
        items: Vec<ShapeItem<N>>,
        sealed: bool,
        unsealed_type: Option<Box<ShapeUnsealedType<N>>>,
    },
    ConditionalForParameter {
        parameter: ByteString,
        negated: bool,
        target: Box<Type<N>>,
        then: Box<Type<N>>,
        otherwise: Box<Type<N>>,
    },
    Conditional {
        subject: Box<Type<N>>,
        negated: bool,
        target: Box<Type<N>>,
        then: Box<Type<N>>,
        otherwise: Box<Type<N>>,
    },
    ValueOf,
    This,
    Missing,
    ConstExpr(Box<ConstExpr<N>>),
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConstExpr<N: Debug + Display> {
    NegativeInteger(ByteString),
    Integer(ByteString),
    Float(ByteString),
    String(ByteString),
    ConstFetch(Type<N>, ByteString),
}

impl<N: Debug + Display> Display for ConstExpr<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NegativeInteger(value) => write!(f, "-{}", value),
            Self::Integer(value) => write!(f, "{}", value),
            Self::Float(value) => write!(f, "{}", value),
            Self::String(value) => write!(f, "{}", value),
            Self::ConstFetch(target, names) => write!(f, "{}::{}", target, names),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GenericTypeArgument<N: Debug + Display> {
    pub r#type: Type<N>,
    pub variance: Option<GenericTypeArgumentVariance>,
}

impl<N: Debug + Display> Display for GenericTypeArgument<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(variance) = &self.variance {
            write!(f, "{} ", variance)?;
        }

        write!(f, "{}", self.r#type)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GenericTypeArgumentVariance {
    Invariant,
    Covariant,
    Contravariant,
    Bivariant,
}

impl Display for GenericTypeArgumentVariance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invariant => write!(f, ""),
            Self::Covariant => write!(f, "covariant"),
            Self::Contravariant => write!(f, "contravariant"),
            Self::Bivariant => write!(f, "bivariant"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShapeItem<N: Debug + Display> {
    pub key_name: Option<ShapeItemKey>,
    pub value_type: Type<N>,
    pub optional: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShapeItemKey {
    Integer(ByteString),
    String(ByteString),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShapeUnsealedType<N: Debug + Display> {
    pub key_type: Option<Type<N>>,
    pub value_type: Type<N>,
}

impl<N: Debug + Display> Display for ShapeUnsealedType<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<")?;

        if let Some(key_type) = &self.key_type {
            write!(f, "{}, ", key_type)?;
        }

        write!(f, "{}>", self.value_type)
    }
}

impl Display for ShapeItemKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShapeItemKey::Integer(value) => write!(f, "{}", value),
            ShapeItemKey::String(value) => write!(f, "{}", value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CallableParameter<N: Debug + Display> {
    pub r#type: Type<N>,
    pub ellipsis: Option<Span>,
    pub ampersand: Option<Span>,
    pub equal: Option<Span>,
    pub name: Option<ByteString>,
}

impl<N: Debug + Display> Display for CallableParameter<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.r#type)?;

        if self.ellipsis.is_some() {
            write!(f, " ...")?;
        } else if self.ampersand.is_some() {
            write!(f, " &")?;
        }

        if let Some(name) = &self.name {
            write!(f, " {}", name)?;
        }

        if self.equal.is_some() {
            write!(f, "=")?;
        }

        Ok(())
    }
}

impl<N: Debug + Display> Type<N> {
    pub fn map<T: Debug + Display>(&self, cb: impl FnOnce(&Type<N>) -> Type<T>) -> Type<T> {
        cb(self)
    }

    pub fn standalone(&self) -> bool {
        matches!(
            self,
            Type::Mixed | Type::Never | Type::Void | Type::Nullable(_)
        )
    }

    pub fn nullable(&self) -> bool {
        matches!(self, Type::Nullable(_))
    }

    pub fn allows_null(&self) -> bool {
        match self {
            Type::Nullable(_) | Type::Null => true,
            Type::Union(types) | Type::Intersection(types) => types.iter().any(|t| t.allows_null()),
            _ => false,
        }
    }

    pub fn includes_callable(&self) -> bool {
        match &self {
            Self::Callable => true,
            Self::Union(types) | Self::Intersection(types) => {
                types.iter().any(|x| x.includes_callable())
            }
            _ => false,
        }
    }

    pub fn is_bottom(&self) -> bool {
        matches!(self, Type::Never | Type::Void)
    }

    pub fn is_object_like(&self) -> bool {
        match self {
            Type::Named(_) | Type::Object => true,
            Type::Nullable(inner) => inner.is_object_like(),
            Type::Union(inner) => inner.iter().any(|t| t.is_object_like()),
            Type::Intersection(inner) => inner.iter().any(|t| t.is_object_like()),
            Type::SelfReference | Type::ParentReference | Type::StaticReference => true,
            _ => false,
        }
    }

    pub fn array_key_types() -> Type<N> {
        Self::Union(vec![Self::String, Self::Integer])
    }
}

impl<N: Debug + Display> Display for Type<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Type::LiteralString(_) => write!(f, "literal-string"),
            Type::CallableString => write!(f, "callable-string"),
            Type::NonEmptyList => write!(f, "non-empty-list"),
            Type::NonEmptyArray => write!(f, "non-empty-array"),
            Type::NonNegativeInteger => write!(f, "non-negative-int"),
            Type::Empty => write!(f, "empty"),
            Type::NonEmptyMixed => write!(f, "non-empty-mixed"),
            Type::NonEmptyString => write!(f, "non-empty-string"),
            Type::ConstExpr(inner) => write!(f, "{}", inner),
            Type::List => write!(f, "list"),
            Type::NumericString => write!(f, "numeric-string"),
            Type::Shaped {
                base,
                items,
                sealed,
                unsealed_type,
            } => {
                write!(f, "{base}{{")?;

                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }

                    if let Some(key) = &item.key_name {
                        write!(f, "{}{}: ", key, if item.optional { "?" } else { "" })?;
                    }

                    write!(f, "{}", item.value_type)?;
                }

                if !sealed {
                    write!(f, ", ...")?;

                    if let Some(unsealed_type) = unsealed_type {
                        write!(f, "{}", unsealed_type)?;
                    }
                }

                write!(f, "}}")
            }
            Type::ClassString => write!(f, "class-string"),
            Type::ValueOf => write!(f, "value-of"),
            Type::Named(inner) => write!(f, "{}", inner),
            Type::Generic(inner, templates) => {
                write!(
                    f,
                    "{}<{}>",
                    inner,
                    templates
                        .iter()
                        .map(|t| t.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
            Type::Nullable(inner) => write!(f, "?{}", inner),
            Type::Union(inner) => write!(
                f,
                "{}",
                inner
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<String>>()
                    .join(" | ")
            ),
            Type::Intersection(inner) => write!(
                f,
                "{}",
                inner
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<String>>()
                    .join(" & ")
            ),
            Type::Void => write!(f, "void"),
            Type::Null => write!(f, "null"),
            Type::True => write!(f, "true"),
            Type::False => write!(f, "false"),
            Type::Never => write!(f, "never"),
            Type::Float => write!(f, "float"),
            Type::Boolean => write!(f, "bool"),
            Type::Integer => write!(f, "int"),
            Type::String => write!(f, "string"),
            Type::Array => write!(f, "array"),
            Type::Object => write!(f, "object"),
            Type::Mixed => write!(f, "mixed"),
            Type::CallableSignature(callable, parameters, return_type) => write!(
                f,
                "{}({}): {}",
                callable,
                parameters
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
                return_type
            ),
            Type::Callable => write!(f, "callable"),
            Type::Iterable => write!(f, "iterable"),
            Type::StaticReference => write!(f, "static"),
            Type::SelfReference => write!(f, "self"),
            Type::ParentReference => write!(f, "parent"),
            Type::ArrayKey => write!(f, "array-key"),
            Type::TypedArray(key, value) => write!(f, "array<{}, {}>", key, value),
            Type::This => write!(f, "$this"),
            Type::ConditionalForParameter {
                parameter,
                negated,
                target,
                then,
                otherwise,
            } => {
                write!(
                    f,
                    "{} is {}{} ? {} : {}",
                    parameter,
                    if *negated { "not " } else { "" },
                    target,
                    then,
                    otherwise
                )
            }
            Type::Conditional { .. } => todo!(),
            Type::Missing => write!(f, "<missing>"),
            Type::Invalid => write!(f, "<invalid>"),
        }
    }
}
