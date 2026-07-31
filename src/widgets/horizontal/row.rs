use std::cmp::max;
use std::collections::VecDeque;
use crate::{Dimension, MeasurementSpecifics, Measurements};
use crate::widgets::{Cell, CellAnchor, CellDimension};

#[derive(Clone)]
pub struct Row {
    pub dim: Dimension,
    pub cells: VecDeque<(Cell, Measurements)>,
}

impl Row {
    fn new(dim: Dimension, cells: VecDeque<(Cell, Measurements)>) -> Self {
        Self { dim, cells }
    }

    pub fn empty() -> Self {
        Self::new(Dimension::new(0, 0), VecDeque::new())
    }

    pub fn push_back(&mut self, cell: Cell, measurement: Measurements) {
        self.dim = self.dim.horizontal_union(measurement.dim);
        self.cells.push_back((cell, measurement));
    }

    pub fn push_front(&mut self, cell: Cell, measurement: Measurements) {
        self.dim = self.dim.horizontal_union(measurement.dim);
        self.cells.push_front((cell, measurement));
    }

    pub fn pop_back(&mut self) -> Option<(Cell, Measurements)> {
        let result = self.cells.pop_back();
        if result.is_some() {
            self.update_dim();
        }
        result
    }

    pub fn pop_front(&mut self) -> Option<(Cell, Measurements)> {
        let result = self.cells.pop_front();
        if result.is_some() {
            self.update_dim();
        }
        result
    }

    fn update_dim(&mut self) {
        self.dim = self
            .cells
            .iter()
            .map(|(_, m)| m.dim)
            .fold(Dimension::empty(), |a, b| a.horizontal_union(b));
    }

    pub fn fixiate_cell_dims(&mut self) {
        self.cells.iter_mut().for_each(|(c, m)| {
            m.dim.height = self.dim.height;
            let mut content_dim = m.specifics.child().map(|m| m.dim).unwrap_or(m.dim);
            if matches!(c.anchor, CellAnchor::Fill) {
                content_dim = Dimension::new(
                    max(content_dim.width, m.dim.width),
                    max(content_dim.height, m.dim.height),
                );
                m.specifics = MeasurementSpecifics::Child(Box::new(content_dim.into()));
            }
            c.dim = CellDimension::Fixed {
                cell: m.dim,
                content: content_dim,
            }
        })
    }
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
}

impl From<Row> for VecDeque<(Cell, Measurements)> {
    fn from(value: Row) -> Self {
        value.cells
    }
}
