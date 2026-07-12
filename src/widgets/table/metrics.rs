use crate::widgets::table::decoration::DecoratedTable;
use crate::widgets::{Cell, CellDimension, CellWidth, Row, TableColumn};
use crate::{Dimension, WrapMode};
use std::cmp::max;

/// Metrics for a table, including widths and heights of cells and rows.
///
/// This struct calculates and stores the dimensions of all cells and rows in a table,
/// taking into account column widths, row heights, and wrapping modes.
pub(crate) struct TableMetrics<'a> {
    table: &'a DecoratedTable<'a>,
    widths: Vec<usize>,
    heights: Vec<usize>,
}

impl<'a> TableMetrics<'a> {
    pub(crate) fn new(
        table: &'a DecoratedTable<'a>,
        max_width: Option<usize>,
        wrap_mode: WrapMode,
    ) -> Self {
        let cols = table.cols;
        let rows = table.rows;
        let mut this = Self {
            table,
            widths: vec![0; cols],
            heights: vec![0; rows],
        };
        this.compute_widths(max_width, wrap_mode);
        this.compute_heights();
        this
    }

    /// Returns all rows of the table, properly formatted and wrapped.
    ///
    /// # Parameters
    /// - `max_width`: The maximum width available for the table
    /// - `wrap_mode`: The wrapping mode to use
    ///
    /// # Returns
    /// A vector of all rows in the table
    pub(crate) fn all_rows(&self, max_width: usize, wrap_mode: WrapMode) -> Vec<Row> {
        let mut len = 0;
        let mut all_rows = (0..self.table.rows)
            .map(|row| self.row(row, max_width, wrap_mode))
            .fold(vec![], |mut acc, row| {
                len += max(len, row.len());
                acc.push(row);
                acc
            });

        let mut result = Vec::with_capacity(len * all_rows.len());
        let mut consumed = true;
        while consumed {
            consumed = false;
            for row in &mut all_rows {
                if !row.is_empty() {
                    result.push(row.remove(0));
                    consumed = true;
                }
            }
        }
        result
    }

    /// Returns a specific row of the table.
    ///
    /// # Parameters
    /// - `row`: The row index
    /// - `max_width`: The maximum width available
    /// - `wrap_mode`: The wrapping mode to use
    ///
    /// # Returns
    /// A vector of Row objects representing the formatted row
    pub(crate) fn row(&self, row: usize, max_width: usize, wrap_mode: WrapMode) -> Vec<Row> {
        let cells = (0..self.table.cols)
            .map(|col| self.cell_at(row, col).unwrap())
            .collect::<Vec<_>>();
        Row::from_cells(cells, max_width, wrap_mode)
    }

    /// Returns the cell at the specified position.
    ///
    /// # Parameters
    /// - `row`: The row index
    /// - `col`: The column index
    ///
    /// # Returns
    /// The cell at the specified position, or None if out of bounds
    pub(crate) fn cell_at(&self, row: usize, col: usize) -> Option<Cell> {
        let cell = self.table.table_column_at(col).map(|table_colum| {
            self.table.cell_at(row, col).map(|content| {
                let cell_dim = Dimension::new(self.widths[col], self.heights[row]);
                let mut content_dim = cell_dim;
                if !self.table.row_index(row).unwrap().is_deco()
                    && !self.table.column_index(col).unwrap().is_deco()
                {
                    content_dim = table_colum
                        .width
                        .calculate_dims(&content, Some(cell_dim.width), table_colum.wrap_mode)
                        .1;
                }

                Cell::new(
                    content,
                    CellDimension::Fixed {
                        cell: cell_dim,
                        content: content_dim,
                    },
                    Some(table_colum.wrap_mode),
                    None,
                    table_colum.anchor,
                )
            })
        });
        cell.flatten()
    }

    fn compute_widths(&mut self, max_width: Option<usize>, wrap_mode: WrapMode) {
        // 1. Step: Compute the widths of each column if width != Fill
        let mut fill_count = 0;
        let mut fixed_width = 0;
        for col in 0..self.table.cols {
            let table_col = self.table.table_column_at(col).unwrap();
            if table_col.width != CellWidth::Fill && max_width.is_some() {
                self.widths[col] =
                    self.compute_width_for_column(col, table_col, max_width, wrap_mode);
                fixed_width += self.widths[col];
            } else {
                fill_count += 1;
            }
        }

        // 2. Step: Compute the widths of those cells with width == Fill
        if fill_count > 0
            && let Some(max_width) = max_width
        {
            let mut fill_width = if fixed_width + fill_count > max_width {
                max_width - fixed_width % max_width
            } else {
                max_width - fixed_width
            };
            for col in 0..self.table.cols {
                let table_col = self.table.table_column_at(col).unwrap();
                if table_col.width == CellWidth::Fill {
                    self.widths[col] = max(1, fill_width / fill_count);
                    fill_width -= self.widths[col];
                    fill_count -= 1;
                }
            }
        }
    }

    fn compute_width_for_column(
        &self,
        col: usize,
        table_column: &TableColumn,
        max_width: Option<usize>,
        wrap_mode: WrapMode,
    ) -> usize {
        let mut width = 0;
        for row in 0..self.table.rows {
            let cell = Cell::new(
                self.table.cell_at(row, col).unwrap(),
                CellDimension::Declarative(table_column.width),
                Some(table_column.wrap_mode),
                None,
                table_column.anchor,
            );
            width = max(width, cell.calculate_dims(max_width, wrap_mode).0.width);
        }
        width
    }

    fn compute_heights(&mut self) {
        for row in 0..self.table.rows {
            let mut height = 0;
            for col in 0..self.table.cols {
                let table_column = self.table.table_column_at(col).unwrap();
                let cell = Cell::new(
                    self.table.cell_at(row, col).unwrap(),
                    CellDimension::Declarative(CellWidth::Fixed(self.widths[col])),
                    Some(table_column.wrap_mode),
                    None,
                    table_column.anchor,
                );
                height = max(
                    height,
                    cell.calculate_dims(None, table_column.wrap_mode).0.height,
                );
            }
            self.heights[row] = height;
        }
    }

    /// Returns the total dimension of the table.
    ///
    /// # Returns
    /// The dimension of the entire table
    pub(crate) fn dim(&self) -> Dimension {
        Dimension::new(
            self.widths.iter().sum::<usize>(),
            self.heights.iter().sum::<usize>(),
        )
    }
}
