use crate::widgets::table::decoration::DecoratedTable;
use crate::widgets::table::metrics::TableMetrics;
use crate::widgets::vertical::FormattedVertical;
use crate::widgets::{CellAnchor, CellWidth, TableDecoration};
use crate::{rc_layout, BoxedFormattedLayout, Dimension, Layout, LayoutOptions, MeasureMode, RcLayout, Rect, WrapMode, Measurements, LayoutContext};
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
    fn pref_dim(&self, max_width: usize, wrap_mode: WrapMode) -> Dimension {
        let table = DecoratedTable::new(self);
        let metrics = TableMetrics::new(&table, Some(max_width), wrap_mode);
        let dim = metrics.dim();
        if dim.width <= max_width {
            return dim;
        }

        (0..table.rows)
            .map(|r| {
                metrics
                    .row(r, max_width, wrap_mode)
                    .iter()
                    .fold(Dimension::empty(), |acc, row| acc.vertical_union(row.dim))
            })
            .fold(Dimension::empty(), |acc, row| acc.vertical_union(row))
    }

    fn min_dim(&self) -> Dimension {
        let table = DecoratedTable::new(self);
        let metrics = TableMetrics::new(&table, None, WrapMode::Wrap);
        metrics.dim()
    }

    fn measure(&self, mode: MeasureMode) -> Measurements {
        todo!()
    }

    fn layout_strict(&'_ self, options: LayoutOptions) -> BoxedFormattedLayout<'_> {
        let table = DecoratedTable::new(self);
        let metrics = TableMetrics::new(&table, Some(options.dim.width), options.wrap_mode);
        let rows = metrics.all_rows(options.dim.width, options.wrap_mode);

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

    fn layout_with_context(&'_ self, context: LayoutContext) -> BoxedFormattedLayout<'_> {
        todo!()
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
                "│    Col 1    │  Col 2   │    Col…\n",
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
}
