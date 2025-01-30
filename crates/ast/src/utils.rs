use std::slice::Iter;
use std::slice::IterMut;

use pxp_span::IsSpanned;
use pxp_span::Span;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]

pub struct CommaSeparated<T> {
    pub inner: Vec<T>,
    pub commas: Vec<Span>, // `,`
}

impl<T> CommaSeparated<T> {
    pub fn iter(&self) -> Iter<'_, T> {
        self.inner.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        self.inner.iter_mut()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl<T: IsSpanned> IsSpanned for CommaSeparated<T> {
    fn span(&self) -> Span {
        self.inner.span()
    }
}

impl<T> IntoIterator for CommaSeparated<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}
