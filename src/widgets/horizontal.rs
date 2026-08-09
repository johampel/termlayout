use crate::widgets::Cell;
use crate::widgets::horizontal::row::Row;
use crate::{
    BoxedFormattedLayout, Layout, LayoutContext, MeasureMode, MeasurementSpecifics, Measurements,
    RcLayout, rc_layout,
};
use std::any::Any;
use std::borrow::Cow;

pub(crate) mod metrics;
pub(crate) mod row;

/// A widget that arranges cells horizontally in a row.
///
/// The children are [`Cell`]s, so for each child the size and positioning can be specified
/// independently. An optional spacer can be added between the children.
///
/// # Example
/// ```rust
/// use termlayout::Layout;
/// use termlayout::widgets::{Cell, Filler, Horizontal, Lines};
///
/// let left = Cell::of(Lines::left("left side"));
/// let right = Cell::of(Lines::left("right side"));
///
/// let horizontal = Horizontal::new(vec![left, right], Some(Filler::vertical("|").into()));
///
/// assert_eq!(format!("{}", horizontal.layout(20)), concat!(
///     "left side|right side\n"
/// ));
/// ```
pub struct Horizontal {
    /// The list of [`Cell`]s to arrange in the horizontal row
    pub content: Vec<Cell>,

    /// The [`Layout`] to use as a spacer between the children, or `None` if no spacer should be used
    pub spacer: Option<RcLayout>,
}

impl Horizontal {
    /// Creates a new instance.
    ///
    /// # Parameters
    /// - `content`: The list of [`Cell`]s to arrange in the horizontal row
    /// - `spacer`: The [`Layout`] to use as a spacer between the children, or `None` if no spacer
    ///   should be used
    ///
    /// # Returns
    /// A new instance
    pub fn new<T>(content: T, spacer: Option<RcLayout>) -> Self
    where
        T: Into<Vec<Cell>>,
    {
        Self {
            content: content.into(),
            spacer,
        }
    }

    fn cells(&self) -> Cow<'_, [Cell]> {
        if let Some(spacer) = &self.spacer
            && !self.content.is_empty()
        {
            let mut cells = Vec::with_capacity(2 * self.content.len() - 1);
            let spacer = Cell::fill(spacer.clone());
            let mut iter = self.content.iter();
            cells.push(iter.next().unwrap().clone());
            for cell in iter {
                cells.push(spacer.clone());
                cells.push(cell.clone());
            }
            return Cow::Owned(cells);
        }
        Cow::Borrowed(&self.content)
    }
}

impl<T> From<T> for Horizontal
where
    T: Into<Vec<Cell>>,
{
    fn from(value: T) -> Self {
        Self::new(value, None)
    }
}

impl Layout for Horizontal {
    fn measure(&self, mode: MeasureMode) -> Measurements {
        let metrics = metrics::HorizontalMetrics::from_cells(self.cells().as_ref(), mode);
        Measurements::new(metrics.dim, MeasurementSpecifics::Rows(metrics.rows))
    }

