use crate::ext::{
    BaseLayoutWriter, BoxedLayoutWriter, DisplayStr, FormattedLayout, LayoutWithOptions,
    LayoutWriter, SizedLayoutResult,
};
use crate::widgets::{Cell, CellAnchor, CellDimension, Filler};
use crate::{BoxedFormattedLayout, Dimension, LayoutOptions, Rect, WrapMode};
use std::cmp::max;
use std::fmt::Write;

/// A `Row` is a helper struct used for horizontal layouting.
/// The basic idea is to arrange [`Cell`]s into `Rows`. A single `Row` contains the (portions) of
/// `Cells` that fit into a single horizontal row. Typically, instances of this type are created
/// via the [`Row::from_cells`] function which returns a vector of `Row`s (the vector contains only
/// one element in case all cells fit into a single row).
pub struct Row {
    /// The list of [`Cell`]s that belong to the row.
    pub(crate) cells: Vec<Cell>,

    /// The total [`Dimension`] of all cells in the row
    pub(crate) dim: Dimension,
}

impl Row {
    /// Creates a group of [`Row`]s based on the provided [`Cell`]s, maximum width, and wrap mode.
    ///
    /// # Parameters
    /// - `cells`: The list of `Cells`
    /// - `max_width`: The maximum width of a single row
    /// - `wrap_mode`: The wrap mode to use for breaking cells into multiple rows
    ///
    /// # Returns
    /// A vector of [`Row`]s representing the arranged cells
    ///
    /// # Example
    /// ```rust
    /// use termlayout::widgets::{Cell, Lines, Row};
    /// use termlayout::{Dimension, WrapMode};
    ///
    /// let cells = vec![
    ///     Cell::of(Lines::left("abcdefghij\nklmnopqrst"))
    ///         .with_dim(Dimension::new(10, 2)),
    ///     Cell::of(Lines::left("01234\n56789"))
    ///         .with_dim(Dimension::new(5, 2)),
    ///     Cell::of(Lines::left("ABCDEFGHIJ\nKLMNOPQRST"))
    ///         .with_dim(Dimension::new(10, 2)),
    /// ];
    ///
    /// let rows = Row::from_cells(cells.clone(), 25, WrapMode::Wrap); // All cells fit in one row
    /// assert_eq!(rows.len(), 1);
    ///
    /// let rows = Row::from_cells(cells.clone(), 10, WrapMode::Wrap); // The cells are split over three rows
    /// assert_eq!(rows.len(), 3);
    /// ```
    pub fn from_cells<T>(cells: T, max_width: usize, wrap_mode: WrapMode) -> Vec<Self>
    where
        T: Into<Vec<Cell>>,
    {
        if max_width == 0 {
            return vec![];
        }
        let cells = cells.into();

        // Check, whether all cells fit in one row
        let total_dim = cells
            .iter()
            .map(Cell::visible_content)
            .fold(Dimension::empty(), |acc, rect| {
                acc.horizontal_union(rect.dim)
            });
        if total_dim.width <= max_width {
            return vec![Row::new(cells, total_dim)];
        }

        // Apply wrap mode
        match wrap_mode {
            WrapMode::Wrap => Self::from_cells_with_wrap(cells, max_width),
            WrapMode::Truncate(indicator) => {
                Self::from_cells_with_truncate(cells, max_width, indicator)
            }
        }
    }

    fn from_cells_with_wrap(mut cells: Vec<Cell>, max_width: usize) -> Vec<Self> {
        let mut rows = Vec::new();
        let mut index = 0;

        while index < cells.len() {
            let (ofs, mut dim) = Self::collect_complete_cells_for_row(&cells[index..], max_width);
            let mut row = cells[index..index + ofs].to_vec();
            index += ofs;
            if index < cells.len() && dim.width < max_width {
                let (cell, rest) = cells[index].split_horizontal(max_width - dim.width);
                dim = dim.horizontal_union(cell.visible_content().dim);
                row.push(cell);
                if !rest.visible_content().is_empty() {
                    cells[index] = rest;
                }
            }
            rows.push(Row::new(row, dim));
        }
        rows
    }

    fn from_cells_with_truncate(
        mut cells: Vec<Cell>,
        max_width: usize,
        indicator: &str,
    ) -> Vec<Self> {
        // compute the limits
        let indicator_len = indicator.display_len();
        let available_width = max(1, max_width.saturating_sub(indicator_len));
        let indicator = indicator.display_slice(0..max_width - available_width);

        // Compute the cells
        let (mut index, mut dim) = Self::collect_complete_cells_for_row(&cells, available_width);
        if index < cells.len() && dim.width < available_width {
            cells[index].truncate_horizontal(available_width - dim.width);
            dim = dim.horizontal_union(cells[index].visible_content().dim);
            index += 1;
        }
        cells.truncate(index);

        // Add the indicator
        let filler = Cell::of(Filler::vertical(indicator))
            .with_anchor(CellAnchor::Fill)
            .with_dim(Dimension::new(indicator.display_len(), 1));
        dim = dim.horizontal_union(filler.visible_content().dim);
        cells.push(filler);

        vec![Row::new(cells, dim)]
    }

