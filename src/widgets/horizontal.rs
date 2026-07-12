use crate::widgets::Cell;
use crate::widgets::horizontal::row::Row;
use crate::widgets::vertical::FormattedVertical;
use crate::{
    BoxedFormattedLayout, Dimension, Layout, LayoutOptions, RcLayout, Rect, WrapMode, rc_layout,
};
use std::any::Any;
use std::borrow::Cow;
use std::cmp::max;

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

    /// Computes the concrete dimensions of each [`Cell`] in the horizontal layout.
    /// The dimensions are computed in two steps:
    ///
    /// In a first step, for all `Cells` not having a declarative [`CellWidth`] set to `Fill`
    /// (so `cell.dim != CellDimension::Declarative(CellWidth::Fill)`): For these, the cell width
    /// can be computed directly based on the `max_width` parameter or the cell itself, without
    /// the need to take other cell dimensions into account. If `max_width` is `None` and the size
    /// of the cell depends on the `max_width` (such as a `CellWidth::Propertional(f64)`), then
    /// the minimum dimension is used.
    ///
    /// In a second step, the size of the "fill" cells is computed. This is done by taking the
    /// `max_width` and the size of the already computed cell sizes into account. "fill" columns
    /// are sized so that the remaining space not covered by the other cells is distributed equally
    /// among the "fill" cells. If the `max_width` is `None`, then the minimum dimension is used.
    /// If the other cells already require more space than `max_width`, then the "fill" cells are
    /// sized so that the next multiple of `max_width` is reached.
    ///
    /// # Parameters
    /// - `cells`: The list of [`Cell`]s to compute the sizes for
    /// - `max_width`: The maximum width available for the cells, or `None` if no maximum is specified
    /// - `wrap_mode`: The default [`WrapMode`] to use for cells that do not specify their own
    ///
    /// # Returns
    /// An [`Vec<(Dimension, Dimension)>`] containing the computed dimensions for each cell, the
    /// first dimension is the cell dimension, the second dimension is the content dimension.
    #[must_use]
    pub(crate) fn compute_fixed_dims(
        cells: &[Cell],
        max_width: Option<usize>,
        wrap_mode: WrapMode,
    ) -> Vec<(Dimension, Dimension)> {
        // 1. Step: Compute the preferred dimensions of each cell, if width != Fill
        let mut result: Vec<(Dimension, Dimension)> = Vec::with_capacity(cells.len());
        let mut fill_count = 0;
        let mut fixed_width = 0;
        for cell in cells {
            let dims = if cell.dim.is_fill() && max_width.is_some() {
                fill_count += 1;
                (Dimension::empty(), Dimension::empty())
            } else {
                cell.calculate_dims(max_width, wrap_mode)
            };
            fixed_width += dims.0.width;
            result.push(dims);
        }

        // 2. Step: Compute the dimensions of those cells with width == Fill
        if fill_count > 0
            && let Some(max_width) = max_width
        {
            let mut fill_width = if fixed_width + fill_count > max_width {
                max_width - fixed_width % max_width
            } else {
                max_width - fixed_width
            };
            for (index, cell) in cells.iter().enumerate() {
                if cell.dim.is_fill() {
                    let w = max(1, fill_width / fill_count);
                    let dims = cell.calculate_dims(Some(w), wrap_mode);
                    fill_width -= dims.0.width;
                    fill_count -= 1;
                    result[index] = dims;
                }
            }
        }
        result
    }

    fn apply_fixed_dims(
        cells: &[Cell],
        max_width: Option<usize>,
        wrap_mode: WrapMode,
    ) -> Vec<Cell> {
        let dims = Self::compute_fixed_dims(cells, max_width, wrap_mode);
        cells
            .iter()
            .zip(dims.iter())
            .map(|(cell, dim)| cell.clone().with_dims(dim.0, dim.1))
            .collect()
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
    fn pref_dim(&self, max_width: usize, wrap_mode: WrapMode) -> Dimension {
        if self.content.is_empty() || max_width == 0 {
            return Dimension::empty();
        }

        let cells = self.cells();
        let rows = Row::from_cells(
            Self::apply_fixed_dims(&cells, Some(max_width), wrap_mode),
            max_width,
            wrap_mode,
        );
        rows.iter()
            .fold(Dimension::empty(), |acc, row| acc.vertical_union(row.dim))
    }

    fn min_dim(&self) -> Dimension {
        if self.content.is_empty() {
            return Dimension::empty();
        }
        let cells = self.cells();
        Self::compute_fixed_dims(cells.as_ref(), None, WrapMode::default())
            .iter()
            .fold(Dimension::empty(), |acc, dim| acc.horizontal_union(dim.0))
    }

    fn layout_strict(&'_ self, options: LayoutOptions) -> BoxedFormattedLayout<'_> {
        let cells = self.cells();
        let rows = Row::from_cells(
            Self::apply_fixed_dims(&cells, Some(options.dim.width), options.wrap_mode),
            options.dim.width,
            options.wrap_mode,
        );

        if rows.len() == 1 {
            return rows[0].layout(options);
        }

        let mut offset = 0;
        let formatted = rows
            .into_iter()
            .map(|row| {
                let row_options = options.intersect(Rect::new(0, offset, row.dim), false);
                offset += row.dim.height;
                row.layout(row_options)
            })
            .collect();
        FormattedVertical::new(formatted, options.with_normalized_clip()).into()
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
    use crate::{Dimension, Layout, LayoutOptions, Rect, WrapMode};

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
    fn horizontal_min_dim() {
        // No spacer
        let horizontal = Horizontal::new(sample_cells(), None);
        assert_eq!(horizontal.min_dim(), Dimension::new(14, 5));

        // With spacer
        let horizontal = Horizontal::new(sample_cells(), Some(Filler::once(" foo ").into()));
        assert_eq!(horizontal.min_dim(), Dimension::new(24, 5));
    }

    #[test]
    fn horizontal_pref_dim() {
        // No spacer
        let horizontal = Horizontal::new(sample_cells(), None);
        assert_eq!(
            horizontal.pref_dim(20, WrapMode::Wrap),
            Dimension::new(14, 5)
        );
        assert_eq!(
            horizontal.pref_dim(10, WrapMode::Wrap),
            Dimension::new(10, 10)
        );
        assert_eq!(
            horizontal.pref_dim(10, WrapMode::default_truncate()),
            Dimension::new(10, 4)
        );

        // With spacer
        let horizontal = Horizontal::new(sample_cells(), Some(Filler::once(" foo ").into()));
        assert_eq!(horizontal.min_dim(), Dimension::new(24, 5));
        assert_eq!(
            horizontal.pref_dim(20, WrapMode::Wrap),
            Dimension::new(20, 10)
        );
        assert_eq!(
            horizontal.pref_dim(10, WrapMode::Wrap),
            Dimension::new(10, 14)
        );
        assert_eq!(
            horizontal.pref_dim(10, WrapMode::default_truncate()),
            Dimension::new(10, 4)
        );
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
                "             |              |                          ABCDE\n",
                "   abcdef    | 123          |                          FGHIJ\n",
                "   ghijkl    | 456          |                          KLMNO\n",
                "   mnopqr    | 789          |                          PQRST\n",
                "   stuvwx    |              |                          UVWXY\n",
                "                              \n"
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
                "               \n"
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
                "                         \n"
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
                "              \n"
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
                "               \n"
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
                "                 \n"   //
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
                "                  \n"  //
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
                "               \n"  //
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
                "               \n"  //
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
                "\n"
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
