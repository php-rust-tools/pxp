use std::path::{Path, PathBuf};

use entities::EntityRegistry;
use file::FileRegistry;

mod entities;
mod file;
mod indexer;
mod location;
mod reflection;
pub mod storage;

pub use file::{FileId, HasFileId};
use indexer::IndexingVisitor;
use pxp_ast::{visitor::Visitor, Statement};
use pxp_bytestring::ByteString;
use pxp_lexer::Lexer;
use pxp_parser::Parser;

pub use entities::{FunctionEntity, Parameter, Parameters};
pub use location::{HasLocation, Location};
pub use reflection::{
    ReflectionClass, ReflectionFunction, ReflectionFunctionLike, ReflectionParameter,
    ReflectionType, ReflectsParameters,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Index {
    files: FileRegistry,
    pub(crate) entities: EntityRegistry,
}

impl Index {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn index_file(&mut self, path: &Path) {
        let file_id = self.files.get_or_insert(path);
        let contents = std::fs::read(path).unwrap();
        let parse_result = Parser::parse(Lexer::new(&contents));

        self.index(file_id, &parse_result.ast);
    }

    pub fn index(&mut self, file_id: FileId, ast: &[Statement]) {
        let mut visitor = IndexingVisitor::new(file_id, self);
        visitor.visit(ast);
    }

    pub fn number_of_files(&self) -> usize {
        self.files.len()
    }

    pub fn number_of_functions(&self) -> usize {
        self.entities.functions().len()
    }

    pub fn get_function(&self, name: impl Into<ByteString>) -> Option<ReflectionFunction> {
        self.entities
            .get_function(name)
            .map(ReflectionFunction::new)
    }

    pub fn number_of_classes(&self) -> usize {
        self.entities.classes().len()
    }

    pub fn get_class(&self, name: impl Into<ByteString>) -> Option<ReflectionClass> {
        self.entities.get_class(name).map(ReflectionClass::new)
    }

    pub fn get_file_path(&self, from: impl HasFileId) -> Option<&std::path::Path> {
        self.files.get_file_path(from.file_id())
    }

    pub fn get_file_path_unchecked(&self, from: impl HasFileId) -> &std::path::Path {
        self.files.get_file_path_unchecked(from.file_id())
    }

    pub fn search_files(&self, query: &str) -> Vec<&PathBuf> {
        self.files
            .files()
            .keys()
            .filter(|path| path.to_string_lossy().contains(query))
            .collect()
    }

    pub fn search_classes(&self, query: &str) -> Vec<ReflectionClass> {
        self.entities
            .search_classes(query)
            .into_iter()
            .map(ReflectionClass::new)
            .collect()
    }

    pub fn search_functions(&self, query: &str) -> Vec<ReflectionFunction> {
        self.entities
            .search_functions(query)
            .into_iter()
            .map(ReflectionFunction::new)
            .collect()
    }
}