    fn collect_complete_cells_for_row(cells: &[Cell], max_width: usize) -> (usize, Dimension) {
        let mut dim = Dimension::empty();
        for (index, cell) in cells.iter().enumerate() {
            let cell_dim = cell.visible_content().dim;
            if dim.width + cell_dim.width > max_width {
                return (index, dim);
            }
            dim = dim.horizontal_union(cell_dim);
        }
        (cells.len(), dim)
    }

    fn new(mut cells: Vec<Cell>, dim: Dimension) -> Self {
        cells
            .iter_mut()
            .filter(|cell| cell.anchor == CellAnchor::Fill)
            .for_each(|cell| {
                let dims = cell.dim.dims().unwrap();
                cell.dim = CellDimension::Fixed {
                    cell: Dimension::new(dims.0.width, dim.height),
                    content: Dimension::new(dims.0.width, dim.height),
                };
            });
        Self { cells, dim }
    }

    /// Creates a [`FormattedLayout`] for the row with the given options.
    ///
    /// # Parameters
    /// - `options`: Layout options for the row
    ///
    /// # Returns
    /// A boxed formatted layout for the row
    #[must_use]
    pub fn layout(&self, options: LayoutOptions) -> BoxedFormattedLayout<'static> {
        FormattedRow::new(&self.cells, options).into()
    }
}

struct FormattedRow {
    options: LayoutOptions,
    formatted_cells: Vec<BoxedFormattedLayout<'static>>,
}

impl FormattedRow {
    fn new(cells: &[Cell], options: LayoutOptions) -> Self {
        let visible_rect = options.visible_rect();
        let mut col = 0;
        let mut formatted_cells = Vec::with_capacity(cells.len());

        for (index, cell) in cells.iter().enumerate() {
            let cell_dim = Dimension::new(cell.visible_content().dim.width, options.dim.height);
            let cell_clip = Rect::new(col, 0, cell_dim).intersect_relative(visible_rect);
            let cell_opts = LayoutOptions::new(
                cell_dim,
                options.fill_rows || index != cells.len() - 1,
                cell.effective_wrap_mode(options.wrap_mode),
                Some(cell_clip),
            );
            let cell: BoxedFormattedLayout<'static> =
                LayoutWithOptions::of(cell.clone().into(), cell_opts).into();
            formatted_cells.push(cell);
            col += cell_dim.width;
            if col > options.dim.width {
                break;
            }
        }
        Self {
            options: options.with_dim(visible_rect.dim).with_clip(None),
            formatted_cells,
        }
    }
}

impl FormattedLayout for FormattedRow {
    fn options(&self) -> &LayoutOptions {
        &self.options
    }

    fn new_writer(&'_ self) -> BoxedLayoutWriter<'_> {
        Box::new(RowWriter::new(
            self.formatted_cells
                .iter()
                .map(|layout| layout.new_writer())
                .collect(),
            &self.options,
        ))
    }
}

impl From<FormattedRow> for BoxedFormattedLayout<'static> {
    fn from(value: FormattedRow) -> Self {
        Box::new(value)
    }
}

struct RowWriter<'wrt> {
    base: BaseLayoutWriter<'wrt>,
    cells: Vec<BoxedLayoutWriter<'wrt>>,
}

impl<'wrt> RowWriter<'wrt> {
    fn new(cells: Vec<BoxedLayoutWriter<'wrt>>, options: &'wrt LayoutOptions) -> Self {
        Self {
            base: BaseLayoutWriter::new(options),
            cells,
        }
    }

    fn write_cells(&mut self, w: &mut dyn Write) -> std::fmt::Result {
        for cell in &mut self.cells {
            self.base.write_row(cell.as_mut(), w)?;
        }
        Ok(())
    }
}

impl<'wrt> LayoutWriter<'wrt> for RowWriter<'wrt> {
    fn options(&self) -> &'wrt LayoutOptions {
        self.base.options()
    }

    fn write_row(&mut self, w: &mut dyn Write) -> SizedLayoutResult {
        self.write_cells(w)?;
        self.base.end_row(w)
    }
}

#[cfg(test)]
mod tests {
    use crate::widgets::horizontal::row::{FormattedRow, Row};
    use crate::widgets::{Cell, CellAnchor, Lines};
    use crate::{BoxedFormattedLayout, Dimension, LayoutOptions, Rect};

