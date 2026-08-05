use crate::ext::{
    BaseLayoutWriter, BoxedLayoutWriter, FormattedLayout, LayoutWithContext, LayoutWriter,
    SizedLayoutResult,
};
use crate::widgets::vertical::FormattedVertical;
use crate::widgets::{Cell, CellAnchor, CellDimension};
use crate::{
    BoxedFormattedLayout, Dimension, LayoutContext, LayoutOptions, MeasurementSpecifics,
    Measurements, Rect,
};
use std::cmp::max;
use std::collections::VecDeque;
use std::fmt::Write;

/// Represents a horizontally arranged set of [`Cell`]s and its corresponding [`Measurments`].
///
/// A `Row` is a helper struct for the [`Horizontal`](super::Horizontal) and
/// [`Table`](crate::widgets::Table) widgets. It does not implement the `Layout` trait and is not
/// aware of line wrapping or truncation. It is primarily used to collect `Cells` that fit into
/// one display row.
///
/// Custom widget implementations might reuse this type.
#[derive(Clone)]
pub struct Row {
    /// The overall [`Dimension`]
    pub dim: Dimension,
    /// The [`Cell`] - [`Measurements`] - pairs that make up the row.
    pub cells: VecDeque<(Cell, Measurements)>,
}

impl Row {
    #[must_use]
    fn new(dim: Dimension, cells: VecDeque<(Cell, Measurements)>) -> Self {
        Self { dim, cells }
    }

    /// Creates an initially empty instance.
    ///
    /// # Returns
    /// The new instance
    ///
    /// # Example
    /// ```rust
    ///
    /// use termlayout::ext::Row;
    ///
    /// let row = Row::empty();
    ///
    /// assert_eq!(row.is_empty(), true);
    /// assert_eq!(row.cells.is_empty(), true);
    /// assert_eq!(row.dim.is_empty(), true);
    /// ```
    #[must_use]
    pub fn empty() -> Self {
        Self::new(Dimension::new(0, 0), VecDeque::new())
    }

    /// Adds a [`Cell`]-[`Measurement`] pair to the end of the instance.
    /// The method implicitly updates the `dim` field.
    ///
    /// # Parameters
    /// - `cell`: The [`Cell`] to append
    /// - `measurements`: The corresponding [`Measurments`]
    ///
    /// # Èxample
    /// ```rust
    ///
    /// use termlayout::{Dimension, MeasurementSpecifics, Measurements};
    /// use termlayout::ext::Row;
    /// use termlayout::widgets::{Cell, Lines};
    ///
    /// let cell1 = Cell::of(Lines::left("first\nline"));
    /// let measurements1 = Measurements::new(Dimension::new(5, 2), MeasurementSpecifics::None);
    /// let cell2 = Cell::of(Lines::left("a\nfuther\nline"));
    /// let measurements2 = Measurements::new(Dimension::new(6, 3), MeasurementSpecifics::None);
    ///
    /// let mut row = Row::empty();
    /// row.push_back(cell1, measurements1);
    /// row.push_back(cell2, measurements2);
    ///
    /// assert_eq!(row.cells.len(), 2);
    /// assert_eq!(row.dim, Dimension::new(11, 3));
    /// assert_eq!(row.cells[0].1.dim, Dimension::new(5, 2));
    /// assert_eq!(row.cells[1].1.dim, Dimension::new(6, 3));
    /// ```
    pub fn push_back(&mut self, cell: Cell, measurement: Measurements) {
        self.dim = self.dim.horizontal_union(measurement.dim);
        self.cells.push_back((cell, measurement));
    }

