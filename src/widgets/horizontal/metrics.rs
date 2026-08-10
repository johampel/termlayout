use crate::ext::DisplayStr;
use crate::widgets::horizontal::row::Row;
use crate::widgets::{Cell, CellAnchor, Filler};
use crate::{Dimension, Layout, MeasureMode, MeasurementSpecifics, Measurements, WrapMode};
use std::cmp::{max, min};

/// Precomputed layout metrics for a [`crate::widgets::Horizontal`] widget.
///
/// `HorizontalMetrics` holds the overall dimension and the row distribution that results from
/// measuring and wrapping a sequence of [`crate::widgets::Cell`]s into display rows.
/// It is an intermediate result produced during the `measure` phase and consumed when
/// rendering the [`crate::widgets::Horizontal`] widget.
pub struct HorizontalMetrics {
    /// The overall [`Dimension`] of the laid-out widget.
    pub dim: Dimension,
    /// The list of [`Row`]s that make up the widget after wrapping.
    pub rows: Vec<Row>,
}

impl HorizontalMetrics {
    /// Measures a slice of [`crate::widgets::Cell`]s and organises them into rows.
    ///
    /// This is the main entry point for computing [`HorizontalMetrics`] from a list of cells.
    /// It measures each cell individually and then calls [`from_row`](Self::from_row) to apply
    /// wrapping or truncation as required by `mode`.
    ///
    /// # Parameters
    /// - `cells`: The slice of [`crate::widgets::Cell`]s to measure.
    /// - `mode`: The [`MeasureMode`] that controls sizing and wrapping.
    ///
    /// # Returns
    /// The resulting [`HorizontalMetrics`].
    #[must_use] 
    pub fn from_cells(cells: &[Cell], mode: MeasureMode) -> Self {
        let max_height = mode.height();
        // Build a row containing all cells
        let measurements = Self::measure_cells(cells, mode);
        let mut row = Row::empty();
        for (c, m) in cells.iter().zip(measurements.iter()) {
            row.push_back(c.clone(), m.clone());
        }
        if let Some(height) = max_height {
            row.dim.height = height;
        }

        Self::from_row(
            row,
            mode.wrap_mode(),
            mode.coerce_width(usize::MAX),
            max_height,
            true,
        )
    }

    /// Converts a single [`Row`] into [`HorizontalMetrics`] by applying wrapping or truncation.
    ///
    /// If the row fits within `max_width`, it is returned as-is. Otherwise the row is either
    /// wrapped onto multiple rows ([`WrapMode::Wrap`]) or truncated to a single row
    /// ([`WrapMode::Truncate`]).
    ///
    /// # Parameters
    /// - `row`: The [`Row`] to lay out.
    /// - `wrap_mode`: Controls whether content wraps or is truncated.
    /// - `max_width`: The maximum available display width.
    /// - `max_height`: Optional maximum height. If `Some`, rendering stops at that height.
    /// - `fill_height`: If `true`, the last row is padded to reach `max_height`.
    ///
    /// # Returns
    /// The resulting [`HorizontalMetrics`].
    #[must_use] 
    pub fn from_row(
        row: Row,
        wrap_mode: WrapMode,
        max_width: usize,
        max_height: Option<usize>,
        fill_height: bool,
    ) -> Self {
        if max_width >= row.dim.width {
            return Self {
                dim: row.dim,
                rows: vec![row],
            };
        } else if max_width == 0 {
            return Self {
                dim: Dimension::new(0, 0),
                rows: vec![],
            };
        }
        match wrap_mode {
            WrapMode::Truncate(indicator) => {
                Self::new_with_truncation(row, max_width, max_height, fill_height, indicator)
            }
            WrapMode::Wrap => Self::new_with_wrap(row, max_width, max_height, fill_height),
        }
    }