    fn layout_with_context(&'_ self, context: LayoutContext) -> BoxedFormattedLayout<'_> {
        match &context.measurements.specifics { 
            MeasurementSpecifics::Rows(_) => Row::layout(context).unwrap(),
            _ => self.layout_strict(context.options)
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

rc_layout!(Horizontal);

#[cfg(test)]
mod tests {
    use crate::widgets::horizontal::Horizontal;
    use crate::widgets::{Cell, CellAnchor, CellDimension, CellWidth, Filler, Lines};
    use crate::{Dimension, Layout, LayoutOptions, MeasureMode, Rect, WrapMode};

    fn sample_cells() -> Vec<Cell> {
        vec![
            Cell::of(Lines::left("abcdef\nghijkl\nmnopqr\nstuvwx")).with_width(CellWidth::Fixed(6)),
            Cell::of(Lines::left("123\n456\n789"))
                .with_width(CellWidth::Fixed(3))
                .with_anchor(CellAnchor::Center),
            Cell::of(Lines::left("ABCDE\nFGHIJ\nKLMNO\nPQRST\nUVWXY"))
                .with_width(CellWidth::Fixed(5)),
        ]
    }

    #[test]
    fn horizontal_measure_min() {
        let mode = MeasureMode::min();

        // No spacer
        let horizontal = Horizontal::new(sample_cells(), None);
        let measurement = horizontal.measure(mode);

        assert_eq!(measurement.dim, Dimension::new(14, 5));
        assert_eq!(measurement.specifics.rows().unwrap().len(), 1);

        // With spacer
        let horizontal = Horizontal::new(sample_cells(), Some(Filler::once(" foo ").into()));
        let measurement = horizontal.measure(mode);

        assert_eq!(measurement.dim, Dimension::new(24, 5));
        assert_eq!(measurement.specifics.rows().unwrap().len(), 1);
    }

    #[test]
    fn horizontal_measure_pref_width() {
        // No spacer
        let horizontal = Horizontal::new(sample_cells(), None);

        let mode = MeasureMode::pref_width(20, WrapMode::Wrap);
        let measurement = horizontal.measure(mode);
        assert_eq!(measurement.dim, Dimension::new(14, 5));
        assert_eq!(measurement.specifics.rows().unwrap().len(), 1);

        let mode = MeasureMode::pref_width(10, WrapMode::Wrap);
        let measurement = horizontal.measure(mode);
        assert_eq!(measurement.dim, Dimension::new(10, 10));
        assert_eq!(measurement.specifics.rows().unwrap().len(), 2);

        // With spacer
        let horizontal = Horizontal::new(sample_cells(), Some(Filler::once(" foo ").into()));

        let mode = MeasureMode::pref_width(20, WrapMode::Wrap);
        let measurement = horizontal.measure(mode);
        assert_eq!(measurement.dim, Dimension::new(20, 10));
        assert_eq!(measurement.specifics.rows().unwrap().len(), 2);

        let mode = MeasureMode::pref_width(10, WrapMode::Wrap);
        let measurement = horizontal.measure(mode);
        assert_eq!(measurement.dim, Dimension::new(10, 14));
        assert_eq!(measurement.specifics.rows().unwrap().len(), 3);
    }

    #[test]
    fn horizontal_layout_fit_no_clip_wide_cells() {
        let mut cells = sample_cells();
        cells[0].dim = CellDimension::Declarative(CellWidth::Preferred(12));
        cells[0].anchor = CellAnchor::Center;
        cells[1].dim = CellDimension::Declarative(CellWidth::Fixed(12));
        cells[1].anchor = CellAnchor::Center;
        cells[2].dim = CellDimension::Declarative(CellWidth::Fill);
        cells[2].anchor = CellAnchor::East;
        let horizontal = Horizontal::new(cells, Some(Filler::vertical(" | ").into()));

        let options = LayoutOptions::new(Dimension::new(60, 6), false, WrapMode::Wrap, None);
        assert_eq!(
            format!("{}", horizontal.layout_strict(options)),
            concat!(
                "       |              |                                ABCDE\n",
                "abcdef | 123          |                                FGHIJ\n",
                "ghijkl | 456          |                                KLMNO\n",
                "mnopqr | 789          |                                PQRST\n",
                "stuvwx |              |                                UVWXY\n",
                "       |              | \n"
            )
        );
    }

    #[test]
    fn horizontal_layout_fit_no_clip() {
        let horizontal = Horizontal::new(sample_cells(), Some(Filler::vertical(" | ").into()));

        let options = LayoutOptions::new(Dimension::new(25, 6), false, WrapMode::Wrap, None);
        assert_eq!(
            format!("{}", horizontal.layout_strict(options)),
            concat!(
                "abcdef |     | ABCDE\n",
                "ghijkl | 123 | FGHIJ\n",
                "mnopqr | 456 | KLMNO\n",
                "stuvwx | 789 | PQRST\n",
                "       |     | UVWXY\n",
                "       |     | \n"
            )
        );

        let options = LayoutOptions::new(Dimension::new(25, 6), true, WrapMode::Wrap, None);
        assert_eq!(
            format!("{}", horizontal.layout_strict(options)),
            concat!(
                "abcdef |     | ABCDE     \n",
                "ghijkl | 123 | FGHIJ     \n",
                "mnopqr | 456 | KLMNO     \n",
                "stuvwx | 789 | PQRST     \n",
                "       |     | UVWXY     \n",
                "       |     |           \n"
            )
        );
    }

    #[test]
    fn horizontal_layout_fit_with_clip() {
        let horizontal = Horizontal::new(sample_cells(), Some(Filler::vertical(" | ").into()));

        let options = LayoutOptions::new(
            Dimension::new(25, 6),
            false,
            WrapMode::Wrap,
            Some(Rect::new(1, 2, Dimension::new(15, 4))),
        );
        assert_eq!(
            format!("{}", horizontal.layout_strict(options)),
            concat!(
                "nopqr | 456 | K\n",
                "tuvwx | 789 | P\n",
                "      |     | U\n",
                "      |     | \n"
            )
        );

        let options = LayoutOptions::new(
            Dimension::new(25, 6),
            true,
            WrapMode::Wrap,
            Some(Rect::new(1, 2, Dimension::new(15, 4))),
        );
        assert_eq!(
            format!("{}", horizontal.layout_strict(options)),
            concat!(
                "nopqr | 456 | K\n",
                "tuvwx | 789 | P\n",
                "      |     | U\n",
                "      |     |  \n"
            )
        );
    }

    #[test]
    fn horizontal_layout_truncate_no_clip() {
        let horizontal = Horizontal::new(sample_cells(), Some(Filler::vertical(" | ").into()));

        let options = LayoutOptions::new(
            Dimension::new(18, 6),
            false,
            WrapMode::default_truncate(),
            None,
        );
        assert_eq!(
            format!("{}", horizontal.layout_strict(options)),
            concat!(
                "abcdef |     | AB…\n", //
                "ghijkl | 123 | FG…\n", //
                "mnopqr | 456 | KL…\n", //
                "stuvwx | 789 | PQ…\n", //
                "       |     | UV…\n", //
                "       |     |   …\n"  //
            )
        );

        let options = LayoutOptions::new(
            Dimension::new(18, 6),
            true,
            WrapMode::default_truncate(),
            None,
        );
        assert_eq!(
            format!("{}", horizontal.layout_strict(options)),
            concat!(
                "abcdef |     | AB…\n", //
                "ghijkl | 123 | FG…\n", //
                "mnopqr | 456 | KL…\n", //
                "stuvwx | 789 | PQ…\n", //
                "       |     | UV…\n", //
                "       |     |   …\n"  //
            )
        );
    }

    #[test]
    fn horizontal_layout_truncate_with_clip() {
        let horizontal = Horizontal::new(sample_cells(), Some(Filler::vertical(" | ").into()));

        let options = LayoutOptions::new(
            Dimension::new(18, 6),
            false,
            WrapMode::default_truncate(),
            Some(Rect::new(1, 2, Dimension::new(15, 4))),
        );
        assert_eq!(
            format!("{}", horizontal.layout_strict(options)),
            concat!(
                "nopqr | 456 | K\n", //
                "tuvwx | 789 | P\n", //
                "      |     | U\n", //
                "      |     |  \n"  //
            )
        );

        let options = LayoutOptions::new(
            Dimension::new(18, 6),
            true,
            WrapMode::default_truncate(),
            Some(Rect::new(1, 2, Dimension::new(15, 4))),
        );
        assert_eq!(
            format!("{}", horizontal.layout_strict(options)),
            concat!(
                "nopqr | 456 | K\n", //
                "tuvwx | 789 | P\n", //
                "      |     | U\n", //
                "      |     |  \n"  //
            )
        );
    }

    #[test]
    fn horizontal_layout_wrap_no_clip() {
        let horizontal = Horizontal::new(sample_cells(), Some(Filler::vertical(" | ").into()));

        let options = LayoutOptions::new(Dimension::new(14, 10), false, WrapMode::Wrap, None);
        assert_eq!(
            format!("{}", horizontal.layout_strict(options)),
            concat!(
                "abcdef | 123 |\n",
                "ghijkl | 456 |\n",
                "mnopqr | 789 |\n",
                "stuvwx |     |\n",
                " ABCDE\n",
                " FGHIJ\n",
                " KLMNO\n",
                " PQRST\n",
                " UVWXY\n",
                " \n"
            )
        );

        let options = LayoutOptions::new(Dimension::new(14, 10), true, WrapMode::Wrap, None);
        assert_eq!(
            format!("{}", horizontal.layout_strict(options)),
            concat!(
                "abcdef | 123 |\n",
                "ghijkl | 456 |\n",
                "mnopqr | 789 |\n",
                "stuvwx |     |\n",
                " ABCDE        \n",
                " FGHIJ        \n",
                " KLMNO        \n",
                " PQRST        \n",
                " UVWXY        \n",
                "              \n"
            )
        );
    }

    #[test]
    fn horizontal_layout_wrap_with_clip() {
        let horizontal = Horizontal::new(sample_cells(), Some(Filler::vertical(" | ").into()));

        let options = LayoutOptions::new(
            Dimension::new(14, 10),
            false,
            WrapMode::Wrap,
            Some(Rect::new(1, 2, Dimension::new(9, 6))),
        );
        assert_eq!(
            format!("{}", horizontal.layout_strict(options)),
            concat!(
                "nopqr | 7\n", //
                "tuvwx |  \n", //
                "ABCDE\n",     //
                "FGHIJ\n",     //
                "KLMNO\n",     //
                "PQRST\n",     //
            )
        );

        let options = LayoutOptions::new(
            Dimension::new(14, 10),
            true,
            WrapMode::Wrap,
            Some(Rect::new(1, 2, Dimension::new(9, 6))),
        );
        assert_eq!(
            format!("{}", horizontal.layout_strict(options)),
            concat!(
                "nopqr | 7\n",
                "tuvwx |  \n",
                "ABCDE    \n",
                "FGHIJ    \n",
                "KLMNO    \n",
                "PQRST    \n",
            )
        );
    }
}
