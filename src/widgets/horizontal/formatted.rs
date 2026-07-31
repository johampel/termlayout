use crate::ext::{
    BaseLayoutWriter, BoxedLayoutWriter, FormattedLayout, LayoutWithContext, LayoutWriter,
    SizedLayoutResult,
};
use crate::widgets::Cell;
use crate::{BoxedFormattedLayout, LayoutContext, LayoutOptions, Measurements};
use std::fmt::Write;

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
            let cell: BoxedFormattedLayout = LayoutWithContext::of(cell.clone().into(), ctxt).into();
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
    use crate::widgets::horizontal::formatted::FormattedRow;
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
