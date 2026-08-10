use crate::ext::{HorizontalMetrics, Row};
use crate::widgets::table::decoration::DecoratedTable;
use crate::widgets::{Cell, CellAnchor, CellDimension, CellWidth};
use crate::{Dimension, Layout, MeasureMode, MeasurementSpecifics, Measurements, WrapMode};
use std::cmp::max;
use std::collections::VecDeque;

/// Metrics for a table, including widths and heights of cells and rows.
///
/// This struct calculates and stores the dimensions of all cells and rows in a table,
/// taking into account column widths, row heights, and wrapping modes.
pub(crate) struct TableMetrics<'a> {
    table: &'a DecoratedTable<'a>,
    widths: Vec<usize>,
    heights: Vec<usize>,
    measurements: Vec<Measurements>,
}

impl<'a> TableMetrics<'a> {
    pub(crate) fn new(table: &'a DecoratedTable<'a>, mode: MeasureMode) -> Self {
        let cols = table.cols;
        let rows = table.rows;
        let mut this = Self {
            table,
            widths: vec![0; cols],
            heights: vec![0; rows],
            measurements: vec![Measurements::empty(); cols * rows],
        };
        this.measure_widths(mode);
        this.measure_cell_contents_and_heights(mode);
        this
    }

    pub(crate) fn all_rows(&self, mode: MeasureMode) -> Vec<Row> {
        let mut len = 0;
        let wrap_mode = mode.wrap_mode();
        let mut req_height = mode.height();
        let max_width = mode.coerce_width(usize::MAX);

        // First collect all rows
        let mut all_rows = VecDeque::with_capacity(self.table.rows);
        for row in 0..self.table.rows {
            let metrics = self.row(row, wrap_mode, max_width, req_height);
            if let Some(height) = req_height {
                req_height = Some(height.saturating_sub(metrics.dim.height));
            }

            len += metrics.rows.len();
            all_rows.push_back(metrics.rows);
            if req_height == Some(0) {
                break;
            }
        }

        // Now reorder the rows
        let mut result = Vec::with_capacity(len);
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

    fn row(
        &self,
        row: usize,
        wrap_mode: WrapMode,
        max_width: usize,
        req_height: Option<usize>,
    ) -> HorizontalMetrics {
        let mut all_cells = Row::empty();
        (0..self.table.cols)
            .filter_map(|c| self.cell_at(row, c))
            .for_each(|(c, m)| all_cells.push_back(c, m));
        HorizontalMetrics::from_row(all_cells, wrap_mode, max_width, req_height, false)
    }

    /// Returns the [`Cell`]/[`Measurements`] pair at the specified position.
    ///
    /// # Parameters
    /// - `row`: The row index
    /// - `col`: The column index
    ///
    /// # Returns
    /// The [`Cell`]/[`Measurements`] pair at the specified position, or None if out of bounds
    pub(crate) fn cell_at(&self, row: usize, col: usize) -> Option<(Cell, Measurements)> {
        let cell = self.table.table_column_at(col).map(|table_colum| {
            self.table.cell_at(row, col).map(|content| {
                let cell_dim = Dimension::new(self.widths[col], self.heights[row]);
                let mut measurements = self.measurements[row * self.table.cols + col].clone();
                if self.table.is_deco(row, col) {
                    measurements.dim = cell_dim;
                }
                let content_dim = measurements.dim;
                (
                    Cell::new(
                        content,
                        CellDimension::Fixed {
                            cell: cell_dim,
                            content: content_dim,
                        },
                        Some(table_colum.wrap_mode),
                        None,
                        table_colum.anchor,
                    ),
                    Measurements::new(cell_dim, MeasurementSpecifics::Child(measurements.into())),
                )
            })
        });
        cell.flatten()
    }

    fn measure_widths(&mut self, mode: MeasureMode) {
        // 1. Step: Compute the widths of each column if width != Fill
        let mut fill_count = 0;
        let mut fixed_width = 0;
        for col in 0..self.table.cols {
            let table_col = self.table.table_column_at(col).unwrap();
            if table_col.width == CellWidth::Fill {
                fill_count += 1;
            } else {
                self.widths[col] = self.measure_column_width(col, table_col.width, mode);
                fixed_width += self.widths[col];
            }
        }

        // 2. Step: Compute the widths of those cells with width == Fill
        if fill_count > 0 {
            // fill_width is None or Some width to fill, depending on the mode
            let mut fill_width = mode
                .width()
                .map(|max_width| max_width.saturating_sub(fixed_width));
            for col in 0..self.table.cols {
                let table_col = self.table.table_column_at(col).unwrap();
                if table_col.width != CellWidth::Fill {
                    continue;
                }
                match fill_width {
                    Some(width) => {
                        self.widths[col] = width.checked_div(fill_count).map_or(1, |w| w.max(1));
                        fill_width = Some(width - self.widths[col]);
                        fill_count -= 1;
                    }
                    None => {
                        self.widths[col] = self.measure_column_width(col, CellWidth::Minimal, mode);
                    }
                }
            }
        }
    }

    fn measure_column_width(&self, col: usize, cell_width: CellWidth, mode: MeasureMode) -> usize {
        match cell_width {
            CellWidth::Fixed(width) | CellWidth::Preferred(width) => width,
            CellWidth::Proportional(weight) if mode.width().is_some() => {
                #[allow(
                    clippy::cast_precision_loss,
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss
                )]
                let w = (mode.width().unwrap() as f32 * weight) as usize;
                w
            }
            CellWidth::Fill if mode.width().is_some() => 0, // Computed differently
            _ => {
                let mut width = 0;
                for row in 0..self.table.rows {
                    let cell = Cell::new(
                        self.table.cell_at(row, col).unwrap(),
                        CellDimension::Declarative(CellWidth::Minimal),
                        None, //  WrapMode - plays no role in case of minimal
                        None,
                        CellAnchor::NorthWest, // Plays no role in case of minimal
                    );
                    let cell_measurements = cell.measure(MeasureMode::Min);
                    width = cell_measurements.dim.width.max(width);
                }
                width
            }
        }
    }

    fn measure_cell_contents_and_heights(&mut self, mode: MeasureMode) {
        for row in 0..self.table.rows {
            let mut height = 0;
            for col in 0..self.table.cols {
                let table_column = self.table.table_column_at(col).unwrap();
                let cell = Cell::new(
                    self.table.cell_at(row, col).unwrap(),
                    CellDimension::Declarative(table_column.width),
                    Some(table_column.wrap_mode),
                    None,
                    table_column.anchor,
                );
                let measurements =
                    cell.measure(MeasureMode::fixed_width(self.widths[col], mode.wrap_mode()));
                height = max(height, measurements.dim.height);
                self.measurements[row * self.table.cols + col] =
                    measurements.specifics.try_into().unwrap();
            }
            self.heights[row] = height;
        }
    }
}
