use std::fmt::{Debug, Display};

use pxp_bytestring::ByteString;
use pxp_span::Span;

#[derive(Debug, PartialEq, Eq, Clone, Hash, Default)]
pub enum Type<N: Debug + Display> {
    Named(N),
    Generic(Box<Type<N>>, Vec<Type<N>>),
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
    ClassString,
    String,
    Array,
    Object,
    #[default]
    Mixed,
    Callable,
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
        unsealed_type: Option<Box<ShapeUnsealedType<N>>>
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
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShapeItem<N: Debug + Display> {
    pub key_name: Option<ShapeItemKey>,
    pub value_type: Type<N>,
    pub optional: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ShapeItemKey {
    Integer(ByteString),
    String(ByteString),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShapeUnsealedType<N: Debug + Display> {
    pub key_type: Option<Type<N>>,
    pub value_type: Type<N>,
}

impl Display for ShapeItemKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShapeItemKey::Integer(value) => write!(f, "{}", value),
            ShapeItemKey::String(value) => write!(f, "{}", value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
            Type::Shaped { base, items, sealed, unsealed_type } => {
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

                write!(f, "}}")
            },
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
        }
    }
}
