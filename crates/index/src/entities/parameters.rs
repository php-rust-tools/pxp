use pxp_ast::{Name, ResolvedName, SimpleVariable};
use pxp_type::Type;
use serde::{Deserialize, Serialize};

use crate::location::Location;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameters {
    parameters: Vec<Parameter>,
}

impl Parameters {
    pub fn new(parameters: Vec<Parameter>) -> Self {
        Self { parameters }
    }

    pub fn len(&self) -> usize {
        self.parameters.len()
    }

    pub fn is_empty(&self) -> bool {
        self.parameters.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Parameter> {
        self.parameters.iter()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    pub(crate) name: SimpleVariable,
    pub(crate) r#type: Option<Type<ResolvedName>>,
    pub(crate) optional: bool,
    pub(crate) variadic: bool,
    pub(crate) location: Location,
}
