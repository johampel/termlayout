use crate::ext::Row;
use crate::widgets::table::decoration::DecoratedTable;
use crate::widgets::table::metrics::TableMetrics;
use crate::widgets::{CellAnchor, CellWidth, TableDecoration};
use crate::{
    BoxedFormattedLayout, Dimension, Layout, LayoutContext, MeasureMode, MeasurementSpecifics,
    Measurements, RcLayout, WrapMode, rc_layout,
};
use std::any::Any;

pub(crate) mod decoration;
mod metrics;

/// A widget that displays tabular data with rows and columns.
///
/// A `Table` consists of a [`TableDecoration`] for borders and styling, a list of
/// [`TableColumn`] definitions that specify column headers and properties, and the
/// actual cell content as a 2D vector of [`RcLayout`]s.
///
/// # Example
/// ```rust
/// use termlayout::*;
/// use termlayout::widgets::{Table, TableColumn, TableDecoration, CellWidth, Lines};
///
/// let table = Table::new(
///     TableDecoration::boxed_grid(),
///     vec![
///         TableColumn::default().with_header(Lines::center("Name")).with_width(CellWidth::Minimal),
///         TableColumn::default().with_header(Lines::center("Age")).with_width(CellWidth::Fill),
///     ],
///     vec![
///         vec![Lines::left("Alice").into(), Lines::right("30").into()],
///         vec![Lines::left("Bob").into(), Lines::right("25").into()],
///     ],
/// );
/// ```
pub struct Table {
    /// The decoration to use for the table (e.g., borders).
    pub decoration: TableDecoration,

    /// The list of column definitions.
    pub columns: Vec<TableColumn>,

    /// The actual cells as a 2D vector of [`RcLayout`]s.
    pub cells: Vec<Vec<RcLayout>>,
}

impl Table {
    /// Creates a new `Table` with the specified decoration, columns, and cells.
    ///
    /// # Parameters
    /// - `decoration`: The decoration to use for the table (e.g., borders).
    /// - `columns`: A list of column definitions.
    /// - `cells`: A 2D vector where each inner vector represents a row of cells.
    ///
    /// # Returns
    /// A new `Table` instance.
    #[must_use]
    pub fn new(
        decoration: TableDecoration,
        columns: Vec<TableColumn>,
        cells: Vec<Vec<RcLayout>>,
    ) -> Self {
        Self {
            decoration,
            columns,
            cells,
        }
    }
}

impl Layout for Table {
    fn measure(&self, mode: MeasureMode) -> Measurements {
        if mode.is_empty() {
            return Measurements::empty().with_specifics(MeasurementSpecifics::Rows(vec![]));
        }
        let table = DecoratedTable::new(self);
        let metrics = TableMetrics::new(&table, mode);
        let rows = metrics.all_rows(mode);
        let dim = rows
            .iter()
            .map(|row| row.dim)
            .fold(Dimension::empty(), |acc, dim| acc.vertical_union(dim));
        Measurements::new(dim, MeasurementSpecifics::Rows(rows))
    }

