use crate::core::measurements::MeasurementSpecifics;
use crate::{Dimension, MeasureMode, Measurements, RcLayout, WrapMode};

/// Defines the dimension of the content of a [`Cell`].
/// The dimension might be a plain fixed [`Dimension`], which provides concrete values for
/// the width and height, or it might be a [`CellWidth`] that describes how the width is computed.
///
/// Instances of this enum can be created using the `From` trait implementation,
/// where the source is either a [`Dimension`] or a [`CellWidth`].
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CellDimension {
    /// The dimension of the cell and its content is fixed and defined by the given [`Dimension`].
    Fixed {
        /// The [`Dimension`] of the cell.
        cell: Dimension,
        /// The [`Dimension`] of the content.
        content: Dimension,
    },

    /// The dimension of the cell is computed based on the given [`CellWidth`]. The exact sizing
    /// behavior depends on the usage.
    Declarative(CellWidth),
}

impl CellDimension {
    /// Returns the [`Dimension`]s of the cell and content, if known.
    /// For the `Fixed` variant it returns the `Dimension`s, otherwise `None`
    ///
    /// # Returns
    /// The `Dimension`s (cell, content) or `None`
    #[must_use]
    pub fn dims(&self) -> Option<(Dimension, Dimension)> {
        match self {
            CellDimension::Fixed { cell, content } => Some((*cell, *content)),
            CellDimension::Declarative(_) => None,
        }
    }

    /// Returns the declarative [`CellWidth`] of the dimension.
    /// If the dimension is fixed, returns `CellWidth::Fixed` with the fixed dimension's width
    ///
    /// # Returns
    /// The [`CellWidth`]
    ///
    /// # Example
    /// ```rust
    ///
    /// use termlayout::Dimension;
    /// use termlayout::widgets::{CellDimension, CellWidth};
    ///
    /// let dim = CellDimension::Declarative(CellWidth::Minimal);
    /// assert_eq!(dim.cell_width(), CellWidth::Minimal);
    ///
    /// let dim = CellDimension::Fixed{cell: Dimension::new(17, 13), content: Dimension::new(11, 9)};
    /// assert_eq!(dim.cell_width(), CellWidth::Fixed(17));
    /// ```
    #[must_use]
    pub fn cell_width(&self) -> CellWidth {
        match self {
            CellDimension::Fixed { cell, content } => {
                if cell.width >= content.width {
                    CellWidth::Fixed(cell.width)
                } else {
                    CellWidth::Preferred(cell.width)
                }
            }
            CellDimension::Declarative(width) => *width,
        }
    }

    /// Checks, whether this instance is a fill dimension
    ///
    /// # Returns
    /// `true`, if this is a `CellDimension::Declarative(CellWidth::Fill)`
    #[must_use]
    pub fn is_fill(&self) -> bool {
        matches!(self, CellDimension::Declarative(CellWidth::Fill))
    }

    /// Calculates the cell and content [`Dimension`] for the given `content`, `max_width`, and
    /// `wrap_mode`.
    ///
    /// # Parameters
    /// - `content`: The [`RcLayout`] of the cell content being laid out with this
    ///   [`CellWidth`].
    /// - `max_width`: The maximum width in terms of columns; if set to `None`, the dimension is
    ///   calculated based on the minimum size of the content.
    /// - `wrap_mode`: The default [`WrapMode`] that is used in case the cell's wrap mode is not set.
    ///
    /// # Returns
    /// The calculated [`Dimension`] for the cell and content.
    #[must_use]
    #[deprecated(since = "0.1.1", note = "Use `measure()` instead")]
    pub fn calculate_dims(
        &self,
        content: &RcLayout,
        max_width: Option<usize>,
        wrap_mode: WrapMode,
    ) -> (Dimension, Dimension) {
        match self {
            CellDimension::Fixed { cell, content } => (*cell, *content),
            CellDimension::Declarative(width) => {
                width.calculate_dims(content, max_width, wrap_mode)
            }
        }
    }

    /// Calculates the [`Measurements`] for this instance.
    /// If this is the `Fixed` variant, it can automatically derive the dimensions from this
    /// instance, otherwise it delegates to the [`CellWidth` implementation](CellWidth::measure)
    ///
    /// # Parameters
    /// - `cell_content`: The [`RcLayout`] representing the cell's content
    /// - `mode`: The [`MeasureMode`] defining how to measure the cell's dimension.
    ///
    /// # Returns
    /// The [`Measurement`] for the cell, which has a child representing the `Measurements` of the
    /// content.
    #[must_use]
    pub fn measure(&self, cell_content: &RcLayout, mode: MeasureMode) -> Measurements {
        match self {
            CellDimension::Fixed { cell, content } => {
                let inner = cell_content.measure(MeasureMode::exact(*content, mode.wrap_mode()));
                Measurements::new(*cell, MeasurementSpecifics::Child(Box::new(inner)))
            }
            CellDimension::Declarative(width) => {
                width.measure(cell_content, mode)
            }
        }
    }
}

