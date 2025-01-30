use pxp_span::{IsSpanned, Span};
use serde::{Deserialize, Serialize};

use crate::{FileId, HasFileId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    file: FileId,
    span: Span,
}

impl Location {
    pub fn new(file: FileId, span: Span) -> Self {
        Self { file, span }
    }
}

impl IsSpanned for Location {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasFileId for Location {
    fn file_id(&self) -> FileId {
        self.file
    }
}

pub trait HasLocation {
    fn location(&self) -> Location;
}
