use std::fmt::Debug;

use pxp_ast::Name;
use pxp_symbol::Symbol;
use pxp_type::Type;

use crate::{class_like::{ClassLike, Method, Property}, constant::Constant, function::Function, parameter::Parameter, Index};

#[derive(Clone)]
pub struct ReflectionFunction<'a> {
    pub(crate) function: &'a Function,
    pub(crate) index: &'a Index,
}

impl<'a> ReflectionFunction<'a> {
    pub fn get_name(&self) -> Symbol {
        self.function.name
    }

    pub fn get_short_name(&self) -> Symbol {
        self.function.short
    }

    pub fn get_namespace(&self) -> Option<Symbol> {
        self.function.namespace
    }

    pub fn get_return_type(&self) -> &Type<Name> {
        &self.function.return_type
    }

    pub fn returns_by_reference(&self) -> bool {
        self.function.returns_by_reference
    }

    pub fn get_parameters(&'a self) -> impl Iterator<Item = ReflectionParameter> + 'a {
        self.function.parameters.iter().map(|parameter| ReflectionParameter { parameter, index: self.index })
    }

    pub fn get_parameter(&self, name: Symbol) -> Option<ReflectionParameter> {
        self.function.parameters.iter().find(|parameter| parameter.name == name).map(|parameter| ReflectionParameter { parameter, index: self.index })
    }
}

#[derive(Clone)]
pub struct ReflectionParameter<'a> {
    pub(crate) parameter: &'a Parameter,
    pub(crate) index: &'a Index,
}

impl<'a> ReflectionParameter<'a> {
    pub fn get_name(&self) -> Symbol {
        self.parameter.name
    }

    pub fn get_type(&self) -> &Type<Name> {
        &self.parameter.r#type
    }

    pub fn is_optional(&self) -> bool {
        self.parameter.default
    }

    pub fn is_passed_by_reference(&self) -> bool {
        self.parameter.reference
    }

    pub fn is_variadic(&self) -> bool {
        self.parameter.variadic
    }
}

#[derive(Clone)]
pub struct ReflectionClass<'a> {
    pub(crate) class: &'a ClassLike,
    pub(crate) index: &'a Index,
}

impl<'a> ReflectionClass<'a> {
    pub fn is_final(&self) -> bool {
        self.class.modifiers.has_final()
    }

    pub fn is_abstract(&self) -> bool {
        self.class.modifiers.has_abstract()
    }

    pub fn is_readonly(&self) -> bool {
        self.class.modifiers.has_readonly()
    }

    pub fn get_properties(&'a self) -> impl Iterator<Item = ReflectionProperty> + 'a {
        self.class.properties.iter().map(|property| ReflectionProperty { class: self, property, index: self.index })
    }

    pub fn get_property(&self, name: Symbol) -> Option<ReflectionProperty> {
        self.class.properties.iter().find(|property| property.name == name).map(|property| ReflectionProperty { class: self, property, index: self.index })
    }

    pub fn get_name(&self) -> Symbol {
        self.class.name
    }

    pub fn get_short_name(&self) -> Symbol {
        self.class.short
    }

    pub fn get_namespace(&self) -> Option<Symbol> {
        self.class.namespace
    }

    pub fn get_parent(&self) -> Option<ReflectionClass<'a>> {
        self.class.parent.as_ref().map(|parent| self.index.get_class(*parent)).flatten()
    }

    pub fn get_interfaces(&self) -> impl Iterator<Item = ReflectionClass> + '_ {
        self.class.interfaces.iter().map(move |interface| self.index.get_class(*interface)).flatten()
    }

    pub fn get_methods(&'a self) -> impl Iterator<Item = ReflectionMethod> + 'a {
        self.class.methods.iter().map(|method| ReflectionMethod { class: self, method, index: self.index })
    }

    pub fn get_method(&self, name: Symbol) -> Option<ReflectionMethod> {
        self.class.methods.iter().find(|method| method.name == name).map(|method| ReflectionMethod { class: self, method, index: self.index })
    }
}

impl<'a> Debug for ReflectionClass<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReflectionClass")
            .field("name", &self.get_name())
            .field("short", &self.get_short_name())
            .field("namespace", &self.get_namespace())
            .field("properties", &self.get_properties().collect::<Vec<_>>())
            .field("methods", &self.get_methods().collect::<Vec<_>>())
            .finish()
    }
}

#[derive(Clone)]
pub struct ReflectionProperty<'a> {
    pub(crate) class: &'a ReflectionClass<'a>,
    pub(crate) property: &'a Property,
    pub(crate) index: &'a Index,
}

impl<'a> ReflectionProperty<'a> {
    pub fn is_static(&self) -> bool {
        self.property.modifiers.has_static()
    }

    pub fn is_readonly(&self) -> bool {
        self.property.modifiers.has_readonly()
    }

    pub fn is_public(&self) -> bool {
        self.property.modifiers.is_public()
    }

    pub fn is_protected(&self) -> bool {
        self.property.modifiers.is_protected()
    }

    pub fn is_private(&self) -> bool {
        self.property.modifiers.is_private()
    }

    pub fn get_type(&self) -> &Type<Name> {
        &self.property.r#type
    }
}

impl Debug for ReflectionProperty<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReflectionProperty")
            .field("name", &self.property.name)
            .field("type", &self.property.r#type)
            .field("default", &self.property.default)
            .field("modifiers", &self.property.modifiers)
            .finish()
    }
}

#[derive(Clone)]
pub struct ReflectionMethod<'a> {
    pub(crate) class: &'a ReflectionClass<'a>,
    pub(crate) method: &'a Method,
    pub(crate) index: &'a Index,
}

impl<'a> ReflectionMethod<'a> {
    pub fn is_static(&self) -> bool {
        self.method.modifiers.has_static()
    }

    pub fn is_public(&self) -> bool {
        self.method.modifiers.is_public()
    }

    pub fn is_protected(&self) -> bool {
        self.method.modifiers.is_protected()
    }

    pub fn is_private(&self) -> bool {
        self.method.modifiers.is_private()
    }

    pub fn get_return_type(&self) -> &Type<Name> {
        &self.method.return_type
    }
}

impl Debug for ReflectionMethod<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReflectionMethod")
            .field("name", &self.method.name)
            .field("return_type", &self.method.return_type)
            .field("modifiers", &self.method.modifiers)
            .field("parameters", &self.method.parameters)
            .finish()
    }
}

#[derive(Clone)]
pub struct ReflectionConstant<'a> {
    pub(crate) constant: &'a Constant,
    pub(crate) index: &'a Index,
}