impl From<Dimension> for CellDimension {
    fn from(dim: Dimension) -> Self {
        CellDimension::Fixed {
            cell: dim,
            content: dim,
        }
    }
}

impl From<CellWidth> for CellDimension {
    fn from(width: CellWidth) -> Self {
        CellDimension::Declarative(width)
    }
}

/// Defines the width of the content of a [`Cell`].
/// Using this enum, it is possible to define the width of the cell's content as a fixed value or a
/// dynamic one that depends on the available space and usage.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum CellWidth {
    /// The width of the cell content is the same as the minimum width of its content, as
    /// returned by [`min_dim()`](Layout::min_dim)
    #[default]
    Minimal,

    /// The width of the cell content is exactly the width specified with `value`.
    Fixed(usize),

    /// The width of the cell content is the same as the preferred width of the content,
    /// as returned by [`pref_dim()`](Layout::pref_dim) for the given `value`. Note that this is a
    /// subtile way different from `Fixed`, since the actual width might be smaller than `value`.
    Preferred(usize),

    /// The width of the cell content is proportional to the width of the
    /// entire available width. It depends on the usage what the entire available width is.
    /// The actual width is `value` times the width of the available width, so if - for example - the
    /// available width has a width of 100, and the cell is set to `Proportional(0.5)`, the cell
    /// will have a width of 50.
    Proportional(f32),

    /// The width of the cell content is calculated so that it fills the available width.
    /// Depending on the usage - for example in a table - the available width is the the width of
    /// a table row without the widths of the other cells in the row, devided by the number of cells
    /// with the `Fill` width.
    Fill,
}