    fn small_sample_cells() -> Vec<Cell> {
        vec![
            Cell::of(Lines::left("abcdef\nghijkl\nmnopqr\nstuvwx")).with_dim(Dimension::new(6, 4)),
            Cell::of(Lines::left("123\n456\n789"))
                .with_dim(Dimension::new(3, 3))
                .with_anchor(CellAnchor::Center),
            Cell::of(Lines::left("ABCDE\nFGHIJ\nKLMNO\nPQRST\nUVWXY"))
                .with_dim(Dimension::new(5, 5)),
        ]
    }

    fn big_sample_cells() -> Vec<Cell> {
        vec![
            Cell::of(Lines::left("abcdefghijkl\nmnopqrstuvwx")).with_dim(Dimension::new(12, 2)),
            Cell::of(Lines::left("123\n456\n789"))
                .with_dim(Dimension::new(3, 3))
                .with_anchor(CellAnchor::Center),
            Cell::of(Lines::left("ABCDEFGHIJKLMNOPQRSTUVWXY")).with_dim(Dimension::new(25, 1)),
        ]
    }

    #[test]
    fn row_from_cells_with_wrap() {
        // No wrap
        let cells = big_sample_cells();
        let rows = Row::from_cells_with_wrap(cells, 50);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].cells.len(), 3);
        assert_eq!(
            rows[0].cells[0].visible_content(),
            Rect::new(0, 0, Dimension::new(12, 2))
        );
        assert_eq!(
            rows[0].cells[1].visible_content(),
            Rect::new(0, 0, Dimension::new(3, 3))
        );
        assert_eq!(
            rows[0].cells[2].visible_content(),
            Rect::new(0, 0, Dimension::new(25, 1))
        );
        assert_eq!(rows[0].dim, Dimension::new(40, 3));

        // Wrap on cell border
        let cells = big_sample_cells();
        let rows = Row::from_cells_with_wrap(cells, 15);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].cells.len(), 2);
        assert_eq!(
            rows[0].cells[0].visible_content(),
            Rect::new(0, 0, Dimension::new(12, 2))
        );
        assert_eq!(
            rows[0].cells[1].visible_content(),
            Rect::new(0, 0, Dimension::new(3, 3))
        );
        assert_eq!(rows[0].dim, Dimension::new(15, 3));
        assert_eq!(rows[1].cells.len(), 1);
        assert_eq!(
            rows[1].cells[0].visible_content(),
            Rect::new(0, 0, Dimension::new(15, 1))
        );
        assert_eq!(rows[1].dim, Dimension::new(15, 1));
        assert_eq!(rows[2].cells.len(), 1);
        assert_eq!(
            rows[2].cells[0].visible_content(),
            Rect::new(15, 0, Dimension::new(10, 1))
        );
        assert_eq!(rows[2].dim, Dimension::new(10, 1));

        // Wrap 4 times
        let cells = big_sample_cells();
        let rows = Row::from_cells_with_wrap(cells, 10);
        assert_eq!(rows.len(), 4);
        assert_eq!(rows[0].cells.len(), 1);
        assert_eq!(
            rows[0].cells[0].visible_content(),
            Rect::new(0, 0, Dimension::new(10, 2))
        );
        assert_eq!(rows[0].dim, Dimension::new(10, 2));
        assert_eq!(rows[1].cells.len(), 3);
        assert_eq!(
            rows[1].cells[0].visible_content(),
            Rect::new(10, 0, Dimension::new(2, 2))
        );
        assert_eq!(
            rows[1].cells[1].visible_content(),
            Rect::new(0, 0, Dimension::new(3, 3))
        );
        assert_eq!(
            rows[1].cells[2].visible_content(),
            Rect::new(0, 0, Dimension::new(5, 1))
        );
        assert_eq!(rows[1].dim, Dimension::new(10, 3));
        assert_eq!(rows[2].cells.len(), 1);
        assert_eq!(
            rows[2].cells[0].visible_content(),
            Rect::new(5, 0, Dimension::new(10, 1))
        );
        assert_eq!(rows[2].dim, Dimension::new(10, 1));
        assert_eq!(rows[3].cells.len(), 1);
        assert_eq!(
            rows[3].cells[0].visible_content(),
            Rect::new(15, 0, Dimension::new(10, 1))
        );
        assert_eq!(rows[3].dim, Dimension::new(10, 1));
    }

    #[test]
    fn row_from_cells_with_truncate() {
        // Minimal space
        let cells = big_sample_cells();
        let rows = Row::from_cells_with_truncate(cells, 3, "...");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].cells.len(), 2);
        assert_eq!(
            rows[0].cells[0].visible_content(),
            Rect::new(0, 0, Dimension::new(1, 2))
        );
        assert_eq!(
            rows[0].cells[1].visible_content(),
            Rect::new(0, 0, Dimension::new(2, 2))
        );
        assert_eq!(rows[0].dim, Dimension::new(3, 2));

        // Exact column fit
        let cells = big_sample_cells();
        let rows = Row::from_cells_with_truncate(cells, 18, "...");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].cells.len(), 3);
        assert_eq!(
            rows[0].cells[0].visible_content(),
            Rect::new(0, 0, Dimension::new(12, 2))
        );
        assert_eq!(
            rows[0].cells[1].visible_content(),
            Rect::new(0, 0, Dimension::new(3, 3))
        );
        assert_eq!(
            rows[0].cells[2].visible_content(),
            Rect::new(0, 0, Dimension::new(3, 3))
        );
        assert_eq!(rows[0].dim, Dimension::new(18, 3));

        // Part column fit
        let cells = big_sample_cells();
        let rows = Row::from_cells_with_truncate(cells, 19, "...");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].cells.len(), 4);
        assert_eq!(
            rows[0].cells[0].visible_content(),
            Rect::new(0, 0, Dimension::new(12, 2))
        );
        assert_eq!(
            rows[0].cells[1].visible_content(),
            Rect::new(0, 0, Dimension::new(3, 3))
        );
        assert_eq!(
            rows[0].cells[2].visible_content(),
            Rect::new(0, 0, Dimension::new(1, 1))
        );
        assert_eq!(
            rows[0].cells[3].visible_content(),
            Rect::new(0, 0, Dimension::new(3, 3))
        );
        assert_eq!(rows[0].dim, Dimension::new(19, 3));

        // All fits
        let cells = big_sample_cells();
        let rows = Row::from_cells_with_truncate(cells, 100, "...");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].cells.len(), 4);
        assert_eq!(
            rows[0].cells[0].visible_content(),
            Rect::new(0, 0, Dimension::new(12, 2))
        );
        assert_eq!(
            rows[0].cells[1].visible_content(),
            Rect::new(0, 0, Dimension::new(3, 3))
        );
        assert_eq!(
            rows[0].cells[2].visible_content(),
            Rect::new(0, 0, Dimension::new(25, 1))
        );
        assert_eq!(
            rows[0].cells[3].visible_content(),
            Rect::new(0, 0, Dimension::new(3, 3))
        );
        assert_eq!(rows[0].dim, Dimension::new(43, 3));
    }

    #[test]
    fn row_collect_complete_cells_for_row() {
        let cells = big_sample_cells();

        let (index, dim) = Row::collect_complete_cells_for_row(&cells, 10);
        assert_eq!(index, 0);
        assert_eq!(dim, Dimension::empty());

        let (index, dim) = Row::collect_complete_cells_for_row(&cells, 12);
        assert_eq!(index, 1);
        assert_eq!(dim, Dimension::new(12, 2));

        let (index, dim) = Row::collect_complete_cells_for_row(&cells, 14);
        assert_eq!(index, 1);
        assert_eq!(dim, Dimension::new(12, 2));

        let (index, dim) = Row::collect_complete_cells_for_row(&cells, 25);
        assert_eq!(index, 2);
        assert_eq!(dim, Dimension::new(15, 3));

        let (index, dim) = Row::collect_complete_cells_for_row(&cells, 100);
        assert_eq!(index, 3);
        assert_eq!(dim, Dimension::new(40, 3));
    }

    #[test]
    fn formatted_row_no_clip() {
        let cells = small_sample_cells();
        let options = LayoutOptions::default()
            .with_dim(Dimension::new(14, 6))
            .with_fill_rows(false);

        let formatted: BoxedFormattedLayout = FormattedRow::new(&cells, options).into();

        assert_eq!(
            format!("{formatted}"),
            concat!(
                "abcdef   ABCDE\n",
                "ghijkl123FGHIJ\n",
                "mnopqr456KLMNO\n",
                "stuvwx789PQRST\n",
                "         UVWXY\n",
                "         \n"
            )
        );
    }

    #[test]
    fn formatted_row_with_clip() {
        let cells = small_sample_cells();
        let options = LayoutOptions::default()
            .with_dim(Dimension::new(14, 6))
            .with_fill_rows(false)
            .with_clip(Some(Rect::new(2, 1, Dimension::new(10, 4))));

        let formatted: BoxedFormattedLayout = FormattedRow::new(&cells, options).into();

        assert_eq!(
            format!("{formatted}"),
            concat!(
                "ijkl123FGH\n", //
                "opqr456KLM\n", //
                "uvwx789PQR\n", //
                "       UVW\n", //
            )
        );
    }
}