    fn layout_with_context(&'_ self, context: LayoutContext) -> BoxedFormattedLayout<'_> {
        match &context.measurements.specifics {
            MeasurementSpecifics::Rows(_) => Row::layout(context).unwrap(),
            _ => self.layout_strict(context.options),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

rc_layout!(Table);

/// Represents the configuration for a single column in a [`Table`].
pub struct TableColumn {
    /// The optional layout to be used as the column header.
    pub(crate) header: Option<RcLayout>,

    /// The [`CellWidth`] defining the width of the column.
    pub(crate) width: CellWidth,

    /// The [`CellAnchor`] defining how the column is placed within the table.
    pub(crate) anchor: CellAnchor,

    /// The [`WrapMode`] defining how the column's content is wrapped.
    pub(crate) wrap_mode: WrapMode,
}

impl TableColumn {
    /// Constant for the default table column configuration.
    pub const DEFAULT: TableColumn = TableColumn::new(
        None,
        CellWidth::Minimal,
        CellAnchor::NorthWest,
        WrapMode::Wrap,
    );

    /// Creates a new `TableColumn` with the specified parameters.
    ///
    /// # Parameters
    /// - `header`: Optional layout to be used as the column header.
    /// - `width`: The [`CellWidth`]
    /// - `anchor`: The [`CellAnchor`]
    /// - `wrap_mode`: The [`WrapMode`]
    /// # Returns
    /// A new `TableColumn` instance.
    #[must_use]
    pub const fn new(
        header: Option<RcLayout>,
        width: CellWidth,
        anchor: CellAnchor,
        wrap_mode: WrapMode,
    ) -> Self {
        Self {
            header,
            width,
            anchor,
            wrap_mode,
        }
    }

    /// Returns a new `TableColumn` with the specified header.
    ///
    /// # Parameters
    /// - `header`: The layout to be used as the column header.
    ///
    /// # Returns
    /// A new `TableColumn` instance with the updated header.
    #[must_use]
    pub fn with_header(&self, header: impl Into<RcLayout>) -> Self {
        Self {
            header: Some(header.into()),
            ..*self
        }
    }

    /// Returns a new `TableColumn` with the specified width.
    ///
    /// # Parameters
    /// - `width`: The [`CellWidth`]
    ///
    /// # Returns
    /// A new `TableColumn` instance with the updated width.
    #[must_use]
    pub fn with_width(&self, width: CellWidth) -> Self {
        Self {
            width,
            header: self.header.clone(),
            ..*self
        }
    }

    /// Returns a new `TableColumn` with the specified anchor.
    ///
    /// # Parameters
    /// - `anchor`: The [`CellAnchor`]
    ///
    /// # Returns
    /// A new `TableColumn` instance with the updated anchor.
    #[must_use]
    pub fn with_anchor(&self, anchor: CellAnchor) -> Self {
        Self {
            anchor,
            header: self.header.clone(),
            ..*self
        }
    }

    /// Returns a new `TableColumn` with the specified wrap mode.
    ///
    /// # Parameters
    /// - `wrap_mode`: The [`WrapMode`]
    ///
    /// # Returns
    /// A new `TableColumn` instance with the updated wrap mode.
    #[must_use]
    pub fn with_format(&self, wrap_mode: WrapMode) -> Self {
        Self {
            wrap_mode,
            header: self.header.clone(),
            ..*self
        }
    }
}

impl Default for TableColumn {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[cfg(test)]
mod tests {
    use crate::widgets::table::*;
    use crate::widgets::{Filler, Lines};
    use crate::{Dimension, LayoutOptions, MeasureMode, Rect, WrapMode};

    // Helper: creates a single-row, N-column table using headless_no_grid decoration
    // (one space separator between columns, no borders, no header rows).
    fn headless_table(widths: Vec<CellWidth>, cells: Vec<RcLayout>) -> Table {
        let columns = widths
            .into_iter()
            .map(|w| TableColumn::default().with_width(w))
            .collect();
        Table::new(TableDecoration::headless_no_grid(), columns, vec![cells])
    }

    #[test]
    fn table_layout_fit_with_anchor() {
        let table = Table::new(
            TableDecoration::boxed_grid(),
            vec![
                TableColumn::default()
                    .with_header(Lines::center("Col 1"))
                    .with_anchor(CellAnchor::NorthWest)
                    .with_width(CellWidth::Fill),
                TableColumn::default()
                    .with_header(Lines::center("Col 2"))
                    .with_anchor(CellAnchor::Center)
                    .with_width(CellWidth::Fill),
                TableColumn::default()
                    .with_header(Lines::center("Col 3"))
                    .with_anchor(CellAnchor::SouthEast)
                    .with_width(CellWidth::Fill),
            ],
            vec![vec![
                Lines::left("abcde").into(),
                Lines::left("fghij\nklmno\npqrst").into(),
                Lines::left("uvwxy").into(),
            ]],
        );

        let formatted = table.layout_strict(LayoutOptions::new(
            Dimension::new(39, 8),
            true,
            WrapMode::default(),
            None,
        ));

        let result = format!("{formatted}");
        assert_eq!(
            result,
            concat!(
                "┌───────────┬────────────┬────────────┐\n",
                "│Col 1      │   Col 2    │       Col 3│\n",
                "├───────────┼────────────┼────────────┤\n",
                "│abcde      │   fghij    │            │\n",
                "│           │   klmno    │            │\n",
                "│           │   pqrst    │       uvwxy│\n",
                "└───────────┴────────────┴────────────┘\n",
                "                                       \n",
            )
        );
    }

    #[test]
    fn table_layout_fit() {
        let table = Table::new(
            TableDecoration::boxed_grid(),
            vec![
                TableColumn::default()
                    .with_header(Lines::center("Col 1"))
                    .with_anchor(CellAnchor::Fill)
                    .with_width(CellWidth::Minimal),
                TableColumn::default()
                    .with_header(Lines::center("Col 2"))
                    .with_anchor(CellAnchor::Fill)
                    .with_width(CellWidth::Minimal),
                TableColumn::default()
                    .with_header(Lines::center("Col 3"))
                    .with_anchor(CellAnchor::Fill)
                    .with_width(CellWidth::Minimal),
            ],
            vec![
                vec![
                    Lines::left("abcdefghijklm\nnopqrstuvwxyz").into(),
                    Filler::both("01").into(),
                    Lines::left("ABCDEFGHIJKLM\nNOPQRSTUVWXYZ").into(),
                ],
                vec![
                    Filler::both("ab").into(),
                    Lines::left("0123456789").into(),
                    Filler::both("+-").into(),
                ],
            ],
        );

        // No Clip
        let formatted = table.layout_strict(LayoutOptions::new(
            Dimension::new(40, 9),
            true,
            WrapMode::default(),
            None,
        ));
        let result = format!("{formatted}");
        assert_eq!(
            result,
            concat!(
                "┌─────────────┬──────────┬─────────────┐\n",
                "│    Col 1    │  Col 2   │    Col 3    │\n",
                "├─────────────┼──────────┼─────────────┤\n",
                "│abcdefghijklm│0101010101│ABCDEFGHIJKLM│\n",
                "│nopqrstuvwxyz│0101010101│NOPQRSTUVWXYZ│\n",
                "├─────────────┼──────────┼─────────────┤\n",
                "│ababababababa│0123456789│+-+-+-+-+-+-+│\n",
                "└─────────────┴──────────┴─────────────┘\n",
                "                                        \n",
            )
        );

        // With Clip
        let formatted = table.layout_strict(LayoutOptions::new(
            Dimension::new(40, 8),
            true,
            WrapMode::default(),
            Some(Rect::new(2, 1, Dimension::new(30, 5))),
        ));
        let result = format!("{formatted}");
        assert_eq!(
            result,
            concat!(
                "   Col 1    │  Col 2   │    Co\n",
                "────────────┼──────────┼──────\n",
                "bcdefghijklm│0101010101│ABCDEF\n",
                "opqrstuvwxyz│0101010101│NOPQRS\n",
                "────────────┼──────────┼──────\n",
            )
        );
    }

    #[test]
    fn table_layout_truncate() {
        let table = Table::new(
            TableDecoration::boxed_grid(),
            vec![
                TableColumn::default()
                    .with_header(Lines::center("Col 1"))
                    .with_anchor(CellAnchor::Fill)
                    .with_width(CellWidth::Minimal),
                TableColumn::default()
                    .with_header(Lines::center("Col 2"))
                    .with_anchor(CellAnchor::Fill)
                    .with_width(CellWidth::Minimal),
                TableColumn::default()
                    .with_header(Lines::center("Col 3"))
                    .with_anchor(CellAnchor::Fill)
                    .with_width(CellWidth::Minimal),
            ],
            vec![
                vec![
                    Lines::left("abcdefghijklm\nnopqrstuvwxyz").into(),
                    Filler::both("01").into(),
                    Lines::left("ABCDEFGHIJKLM\nNOPQRSTUVWXYZ").into(),
                ],
                vec![
                    Filler::both("ab").into(),
                    Lines::left("0123456789").into(),
                    Filler::both("+-").into(),
                ],
            ],
        );

        // No Clip
        let formatted = table.layout_strict(LayoutOptions::new(
            Dimension::new(34, 9),
            true,
            WrapMode::default_truncate(),
            None,
        ));
        let result = format!("{formatted}");
        assert_eq!(
            result,
            concat!(
                "┌─────────────┬──────────┬───────…\n",
                "│    Col 1    │  Col 2   │ Col 3 …\n",
                "├─────────────┼──────────┼───────…\n",
                "│abcdefghijklm│0101010101│ABCDEFG…\n",
                "│nopqrstuvwxyz│0101010101│NOPQRST…\n",
                "├─────────────┼──────────┼───────…\n",
                "│ababababababa│0123456789│+-+-+-+…\n",
                "└─────────────┴──────────┴───────…\n",
                "                                  \n",
            )
        );

        // With Clip
        let formatted = table.layout_strict(LayoutOptions::new(
            Dimension::new(34, 8),
            true,
            WrapMode::default_truncate(),
            Some(Rect::new(2, 1, Dimension::new(30, 5))),
        ));
        let result = format!("{formatted}");
        assert_eq!(
            result,
            concat!(
                "   Col 1    │  Col 2   │ Col 3\n",
                "────────────┼──────────┼──────\n",
                "bcdefghijklm│0101010101│ABCDEF\n",
                "opqrstuvwxyz│0101010101│NOPQRS\n",
                "────────────┼──────────┼──────\n",
            )
        );
    }

    #[test]
    fn table_layout_wrap() {
        let table = Table::new(
            TableDecoration::boxed_grid(),
            vec![
                TableColumn::default()
                    .with_header(Lines::center("Col 1"))
                    .with_anchor(CellAnchor::Fill)
                    .with_width(CellWidth::Minimal),
                TableColumn::default()
                    .with_header(Lines::center("Col 2"))
                    .with_anchor(CellAnchor::Fill)
                    .with_width(CellWidth::Minimal),
                TableColumn::default()
                    .with_header(Lines::center("Col 3"))
                    .with_anchor(CellAnchor::Fill)
                    .with_width(CellWidth::Minimal),
            ],
            vec![
                vec![
                    Lines::left("abcdefghijklm\nnopqrstuvwxyz").into(),
                    Filler::both("01").into(),
                    Lines::left("ABCDEFGHIJKLM\nNOPQRSTUVWXYZ").into(),
                ],
                vec![
                    Filler::both("ab").into(),
                    Lines::left("0123456789").into(),
                    Filler::both("+-").into(),
                ],
            ],
        );

        // No Clip
        let formatted = table.layout_strict(LayoutOptions::new(
            Dimension::new(25, 17),
            false,
            WrapMode::Wrap,
            None,
        ));
        let result = format!("{formatted}");
        assert_eq!(
            result,
            concat!(
                "┌─────────────┬──────────\n",
                "│    Col 1    │  Col 2\n",
                "├─────────────┼──────────\n",
                "│abcdefghijklm│0101010101\n",
                "│nopqrstuvwxyz│0101010101\n",
                "├─────────────┼──────────\n",
                "│ababababababa│0123456789\n",
                "└─────────────┴──────────\n",
                "┬─────────────┐\n",
                "│    Col 3    │\n",
                "┼─────────────┤\n",
                "│ABCDEFGHIJKLM│\n",
                "│NOPQRSTUVWXYZ│\n",
                "┼─────────────┤\n",
                "│+-+-+-+-+-+-+│\n",
                "┴─────────────┘\n",
                "\n",
            )
        );

        // With Clip
        let formatted = table.layout_strict(LayoutOptions::new(
            Dimension::new(25, 17),
            false,
            WrapMode::Wrap,
            Some(Rect::new(2, 1, Dimension::new(15, 10))),
        ));
        let result = format!("{formatted}");
        assert_eq!(
            result,
            concat!(
                "   Col 1    │  \n",
                "────────────┼──\n",
                "bcdefghijklm│01\n",
                "opqrstuvwxyz│01\n",
                "────────────┼──\n",
                "babababababa│01\n",
                "────────────┴──\n",
                "────────────┐\n",
                "   Col 3    │\n",
                "────────────┤\n",
            )
        );
    }

    #[test]
    fn table_layout_zero_width() {
        let table = Table::new(
            TableDecoration::boxed_grid(),
            vec![
                TableColumn::default()
                    .with_header(Lines::center("Col 1"))
                    .with_width(CellWidth::Minimal),
                TableColumn::default()
                    .with_header(Lines::center("Col 2"))
                    .with_width(CellWidth::Minimal),
                TableColumn::default()
                    .with_header(Lines::center("Col 3"))
                    .with_width(CellWidth::Minimal),
            ],
            vec![
                vec![
                    Lines::left("abcdefghijklm\nnopqrstuvwxyz").into(),
                    Filler::both("01").into(),
                    Lines::left("ABCDEFGHIJKLM\nNOPQRSTUVWXYZ").into(),
                ],
                vec![
                    Filler::both("ab").into(),
                    Lines::left("0123456789").into(),
                    Filler::both("+-").into(),
                ],
            ],
        );

        assert_eq!(format!("{}", table.layout(0)), "");
    }

    // -- Fill column width distribution tests -------------------------------------------------------

    /// A single Fill column takes all space left after fixed-width and decoration columns.
    ///
    /// Layout (headless_no_grid, 1-char separator, `fill_rows=true`):
    /// ```text
    /// abc x     ← col0(Minimal)=3, sep=1, col1(Fill)=6  →  total 10
    /// ```
    #[test]
    fn fill_single_col_takes_remaining_space() {
        // Using fill_rows=true so each cell is padded to its assigned column width.
        // col0="abc" fills 3 chars; col1="x" fills 1 char and is padded to 6 (fill col).
        let table = headless_table(
            vec![CellWidth::Minimal, CellWidth::Fill],
            vec![Lines::left("abc").into(), Lines::left("x").into()],
        );
        let result = format!(
            "{}",
            table.layout_strict(LayoutOptions::new(
                Dimension::new(10, 1),
                true,
                WrapMode::default_truncate(),
                None,
            ))
        );
        assert_eq!(result, "abc x     \n");
    }

    /// Two Fill columns split the remaining space evenly.
    ///
    /// Layout (headless_no_grid, 1-char separator, total width 11, `fill_rows=true`):
    /// ```text
    /// A     B    ← col0(Fill)=5, sep=1, col1(Fill)=5  →  total 11
    /// ```
    #[test]
    fn fill_two_cols_split_evenly() {
        let table = headless_table(
            vec![CellWidth::Fill, CellWidth::Fill],
            vec![Lines::left("A").into(), Lines::left("B").into()],
        );
        let result = format!(
            "{}",
            table.layout_strict(LayoutOptions::new(
                Dimension::new(11, 1),
                true,
                WrapMode::default_truncate(),
                None,
            ))
        );
        assert_eq!(result, "A     B    \n");
    }

    /// When the remaining space cannot be split evenly, the last Fill column gets the extra char.
    ///
    /// Layout (headless_no_grid, 1-char separator, total width 12, `fill_rows=true`):
    /// ```text
    /// A     B     ← col0(Fill)=5, sep=1, col1(Fill)=6  →  total 12
    /// ```
    #[test]
    fn fill_two_cols_odd_remainder_last_col_gets_more() {
        let table = headless_table(
            vec![CellWidth::Fill, CellWidth::Fill],
            vec![Lines::left("A").into(), Lines::left("B").into()],
        );
        let result = format!(
            "{}",
            table.layout_strict(LayoutOptions::new(
                Dimension::new(12, 1),
                true,
                WrapMode::default_truncate(),
                None,
            ))
        );
        assert_eq!(result, "A     B     \n");
    }

    /// Three Fill columns distribute remaining space using the iterative halving algorithm:
    /// first column gets the floor share; the remainder propagates to the next columns.
    ///
    /// Layout (headless_no_grid, 2 separators, total width 32, `fill_rows=true`):
    /// ```text
    /// A         B         C          ← each col=10, 2 seps  →  total 32
    /// ```
    #[test]
    fn fill_three_cols_distribute_remaining_space() {
        let table = Table::new(
            TableDecoration::headless_no_grid(),
            vec![
                TableColumn::default().with_width(CellWidth::Fill),
                TableColumn::default().with_width(CellWidth::Fill),
                TableColumn::default().with_width(CellWidth::Fill),
            ],
            vec![vec![
                Lines::left("A").into(),
                Lines::left("B").into(),
                Lines::left("C").into(),
            ]],
        );
        let result = format!(
            "{}",
            table.layout_strict(LayoutOptions::new(
                Dimension::new(32, 1),
                true,
                WrapMode::default_truncate(),
                None,
            ))
        );
        assert_eq!(result, "A          B          C         \n");
    }

    /// A Fill column receives the minimum width of 1 when fixed columns already consume all
    /// available space (or more). This was previously broken by a `%` operator precedence bug
    /// and then by a subtraction overflow when computing the per-column fill remainder.
    #[test]
    fn fill_col_gets_minimum_width_when_fixed_cols_exceed_max() {
        // col0 content is 30 chars wide → Minimal width = 30; with sep = 31 fixed chars total.
        // max_width = 20 < 31, so saturating_sub gives fill_width = 0.
        // fill_count = 1, so checked_div(1).max(1) = 1 → Fill col gets width 1.
        // The subsequent update fill_width = 0 - 1 previously caused an underflow panic.
        let table = headless_table(
            vec![CellWidth::Minimal, CellWidth::Fill],
            vec![Lines::left(&"a".repeat(30)).into(), Lines::left("x").into()],
        );
        // Must not panic. The Fill column is assigned the minimum width of 1.
        let dim = table
            .measure(MeasureMode::fixed_width(20, WrapMode::default_truncate()))
            .dim;
        assert!(dim.height > 0);
        assert!(dim.width > 0);
    }

    /// With `MeasureMode::Min` there is no max-width, so Fill columns fall back to measuring
    /// their content minimally (same as `CellWidth::Minimal`).
    #[test]
    fn fill_col_with_min_mode_falls_back_to_minimal() {
        // Both cells contain 3-char text; col0 is Minimal and col1 is Fill (→ also Minimal here).
        // Expected dim: width = 3 + 1(sep) + 3 = 7, height = 1.
        let table = headless_table(
            vec![CellWidth::Minimal, CellWidth::Fill],
            vec![Lines::left("abc").into(), Lines::left("xyz").into()],
        );
        let dim = table.measure(MeasureMode::Min).dim;
        assert_eq!(dim.width, 7);
        assert_eq!(dim.height, 1);
    }

    /// A Proportional column receives the given fraction of the total available width.
    ///
    /// Layout (headless_no_grid, available width 20):
    /// - col0 = Proportional(0.5) → 10 chars
    /// - sep = 1 char
    /// - col1 = Minimal("abc") → 3 chars
    /// - Natural table width = 14
    #[test]
    fn proportional_col_gets_fraction_of_available_width() {
        // Proportional(0.5) is computed relative to the measure-mode width (20).
        // col0 = floor(20 * 0.5) = 10.  Natural total width = 10 + 1 + 3 = 14.
        let table = headless_table(
            vec![CellWidth::Proportional(0.5), CellWidth::Minimal],
            vec![Lines::left("x").into(), Lines::left("abc").into()],
        );
        let dim = table
            .measure(MeasureMode::fixed_width(20, WrapMode::default_truncate()))
            .dim;
        assert_eq!(dim.width, 14); // 10 + 1(sep) + 3 = 14
    }

    /// The height of a row is determined by the tallest cell in that row.
    #[test]
    fn row_height_equals_tallest_cell() {
        // col0: 1-line cell; col1: 3-line cell → row height = 3.
        // No decoration rows in headless_no_grid → total height = 3.
        let table = headless_table(
            vec![CellWidth::Minimal, CellWidth::Minimal],
            vec![
                Lines::left("a").into(),
                Lines::left("line1\nline2\nline3").into(),
            ],
        );
        let dim = table.measure(MeasureMode::Min).dim;
        assert_eq!(dim.height, 3);
        // Width: 1(col0) + 1(sep) + 5(col1) = 7
        assert_eq!(dim.width, 7);
    }

    /// A fixed-width column always uses exactly the specified width, regardless of content.
    #[test]
    fn fixed_col_uses_specified_width() {
        // col0: Fixed(8), content "abc" (3 chars) → still rendered at width 8.
        // col1: Minimal, content "xy" → width 2.  Total = 8 + 1 + 2 = 11.
        let table = headless_table(
            vec![CellWidth::Fixed(8), CellWidth::Minimal],
            vec![Lines::left("abc").into(), Lines::left("xy").into()],
        );
        let dim = table.measure(MeasureMode::Min).dim;
        assert_eq!(dim.width, 11);
    }
}
