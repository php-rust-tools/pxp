use pxp_ast::ResolvedName;
use serde::{Deserialize, Serialize};

use crate::{location::Location, HasFileId};

use super::MethodEntity;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassEntity {
    pub(crate) name: ResolvedName,
    pub(crate) kind: ClassEntityKind,
    pub(crate) methods: Vec<MethodEntity>,
    pub(crate) location: Location,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ClassEntityKind {
    Class,
    Interface,
    Enum,
    Trait,
}

impl HasFileId for ClassEntity {
    fn file_id(&self) -> crate::FileId {
        self.location.file_id()
    }
}
