use crate::widgets::{Cell, CellAnchor, CellDimension};
use crate::{Dimension, RcLayout, WrapMode};
use std::any::Any;
use std::cmp::max;
use std::collections::VecDeque;
use std::rc::Rc;

#[derive(Debug, Clone, Copy)]
pub enum MeasureMode {
    Min,
    PrefWidth {
        max_width: usize,
        wrap_mode: WrapMode,
    },
    FixedWidth {
        width: usize,
        wrap_mode: WrapMode,
    },
    Exact {
        dimension: Dimension,
        wrap_mode: WrapMode,
    },
}

impl MeasureMode {
    pub const fn min() -> Self {
        Self::Min
    }

    pub const fn pref_width(max_width: usize, wrap_mode: WrapMode) -> Self {
        Self::PrefWidth {
            max_width,
            wrap_mode,
        }
    }

    pub const fn fixed_width(width: usize, wrap_mode: WrapMode) -> Self {
        Self::FixedWidth { width, wrap_mode }
    }

    pub const fn exact(dimension: Dimension, wrap_mode: WrapMode) -> Self {
        Self::Exact {
            dimension,
            wrap_mode,
        }
    }

    pub fn wrap_mode(&self) -> WrapMode {
        match self {
            Self::Min => WrapMode::default(),
            Self::PrefWidth { wrap_mode, .. } => *wrap_mode,
            Self::FixedWidth { wrap_mode, .. } => *wrap_mode,
            Self::Exact { wrap_mode, .. } => *wrap_mode,
        }
    }

    pub fn width(&self) -> Option<usize> {
        match self {
            Self::Min => None,
            Self::PrefWidth { max_width, .. } => Some(*max_width),
            Self::FixedWidth { width, .. } => Some(*width),
            Self::Exact { dimension, .. } => Some(dimension.width),
        }
    }

    pub fn coerce_width(&self, width: usize) -> usize {
        match self {
            MeasureMode::Min => width,
            MeasureMode::PrefWidth { max_width, .. } => max(*max_width, width),
            MeasureMode::FixedWidth { width, .. } => *width,
            MeasureMode::Exact { dimension, .. } => dimension.width,
        }
    }

    pub fn height(&self) -> Option<usize> {
        match self {
            Self::Exact { dimension, .. } => Some(dimension.height),
            _ => None,
        }
    }

    pub fn coerce_height(&self, height: usize) -> usize {
        self.height().unwrap_or(height)
    }

    pub fn coerce_dim(&self, dim: Dimension) -> Dimension {
        Dimension {
            width: self.coerce_width(dim.width),
            height: self.coerce_height(dim.height),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            MeasureMode::Min => false,
            MeasureMode::PrefWidth { max_width, .. } => *max_width == 0,
            MeasureMode::FixedWidth { width, .. } => *width == 0,
            MeasureMode::Exact { dimension, .. } => dimension.is_empty(),
        }
    }
}

#[derive(Clone)]
pub struct Measurements {
    pub dim: Dimension,
    pub specifics: MeasurementSpecifics,
}

impl Measurements {
    pub fn new(dim: Dimension, specifics: MeasurementSpecifics) -> Self {
        Self { dim, specifics }
    }

    pub fn empty() -> Self {
        Self {
            dim: Dimension::empty(),
            specifics: MeasurementSpecifics::None,
        }
    }

    pub fn with_specifics(self, specifics: MeasurementSpecifics) -> Self {
        Self {
            dim: self.dim,
            specifics,
        }
    }

    pub fn split_horizontal(self, width: usize) -> (Self, Self) {
        let (left, right) = self.dim.split_horizontal(width);
        (
            Self::new(left, self.specifics.clone()),
            Self::new(right, self.specifics),
        )
    }

    pub fn split_vertical(self, height: usize) -> (Self, Self) {
        let (top, bottom) = self.dim.split_vertical(height);
        (
            Self::new(top, self.specifics.clone()),
            Self::new(bottom, self.specifics),
        )
    }

    pub fn is_empty(&self) -> bool {
        self.dim.is_empty()
    }

    pub fn fold_vertically<'a>(
        iterator: impl Iterator<Item = &'a RcLayout>,
        mode: MeasureMode,
    ) -> Self {
        // First figure out all default measurements, they might differ regarding width
        let children: Vec<(RcLayout, Measurements)> = iterator
            .map(|layout| (layout.clone(), layout.measure(mode)))
            .collect();

        // Compute the overall dimension
        let dim = children.iter().fold(Dimension::empty(), |acc, child| {
            acc.vertical_union(child.1.dim)
        });

        // And finally adjust the width of all items to the same
        let children = children
            .into_iter()
            .map(|(layout, measurement)| {
                if measurement.dim.width != dim.width {
                    layout.measure(MeasureMode::exact(
                        Dimension::new(dim.width, measurement.dim.height),
                        mode.wrap_mode(),
                    ))
                } else {
                    measurement
                }
            })
            .collect();
        Self::new(dim, MeasurementSpecifics::Children(children))
    }
}

impl From<Dimension> for Measurements {
    fn from(value: Dimension) -> Self {
        Self::new(value, MeasurementSpecifics::None)
    }
}

#[derive(Clone)]
pub enum MeasurementSpecifics {
    None,
    Children(Vec<Measurements>),
    Child(Box<Measurements>),
    Rows(Vec<Row>),
    Custom(Rc<dyn Any>),
}

impl MeasurementSpecifics {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn children(&self) -> Option<&[Measurements]> {
        match self {
            MeasurementSpecifics::Children(children) => Some(children),
            _ => None,
        }
    }

    pub fn child(&self) -> Option<&Measurements> {
        match self {
            MeasurementSpecifics::Child(child) => Some(child.as_ref()),
            _ => None,
        }
    }

    pub fn rows(&self) -> Option<&[Row]> {
        match self {
            MeasurementSpecifics::Rows(rows) => Some(rows),
            _ => None,
        }
    }
}

impl TryFrom<MeasurementSpecifics> for Vec<Row> {
    type Error = &'static str;

    fn try_from(value: MeasurementSpecifics) -> Result<Self, Self::Error> {
        match value {
            MeasurementSpecifics::Rows(rows) => Ok(rows),
            _ => Err("MeasurementSpecifics is not Rows"),
        }
    }
}

impl TryFrom<MeasurementSpecifics> for Measurements {
    type Error = &'static str;

    fn try_from(value: MeasurementSpecifics) -> Result<Self, Self::Error> {
        match value {
            MeasurementSpecifics::Child(child) => Ok(*child),
            _ => Err("MeasurementSpecifics is not Child"),
        }
    }
}

impl TryFrom<MeasurementSpecifics> for Vec<Measurements> {
    type Error = &'static str;

    fn try_from(value: MeasurementSpecifics) -> Result<Self, Self::Error> {
        match value {
            MeasurementSpecifics::Children(children) => Ok(children),
            _ => Err("MeasurementSpecifics is not Children"),
        }
    }
}

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