    /// Adds a [`Cell`]-[`Measurement`] pair to the start of the instance.
    /// The method implicitly updates the `dim` field.
    ///
    /// # Parameters
    /// - `cell`: The [`Cell`] to prepend
    /// - `measurements`: The corresponding [`Measurments`]
    ///
    /// # Èxample
    /// ```rust
    ///
    /// use termlayout::{Dimension, MeasurementSpecifics, Measurements};
    /// use termlayout::ext::Row;
    /// use termlayout::widgets::{Cell, Lines};
    ///
    /// let cell1 = Cell::of(Lines::left("first\nline"));
    /// let measurements1 = Measurements::new(Dimension::new(5, 2), MeasurementSpecifics::None);
    /// let cell2 = Cell::of(Lines::left("a\nfuther\nline"));
    /// let measurements2 = Measurements::new(Dimension::new(6, 3), MeasurementSpecifics::None);
    ///
    /// let mut row = Row::empty();
    /// row.push_front(cell1, measurements1);
    /// row.push_front(cell2, measurements2);
    ///
    /// assert_eq!(row.cells.len(), 2);
    /// assert_eq!(row.dim, Dimension::new(11, 3));
    /// assert_eq!(row.cells[0].1.dim, Dimension::new(6, 3));
    /// assert_eq!(row.cells[1].1.dim, Dimension::new(5, 2));
    /// ```
    pub fn push_front(&mut self, cell: Cell, measurement: Measurements) {
        self.dim = self.dim.horizontal_union(measurement.dim);
        self.cells.push_front((cell, measurement));
    }

    /// Removes the [`Cell`]-[`Measurement`] pair from the end of the instance.
    /// The method implicitly updates the `dim` field. This is the inverse operation of
    /// [`push_back`](Row::push_back)
    ///
    /// # Returns
    /// The removed [`Cell`]-[`Measurement`] pair, if any.
    ///
    pub fn pop_back(&mut self) -> Option<(Cell, Measurements)> {
        let result = self.cells.pop_back();
        if result.is_some() {
            self.update_dim();
        }
        result
    }

    pub fn pop_front(&mut self) -> Option<(Cell, Measurements)> {
        let result = self.cells.pop_front();
        if result.is_some() {
            self.update_dim();
        }
        result
    }

    fn update_dim(&mut self) {
        self.dim = self
            .cells
            .iter()
            .map(|(_, m)| m.dim)
            .fold(Dimension::empty(), |a, b| a.horizontal_union(b));
    }

    pub fn fixiate_cell_dims(&mut self) {
        self.cells.iter_mut().for_each(|(c, m)| {
            m.dim.height = self.dim.height;
            let mut content_dim = m.specifics.child().map(|m| m.dim).unwrap_or(m.dim);
            if matches!(c.anchor, CellAnchor::Fill) {
                content_dim = Dimension::new(
                    max(content_dim.width, m.dim.width),
                    max(content_dim.height, m.dim.height),
                );
                m.specifics = MeasurementSpecifics::Child(Box::new(content_dim.into()));
            }
            c.dim = CellDimension::Fixed {
                cell: m.dim,
                content: content_dim,
            }
        })
    }
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    pub fn layout(context: LayoutContext) -> Option<BoxedFormattedLayout<'static>> {
        let specifics: Result<Vec<Row>, _> = context.measurements.specifics.try_into();
        match specifics {
            Ok(mut rows) => {
                let mut y = 0;
                let children = rows
                    .iter_mut()
                    .map(|row| {
                        row.fixiate_cell_dims();
                        let row_options =
                            context.options.intersect(Rect::new(0, y, row.dim), false);
                        y += row.dim.height;
                        FormattedRow::new(
                            row.cells.iter().map(|e| e.clone()).collect::<Vec<_>>(),
                            row_options,
                        )
                        .into()
                    })
                    .collect();
                Some(
                    FormattedVertical::new(children, context.options.with_normalized_clip()).into(),
                )
            }
            Err(_) => None,
        }
    }
}

impl From<Row> for VecDeque<(Cell, Measurements)> {
    fn from(value: Row) -> Self {
        value.cells
    }
}

pub(crate) struct FormattedRow {
    options: LayoutOptions,
    formatted_cells: Vec<BoxedFormattedLayout<'static>>,
}

