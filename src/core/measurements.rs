use std::any::Any;
use crate::{Dimension, LayoutContext, WrapMode};

pub enum MeasureMode {
    Min,
    Pref {
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

    pub const fn pref(max_width: usize, wrap_mode: WrapMode) -> Self {
        Self::Pref {
            max_width,
            wrap_mode,
        }
    }

    pub const fn fixed_width(width: usize, wrap_mode: WrapMode) -> Self {
        Self::FixedWidth {
            width,
            wrap_mode,
        }
    }

    pub const fn exact(dimension: Dimension, wrap_mode: WrapMode) -> Self {
        Self::Exact { dimension, wrap_mode }
    }
}

pub struct Measurements {
    pub dim: Dimension,
    pub specifics: Option<Box<dyn Any>>,
}

impl Measurements {
    pub fn new(dim: Dimension, specifics: Option<Box<dyn Any>>) -> Self {
        Self { dim, specifics }
    }
}

impl From<Dimension> for Measurements {
    fn from(value: Dimension) -> Self {
        Self::new(value, None)
    }
}
