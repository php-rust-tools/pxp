use pxp_ast::{MethodModifierGroup, Name, ResolvedName, SimpleIdentifier};
use pxp_type::Type;
use serde::{Deserialize, Serialize};

use crate::{location::Location, HasFileId};

use super::Parameters;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MethodEntity {
    pub(crate) name: SimpleIdentifier,
    pub(crate) parameters: Parameters,
    pub(crate) return_type: Option<Type<ResolvedName>>,
    pub(crate) returns_reference: bool,
    pub(crate) modifiers: MethodModifierGroup,
    pub(crate) location: Location,
}

impl HasFileId for MethodEntity {
    fn file_id(&self) -> crate::FileId {
        self.location.file_id()
    }
}
