use std::{
    ops::{Index, IndexMut},
    slice::Iter,
};

use crate::grid::cell::Cell;

#[derive(Debug, Clone)]
pub struct Row {
    pub inner: Vec<Cell>,
}

impl Row {
    pub fn new(columns: usize) -> Self {
        let mut inner = Vec::with_capacity(columns);

        inner.resize(columns, Cell::default());

        Self { inner }
    }

    pub fn reset(&mut self) {
        for cell in &mut self.inner {
            cell.c = None;
        }
    }
}

impl Index<usize> for Row {
    type Output = Cell;

    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

impl IndexMut<usize> for Row {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.inner[index]
    }
}

impl<'a> IntoIterator for &'a Row {
    type Item = &'a Cell;
    type IntoIter = Iter<'a, Cell>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}
