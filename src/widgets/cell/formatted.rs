use crate::ext::{
    BaseLayoutWriter, BoxedLayoutWriter, FormattedLayout, LayoutWriter, RangeExt, SizedLayoutResult,
};
use crate::{BoxedFormattedLayout, LayoutOptions, box_formatted_layout};
use std::fmt::Write;
use std::ops::Range;

/// [`FormattedLayout`] implementation for the [`Cell`].
pub(crate) struct FormattedCell<'fmt> {
    content: BoxedFormattedLayout<'fmt>,
    padding: (usize, usize),
    options: LayoutOptions,
}

impl<'fmt> FormattedCell<'fmt> {
    /// Creates a new instance
    /// When creating such an instance, all computations regarding clipping and sizing are already
    /// done - this means that the caller has to ensure by its own that the parameters correctly
    /// fit together. In detail:
    /// - The `options` typically have no own clipping.
    /// - The `content` has already been clipped to the visible area.
    /// - The `padding` also takes the clipping into account.
    ///
    /// # Parameters
    /// - `content`: The [`FormattedLayout`] for the cell's content.
    /// - `padding`: Padding for the content. This is the number of spaces on the left side/empty
    ///   lines at the top before the cell's content.
    /// - `options`: The [`LayoutOptions`] to use.
    pub(crate) fn new(
        content: BoxedFormattedLayout<'fmt>,
        padding: (usize, usize),
        options: LayoutOptions,
    ) -> Self {
        Self {
            content,
            padding,
            options,
        }
    }
}

impl FormattedLayout for FormattedCell<'_> {
    fn options(&self) -> &LayoutOptions {
        &self.options
    }

    fn new_writer(&'_ self) -> BoxedLayoutWriter<'_> {
        Box::new(CellWriter::new(
            self.content.new_writer(),
            self.padding,
            &self.options,
        ))
    }
}

box_formatted_layout!(FormattedCell);

struct CellWriter<'wrt> {
    base: BaseLayoutWriter<'wrt>,
    content: BoxedLayoutWriter<'wrt>,
    content_row_range: Range<usize>,
    padding: (usize, usize),
}

impl<'wrt> CellWriter<'wrt> {
    fn new(
        content: BoxedLayoutWriter<'wrt>,
        padding: (usize, usize),
        options: &'wrt LayoutOptions,
    ) -> Self {
        let visible_content_rows = content.options().visible_rect().y_range();
        let mut this = Self {
            base: BaseLayoutWriter::new(options),
            content,
            content_row_range: visible_content_rows.normalize().add_offset(padding.1),
            padding,
        };
        for _ in 0..this.content.options().visible_rect().y_range().start {
            this.content.write_row(&mut NullWrite).unwrap();
        }
        this
    }
}
impl<'wrt> LayoutWriter<'wrt> for CellWriter<'wrt> {
    fn options(&self) -> &'wrt LayoutOptions {
        self.base.options()
    }

    fn write_row(&mut self, w: &mut dyn Write) -> SizedLayoutResult {
        let row = self.base.row();
        if self.content_row_range.contains(&row) {
            self.base.write_spaces(self.padding.0, w)?;
            self.base.write_row(self.content.as_mut(), w)?;
        }
        self.base.end_row(w)
    }
}

struct NullWrite;
impl Write for NullWrite {
    fn write_str(&mut self, _s: &str) -> std::fmt::Result {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::ext::{FormattedLayout, LayoutWithOptions, LayoutWriter};
    use crate::widgets::Lines;
    use crate::widgets::cell::formatted::CellWriter;
    use crate::{Dimension, LayoutOptions};

    #[test]
    fn cell_writer() {
        // Arrange
        let content = LayoutWithOptions::of(
            Lines::left("abcde\nfghij\nklmno\npqrst\nuvwxy").into(),
            LayoutOptions::default().with_dim(Dimension::new(5, 5)),
        );

        // No fill rows
        let options = LayoutOptions::default()
            .with_fill_rows(false)
            .with_dim(Dimension::new(11, 7));
        let content_writer = content.new_writer();
        let mut writer = CellWriter::new(content_writer, (2, 1), &options);
        let mut result = String::new();
        (0..options.dim.height).for_each(|_| {
            _ = writer.write_row(&mut result);
            result.push('\n');
        });
        assert_eq!(
            result,
            concat!(
                "\n",        //
                "  abcde\n", //
                "  fghij\n", //
                "  klmno\n", //
                "  pqrst\n", //
                "  uvwxy\n", //
                "\n",        //
            )
        );

        // Fill rows
        let options = LayoutOptions::default()
            .with_fill_rows(true)
            .with_dim(Dimension::new(11, 7));
        let content_writer = content.new_writer();
        let mut writer = CellWriter::new(content_writer, (2, 1), &options);
        let mut result = String::new();
        (0..options.dim.height).for_each(|_| {
            _ = writer.write_row(&mut result);
            result.push('\n');
        });
        assert_eq!(
            result,
            concat!(
                "           \n", //
                "  abcde    \n", //
                "  fghij    \n", //
                "  klmno    \n", //
                "  pqrst    \n", //
                "  uvwxy    \n", //
                "           \n", //
            )
        );
    }
}