    fn new_with_wrap(
        mut row: Row,
        max_width: usize,
        req_height: Option<usize>,
        fill_height: bool,
    ) -> Self {
        let max_height = req_height.unwrap_or(usize::MAX);
        let mut rows = vec![];
        let mut dim: Dimension = Dimension::empty();
        let mut current_row = Row::empty();

        while let Some((cell, measurement)) = row.pop_front()
            && max_height > dim.height
        {
            // Append complete cell to current row ;
            if current_row.dim.width + measurement.dim.width <= max_width {
                current_row.push_back(cell, measurement);
                if max_width == current_row.dim.width {
                    dim = dim.vertical_union(current_row.dim);
                    rows.push(current_row);
                    current_row = Row::empty();
                }
                continue;
            }
            let (lc, rc) = cell.split_horizontal(max_width - current_row.dim.width);
            let (lm, rm) = measurement.split_horizontal(max_width - current_row.dim.width);
            row.push_front(rc, rm);
            current_row.push_back(lc, lm);
            dim = dim.vertical_union(current_row.dim);
            rows.push(current_row);
            current_row = Row::empty();
        }
        if !current_row.is_empty() {
            dim = dim.vertical_union(current_row.dim);
            rows.push(current_row);
        }

        // Truncate height if required
        if let Some(height) = req_height
            && let Some(last_row) = rows.last_mut()
            && (dim.height > height || fill_height)
        {
            last_row.dim.height = height.saturating_sub(dim.height - last_row.dim.height);
            dim.height = height;
            // TODO Questionable:
            last_row.cells.iter_mut().for_each(|(_, m)| {
                m.dim.height = last_row.dim.height;
            });
        }

        Self { dim, rows }
    }

    fn new_with_truncation(
        mut row: Row,
        max_width: usize,
        height: Option<usize>,
        fill_height: bool,
        indicator: &str,
    ) -> Self {
        let indicator_len = min(max_width.saturating_sub(1), indicator.display_len());
        let indicator = indicator.display_slice(0..indicator_len);
        let content_width = max_width.saturating_sub(indicator_len);

        while row.dim.width > content_width {
            let (mut cell, mut measurements) = row.pop_back().unwrap();
            if row.dim.width < content_width {
                cell.truncate_horizontal(content_width - row.dim.width);
                measurements.dim.width = content_width - row.dim.width;
                row.push_back(cell, measurements);
                break;
            }
        }
        row.push_back(
            Cell::of(Filler::vertical(indicator))
                .with_anchor(CellAnchor::Fill)
                .with_dim(Dimension::new(indicator.display_len(), 1)),
            Measurements::new(
                Dimension::new(indicator.display_len(), 1),
                MeasurementSpecifics::Child(Box::new(
                    Dimension::new(indicator.display_len(), 1).into(),
                )),
            ),
        );
        if let Some(height) = height
            && fill_height
        {
            row.dim.height = height;
            // TODO Questionable:
            row.cells.iter_mut().for_each(|(_, m)| {
                m.dim.height = height;
            });
        }
        Self {
            dim: row.dim,
            rows: vec![row],
        }
    }

    fn measure_cells(cells: &[Cell], mode: MeasureMode) -> Vec<Measurements> {
        // 1. Step: Compute the preferred dimensions of each cell, if width != Fill
        let mut result: Vec<Measurements> = Vec::with_capacity(cells.len());
        let mut fill_count = 0;
        let mut fixed_width = 0;
        for cell in cells {
            let measurements = if cell.dim.is_fill() && mode.width().is_some() {
                fill_count += 1;
                Measurements::empty()
            } else {
                cell.measure(cell.dim.derive_cell_measure_mode(mode))
            };
            fixed_width += measurements.dim.width;
            result.push(measurements);
        }

        // 2. Step: Compute the dimensions of those cells with width == Fill
        if fill_count > 0
            && let Some(max_width) = mode.width()
        {
            let mut fill_width = if fixed_width + fill_count > max_width {
                max_width - fixed_width % max_width
            } else {
                max_width - fixed_width
            };
            for (index, cell) in cells.iter().enumerate() {
                if cell.dim.is_fill() {
                    let w = max(1, fill_width / fill_count);
                    let measurements = cell.measure(MeasureMode::fixed_width(w, mode.wrap_mode()));
                    fill_width -= measurements.dim.width;
                    fill_count -= 1;
                    result[index] = measurements;
                }
            }
        }
        result
    }
}