impl FormattedRow {
    pub(crate) fn new(cells: Vec<(Cell, Measurements)>, options: LayoutOptions) -> Self {
        let mut col = 0;
        let count = cells.len();
        let mut formatted_cells = Vec::with_capacity(count);
        for (index, (cell, measurements)) in cells.into_iter().enumerate() {
            let mut ctxt = LayoutContext::new_with_intersection(&options, col, 0, measurements);
            ctxt.options.fill_rows = options.fill_rows || index != count - 1;
            col += ctxt.options.dim.width;
            let cell: BoxedFormattedLayout =
                LayoutWithContext::of(cell.clone().into(), ctxt).into();
            formatted_cells.push(cell);
            if col > options.dim.width {
                break;
            }
        }
        Self {
            options: options.with_normalized_clip(),
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
    use crate::core::measurements::MeasurementSpecifics;
    use crate::widgets::horizontal::row::FormattedRow;
    use crate::widgets::{Cell, CellAnchor, Lines};
    use crate::{BoxedFormattedLayout, Dimension, LayoutOptions, Measurements, Rect, WrapMode};

    fn sample_cells() -> Vec<(Cell, Measurements)> {
        vec![
            (
                Cell::of(Lines::left("abcdef\nghijkl\nmnopqr\nstuvwx"))
                    .with_dim(Dimension::new(6, 5)),
                Measurements::new(
                    Dimension::new(6, 5),
                    MeasurementSpecifics::Child(Box::new(Dimension::new(6, 4).into())),
                ),
            ),
            (
                Cell::of(Lines::left("123\n456\n789"))
                    .with_anchor(CellAnchor::Center)
                    .with_dim(Dimension::new(3, 5)),
                Measurements::new(
                    Dimension::new(3, 5),
                    MeasurementSpecifics::Child(Box::new(Dimension::new(3, 3).into())),
                ),
            ),
            (
                Cell::of(Lines::left("ABCDE\nFGHIJ\nKLMNO\nPQRST\nUVWXY"))
                    .with_dim(Dimension::new(5, 5)),
                Measurements::new(
                    Dimension::new(5, 5),
                    MeasurementSpecifics::Child(Box::new(Dimension::new(5, 5).into())),
                ),
            ),
        ]
    }

    #[test]
    fn formatted_row_no_clip_no_fill_rows() {
        // Arrange
        let cells = sample_cells();

        // Act
        let options = LayoutOptions::new(Dimension::new(20, 6), false, WrapMode::default(), None);
        let formatted: BoxedFormattedLayout = FormattedRow::new(cells, options).into();

        assert_eq!(
            format!("{formatted}"),
            concat!(
                "abcdef   ABCDE\n",
                "ghijkl123FGHIJ\n",
                "mnopqr456KLMNO\n",
                "stuvwx789PQRST\n",
                "         UVWXY\n",
                "\n"
            )
        )
    }

    #[test]
    fn formatted_row_no_clip_fill_rows() {
        // Arrange
        let cells = sample_cells();

        // Act
        let options = LayoutOptions::new(Dimension::new(20, 6), true, WrapMode::default(), None);
        let formatted: BoxedFormattedLayout = FormattedRow::new(cells, options).into();

        assert_eq!(
            format!("{formatted}"),
            concat!(
                "abcdef   ABCDE      \n",
                "ghijkl123FGHIJ      \n",
                "mnopqr456KLMNO      \n",
                "stuvwx789PQRST      \n",
                "         UVWXY      \n",
                "                    \n"
            )
        )
    }

    #[test]
    fn formatted_row_with_clip_no_fill_rows() {
        // Arrange
        let cells = sample_cells();

        // Act
        let options = LayoutOptions::new(
            Dimension::new(20, 6),
            false,
            WrapMode::default(),
            Some(Rect::new(2, 1, Dimension::new(8, 3))),
        );
        let formatted: BoxedFormattedLayout = FormattedRow::new(cells, options).into();

        assert_eq!(
            format!("{formatted}"),
            concat!("ijkl123F\n", "opqr456K\n", "uvwx789P\n",)
        )
    }

    #[test]
    fn formatted_row_with_clip_fill_rows() {
        // Arrange
        let cells = sample_cells();

        // Act
        let options = LayoutOptions::new(
            Dimension::new(20, 6),
            true,
            WrapMode::default(),
            Some(Rect::new(2, 1, Dimension::new(15, 5))),
        );
        let formatted: BoxedFormattedLayout = FormattedRow::new(cells, options).into();

        assert_eq!(
            format!("{formatted}"),
            concat!(
                "ijkl123FGHIJ   \n",
                "opqr456KLMNO   \n",
                "uvwx789PQRST   \n",
                "       UVWXY   \n",
                "               \n"
            )
        )
    }
}
