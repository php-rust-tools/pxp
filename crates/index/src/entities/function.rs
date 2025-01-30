use pxp_ast::{Name, ResolvedName};
use pxp_type::Type;
use serde::{Deserialize, Serialize};

use crate::{location::Location, FileId, HasFileId};

use super::parameters::Parameters;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionEntity {
    pub(crate) name: ResolvedName,
    pub(crate) parameters: Parameters,
    pub(crate) return_type: Option<Type<ResolvedName>>,
    pub(crate) returns_reference: bool,
    pub(crate) location: Location,
}

impl HasFileId for FunctionEntity {
    fn file_id(&self) -> FileId {
        self.location.file_id()
    }
}