impl CellWidth {
    /// Calculates the cell and content [`Dimension`] for the given `content`, `max_width`, and
    /// `wrap_mode`.
    ///
    /// # Parameters
    /// - `content`: The [`RcLayout`] of the cell content being laid out with this
    ///   [`CellWidth`].
    /// - `max_width`: The maximum width in terms of columns; if set to `None`, the dimension is
    ///   calculated based on the minimum size of the content.
    /// - `wrap_mode`: The default [`WrapMode`] that is used in case the cell's wrap mode is not set.
    ///
    /// # Returns
    /// The calculated [`Dimension`] for the cell and content.
    #[allow(clippy::missing_panics_doc)]
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_precision_loss)]
    #[deprecated(since = "0.1.1", note = "Use `measure()` instead")]
    #[must_use]
    pub fn calculate_dims(
        &self,
        content: &RcLayout,
        max_width: Option<usize>,
        wrap_mode: WrapMode,
    ) -> (Dimension, Dimension) {
        match self {
            CellWidth::Fixed(w) => {
                let dim = content.pref_dim_fixed_width(*w, wrap_mode);
                (dim, dim)
            }
            CellWidth::Proportional(p) if max_width.is_some() => {
                let w = (max_width.unwrap() as f32 * p) as usize;
                let content = content.pref_dim(w, wrap_mode);
                let cell = Dimension::new(w, content.height);
                (cell, content)
            }
            CellWidth::Fill if max_width.is_some() => {
                let w = max_width.unwrap();
                let content = content.pref_dim(w, wrap_mode);
                let cell = Dimension::new(w, content.height);
                (cell, content)
            }
            CellWidth::Preferred(w) => {
                let content = content.pref_dim(*w, wrap_mode);
                let cell = Dimension::new(*w, content.height);
                (cell, content)
            }
            _ => {
                let dim = content.min_dim();
                (dim, dim)
            }
        }
    }

    /// Calculates the [`Measurements`] for the given `cell_content` and `mode`.
    ///
    /// The method first calculates the `Measurements` of the `cell_content`, mainly based on
    /// the concrete `CellWidth` variant and - if available - the `width` information of the `mode`.
    /// Based on this `Measurement` for the content and the `mode` it determines the `Measurements`
    /// of the cell.
    ///
    /// This means that the `CellWidth` variant influences the content's dimension and the final
    /// cell size depends on the `mode` and optional the dimension of the content's dimension.
    ///
    /// # Parameters
    /// - `cell_content`: The [`RcLayout`] representing the cell's content
    /// - `mode`: The [`MeasureMode`] defining how to measure the cell's dimension.
    ///
    /// # Returns
    /// The [`Measurement`] for the cell, which has a child representing the `Measurements` of the
    /// content.
    pub fn measure(&self, cell_content: &RcLayout, mode: MeasureMode) -> Measurements {
        let inner = match (self, mode.width()) {
            (CellWidth::Fixed(w), _) => {
                cell_content.measure(MeasureMode::fixed_width(*w, mode.wrap_mode()))
            }
            (CellWidth::Preferred(w), _) => {
                cell_content.measure(MeasureMode::pref_width(*w, mode.wrap_mode()))
            }
            (CellWidth::Proportional(p), Some(max_width)) => {
                let w = (max_width as f32 * p) as usize;
                cell_content.measure(MeasureMode::pref_width(w, mode.wrap_mode()))
            }
            (CellWidth::Fill, Some(w)) => {
                cell_content.measure(MeasureMode::pref_width(w, mode.wrap_mode()))
            }
            _ => cell_content.measure(MeasureMode::Min),
        };
        Measurements::new(
            mode.coerce_dim(inner.dim),
            MeasurementSpecifics::Child(Box::new(inner)),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{Dimension, MeasureMode, RcLayout, WrapMode};
    use crate::widgets::{CellDimension, CellWidth, Paragraph};

    #[test]
    fn cell_width_measure_minimal() {
        let cell_content: RcLayout = Paragraph::left("abc def ghijk").into();

        // Min
        let result = CellWidth::Minimal.measure(&cell_content, MeasureMode::min());
        assert_eq!(result.dim, Dimension::new(5, 3));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(5, 3));

        // pref
        let result = CellWidth::Minimal.measure(&cell_content, MeasureMode::pref_width(4, WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(4, 3));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(5, 3));

        let result = CellWidth::Minimal.measure(&cell_content, MeasureMode::pref_width(5, WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(5, 3));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(5, 3));

        let result = CellWidth::Minimal.measure(&cell_content, MeasureMode::pref_width(8, WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(5, 3));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(5, 3));

        // fixed_width
        let result = CellWidth::Minimal.measure(&cell_content, MeasureMode::fixed_width(4, WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(4, 3));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(5, 3));

        let result = CellWidth::Minimal.measure(&cell_content, MeasureMode::fixed_width(5, WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(5, 3));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(5, 3));

        let result = CellWidth::Minimal.measure(&cell_content, MeasureMode::fixed_width(8, WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(8, 3));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(5, 3));

        // exact
        let result = CellWidth::Minimal.measure(&cell_content, MeasureMode::exact(Dimension::new(4,2), WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(4, 2));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(5, 3));

        let result = CellWidth::Minimal.measure(&cell_content, MeasureMode::exact(Dimension::new(5,3), WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(5, 3));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(5, 3));

        let result = CellWidth::Minimal.measure(&cell_content, MeasureMode::exact(Dimension::new(6,4), WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(6, 4));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(5, 3));
    }

    #[test]
    fn cell_width_measure_fixed() {
        let cell_content: RcLayout = Paragraph::left("abc def ghijk").into();

        // Min
        let result = CellWidth::Fixed(8).measure(&cell_content, MeasureMode::min());
        assert_eq!(result.dim, Dimension::new(8, 2));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(8, 2));

        // pref
        let result = CellWidth::Fixed(8).measure(&cell_content, MeasureMode::pref_width(6, WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(6, 2));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(8, 2));

        let result = CellWidth::Fixed(8).measure(&cell_content, MeasureMode::pref_width(7, WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(7, 2));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(8, 2));

        let result = CellWidth::Fixed(8).measure(&cell_content, MeasureMode::pref_width(8, WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(8, 2));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(8, 2));

        // fixed_width
        let result = CellWidth::Fixed(8).measure(&cell_content, MeasureMode::fixed_width(6, WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(6, 2));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(8, 2));

        let result = CellWidth::Fixed(8).measure(&cell_content, MeasureMode::fixed_width(7, WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(7, 2));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(8, 2));

        let result = CellWidth::Fixed(8).measure(&cell_content, MeasureMode::fixed_width(8, WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(8, 2));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(8, 2));

        // exact
        let result = CellWidth::Fixed(8).measure(&cell_content, MeasureMode::exact(Dimension::new(6,2), WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(6, 2));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(8, 2));

        let result = CellWidth::Fixed(8).measure(&cell_content, MeasureMode::exact(Dimension::new(7,3), WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(7, 3));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(8, 2));

        let result = CellWidth::Fixed(8).measure(&cell_content, MeasureMode::exact(Dimension::new(8,4), WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(8, 4));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(8, 2));

    }

    #[test]
    fn cell_width_measure_preferred() {
        let cell_content: RcLayout = Paragraph::left("abc def ghijk").into();

        // Min
        let result = CellWidth::Preferred(8).measure(&cell_content, MeasureMode::min());
        assert_eq!(result.dim, Dimension::new(7, 2));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(7, 2));

        // pref
        let result = CellWidth::Preferred(8).measure(&cell_content, MeasureMode::pref_width(6, WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(6, 2));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(7, 2));

        let result = CellWidth::Preferred(8).measure(&cell_content, MeasureMode::pref_width(7, WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(7, 2));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(7, 2));

        let result = CellWidth::Preferred(8).measure(&cell_content, MeasureMode::pref_width(8, WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(7, 2));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(7, 2));

        // fixed_width
        let result = CellWidth::Preferred(8).measure(&cell_content, MeasureMode::fixed_width(6, WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(6, 2));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(7, 2));

        let result = CellWidth::Preferred(8).measure(&cell_content, MeasureMode::fixed_width(7, WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(7, 2));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(7, 2));

        let result = CellWidth::Preferred(8).measure(&cell_content, MeasureMode::fixed_width(8, WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(8, 2));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(7, 2));

        // exact
        let result = CellWidth::Preferred(8).measure(&cell_content, MeasureMode::exact(Dimension::new(6,2), WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(6, 2));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(7, 2));

        let result = CellWidth::Preferred(8).measure(&cell_content, MeasureMode::exact(Dimension::new(7,3), WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(7, 3));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(7, 2));

        let result = CellWidth::Preferred(8).measure(&cell_content, MeasureMode::exact(Dimension::new(8,4), WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(8, 4));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(7, 2));

    }

    #[test]
    fn cell_width_measure_fill() {
        let cell_content: RcLayout = Paragraph::left("abc def ghijk").into();

        // Min
        let result = CellWidth::Fill.measure(&cell_content, MeasureMode::min());
        assert_eq!(result.dim, Dimension::new(5, 3));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(5, 3));

        // pref
        let result = CellWidth::Fill.measure(&cell_content, MeasureMode::pref_width(6, WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(5, 3));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(5, 3));

        let result = CellWidth::Fill.measure(&cell_content, MeasureMode::pref_width(7, WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(7, 2));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(7, 2));

        let result = CellWidth::Fill.measure(&cell_content, MeasureMode::pref_width(8, WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(7, 2));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(7, 2));

        // fixed_width
        let result = CellWidth::Fill.measure(&cell_content, MeasureMode::fixed_width(6, WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(6, 3));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(5, 3));

        let result = CellWidth::Fill.measure(&cell_content, MeasureMode::fixed_width(7, WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(7, 2));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(7, 2));

        let result = CellWidth::Fill.measure(&cell_content, MeasureMode::fixed_width(8, WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(8, 2));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(7, 2));

        // exact
        let result = CellWidth::Fill.measure(&cell_content, MeasureMode::exact(Dimension::new(6,2), WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(6, 2));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(5, 3));

        let result = CellWidth::Fill.measure(&cell_content, MeasureMode::exact(Dimension::new(7,3), WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(7, 3));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(7, 2));

        let result = CellWidth::Fill.measure(&cell_content, MeasureMode::exact(Dimension::new(8,4), WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(8, 4));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(7, 2));
    }

    #[test]
    fn cell_width_measure_proportional() {
        let cell_content: RcLayout = Paragraph::left("abc def ghijk").into();

        // Min
        let result = CellWidth::Proportional(1.5).measure(&cell_content, MeasureMode::min());
        assert_eq!(result.dim, Dimension::new(5, 3));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(5, 3));

        // pref
        let result = CellWidth::Proportional(1.5).measure(&cell_content, MeasureMode::pref_width(6, WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(6, 2));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(7, 2));

        let result = CellWidth::Proportional(1.5).measure(&cell_content, MeasureMode::pref_width(7, WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(7, 2));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(7, 2));

        let result = CellWidth::Proportional(1.5).measure(&cell_content, MeasureMode::pref_width(8, WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(7, 2));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(7, 2));

        // fixed_width
        let result = CellWidth::Proportional(1.5).measure(&cell_content, MeasureMode::fixed_width(6, WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(6, 2));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(7, 2));

        let result = CellWidth::Proportional(1.5).measure(&cell_content, MeasureMode::fixed_width(7, WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(7, 2));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(7, 2));

        let result = CellWidth::Proportional(1.5).measure(&cell_content, MeasureMode::fixed_width(8, WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(8, 2));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(7, 2));

        // exact
        let result = CellWidth::Proportional(1.5).measure(&cell_content, MeasureMode::exact(Dimension::new(6,2), WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(6, 2));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(7, 2));

        let result = CellWidth::Proportional(1.5).measure(&cell_content, MeasureMode::exact(Dimension::new(7,3), WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(7, 3));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(7, 2));

        let result = CellWidth::Proportional(1.5).measure(&cell_content, MeasureMode::exact(Dimension::new(8,4), WrapMode::default()));
        assert_eq!(result.dim, Dimension::new(8, 4));
        assert_eq!(result.specifics.child().unwrap().dim, Dimension::new(7, 2));
    }
}