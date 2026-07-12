use crate::core::layout::SizedLayoutResult;
use crate::ext::{DisplayStr, LayoutWriter, Style, Transition};
use crate::{Dimension, LayoutOptions, Rect, WrapMode};
use std::cmp::min;
use std::fmt::{Error, Write};
use std::ops::Range;

/// A foundation for implementing [`LayoutWriter`] that provides common functionality.
///
/// `BaseLayoutWriter` handles core responsibilities such as:
/// - Tracking the current cursor position (row and column)
/// - Managing [`LayoutOptions`] and clipping rectangles
/// - Handling style transitions and ANSI escape sequences
/// - Providing helper methods for writing strings and spaces with wrapping/truncation support
///
/// Many custom `LayoutWriter` implementations use `BaseLayoutWriter` as a foundation
/// by delegating basic writing tasks to it.
#[derive(Debug)]
pub struct BaseLayoutWriter<'wrt> {
    options: &'wrt LayoutOptions,
    clip: Rect,
    row: usize,
    col: usize,
    style: Style,
    style_written: bool,
}

impl<'wrt> BaseLayoutWriter<'wrt> {
    /// Creates a new instance based on the provided [`LayoutOptions`].
    ///
    /// # Parameters
    /// - `options`: The layout options to use for this writer.
    ///
    /// # Returns
    /// A new `BaseLayoutWriter` instance.
    #[must_use]
    pub fn new(options: &'wrt LayoutOptions) -> Self {
        Self {
            options,
            clip: options.visible_rect(),
            row: 0,
            col: 0,
            style: Style::default(),
            style_written: false,
        }
    }

    /// Gets the associated [`LayoutOptions`].
    ///
    /// # Returns
    /// The options
    #[must_use]
    pub fn options(&self) -> &'wrt LayoutOptions {
        self.options
    }

    /// Gets the current row index.
    ///
    /// Counting starts with `0` and is incremented each time
    /// [`end_row()`](BaseLayoutWriter::end_row) is called.
    ///
    /// # Returns
    /// The row index.
    #[must_use]
    pub fn row(&self) -> usize {
        self.row
    }

    /// Gets the current column index.
    ///
    /// Counting starts with `0`. It is incremented each time
    /// [`consume_width()`](BaseLayoutWriter::consume_width) is called and reset to `0`
    /// when [`end_row()`](BaseLayoutWriter::end_row) is called.
    ///
    /// # Returns
    /// The column index.
    #[must_use]
    pub fn col(&self) -> usize {
        self.col
    }

    /// Returns the maximum height.
    ///
    /// This is identical to the `height` of the `Dimension` of the current `LayoutOptions`.
    ///
    /// # Returns
    /// The maximum height to be output overall by this writer.
    #[must_use]
    pub fn max_height(&self) -> usize {
        self.options.dim.height
    }

    /// Returns the maximum width.
    ///
    /// This is identical to the `width` of the `Dimension` of the current `LayoutOptions`.
    ///
    /// # Returns
    /// The maximum number of columns to be output overall by this writer.
    #[must_use]
    pub fn max_width(&self) -> usize {
        self.options.dim.width
    }

    /// Returns the [`WrapMode`].
    ///
    /// This is identical to the `wrap_mode` of the current `LayoutOptions`.
    ///
    /// # Returns
    /// The `WrapMode`
    #[must_use]
    pub fn wrap_mode(&self) -> WrapMode {
        self.options.wrap_mode
    }

    /// Returns the number of available columns to be written in the current row.
    ///
    /// This is basically just the difference between [`max_width()`](BaseLayoutWriter::max_width)
    /// and [`col()`](BaseLayoutWriter::col).
    ///
    /// # Returns
    /// The number of characters that might be written in the current row without causing an overflow.
    #[must_use]
    pub fn available_width(&self) -> usize {
        self.max_width() - self.col
    }

    /// Writes the current row of `delegate` and updates the internal state accordingly.
    ///
    /// This method is useful for `LayoutWriter`s that delegate parts of the actual output
    /// operation to one or more subordinated writers.
    ///
    /// # Parameters
    /// - `delegate`: The `LayoutWriter` that will be used to write in the current row.
    /// - `w`: The `Write` implementation that will be used to write the row.
    ///
    /// # Returns
    /// The number of characters physically written in this row.
    ///
    /// # Errors
    /// Returns an error if writing to the underlying `Write` implementation fails or if there is
    /// not enough space.
    pub fn write_row(
        &mut self,
        delegate: &mut dyn LayoutWriter,
        w: &mut dyn Write,
    ) -> SizedLayoutResult {
        let len = delegate.write_row(w)?;
        self.consume_width(len)?;
        Ok(len)
    }

    /// Writes the given `style`.
    ///
    /// # Parameters
    /// - `style`: The style to write
    ///
    /// # Returns
    /// An empty result
    ///
    /// # Errors
    /// Returns an error if writing to the underlying `Write` implementation fails.
    pub fn write_style(&mut self, style: Style, w: &mut dyn Write) -> VoidLayoutResult {
        self.update_style(style, w)
    }

    /// Writes `text` with the given `wrap_mode`.
    ///
    /// The method handles text wrapping and truncation based on available columns and wrap mode.
    /// It guarantees that the text will fit within the available columns; only in case there is
    /// absolutely no space (available columns is zero) will the method fail.
    ///
    /// # Parameters
    /// - `text`: The text to be written.
    /// - `wrap_mode`: The [`WrapMode`] to apply
    /// - `w`: The [`Write`] to write
    ///
    /// # Returns
    /// In case of [`WrapMode::Wrap`] it returns the portion of `text` that cannot be written, if
    /// wrapping takes place. In all other success-cases an empty string is returned.
    ///
    /// # Errors
    /// Returns an error if writing to the underlying `Write` implementation fails or if there is
    /// not enough space.
    pub fn write_str<'text>(
        &mut self,
        text: &'text str,
        wrap_mode: WrapMode,
        w: &mut dyn Write,
    ) -> StrLayoutResult<'text> {
        self.write_str_with_len(text, self.available_width(), wrap_mode, w)
    }

    /// Writes at most `text_len` characters of `text` with the given `wrap_mode`.
    ///
    /// The method handles text wrapping and truncation based on available columns and wrap mode.
    /// It guarantees that the text will fit within the `text_len` columns; only in case there is
    /// absolutely no space (available columns is zero) will the method fail.
    ///
    /// # Parameters
    /// - `text`: The text to be written.
    /// - `text_len`: The number of characters to write.
    /// - `wrap_mode`: The [`WrapMode`] to apply
    /// - `w`: The [`Write`] to write
    ///
    /// # Returns
    /// In case of [`WrapMode::Wrap`] it returns the portion of `text` that cannot be written, if
    /// wrapping takes place. In all other success-cases an empty string is returned.
    ///
    /// # Errors
    /// Returns an error if writing to the underlying `Write` implementation fails or if there is
    /// not enough space.
    pub fn write_str_with_len<'text>(
        &mut self,
        text: &'text str,
        text_len: usize,
        wrap_mode: WrapMode,
        w: &mut dyn Write,
    ) -> StrLayoutResult<'text> {
        let available = min(text_len, self.available_width());
        let len = text.display_len();
        if available >= len {
            self.write_internal(text, w)?;
            return Ok("");
        }
        if available == 0 {
            return Err(Error);
        }

        match wrap_mode {
            WrapMode::Truncate(indicator) => {
                let indicator_len = std::cmp::min(indicator.display_len(), available - 1);
                let (tbw, rest) = text.display_split_at(available - indicator_len);
                self.write_internal(tbw, w)?;
                self.write_internal(indicator.display_split_at(indicator_len).0, w)?;

                let end_style = self.style.transition_for(rest);
                end_style.render(w)?;
                self.style = end_style.after;
                Ok("")
            }
            WrapMode::Wrap => {
                let (tbw, rest) = text.display_split_at(available);
                self.write_internal(tbw, w)?;
                Ok(rest)
            }
        }
    }

    /// Indicates that the end of the row has been reached.
    ///
    /// It increments the row counter, sets the current column to 0, and handles the style transition
    /// if required. It also fills the remaining columns with spaces if required.
    ///
    /// # Parameters
    /// - `w` - The [`Write`] to use
    ///
    /// # Returns
    /// Ok with the number of chars physically written in this row or Err
    ///
    /// # Errors
    /// Returns an error if writing to the underlying `Write` implementation fails or if there is
    /// not enough space.
    pub fn end_row(&mut self, w: &mut dyn Write) -> SizedLayoutResult {
        // fill spaces, if required
        if self.options.fill_rows {
            self.write_spaces(self.available_width(), w)?;
        }

        // Reset style if necessary
        if self.style_written {
            let transition = Transition::new(self.style, Style::default());
            transition.render(w)?;
        }

        // Do the other math
        let physically_written = self.clip_range(0, self.col).map_or(0, |r| r.end - r.start);
        self.row += 1;
        self.col = 0;
        self.style_written = false;
        Ok(physically_written)
    }

    /// Writes `count` spaces to the output.
    ///
    /// The method fails if `count` is greater than the available columns.
    ///
    /// # Parameters
    /// - `count` - The number of spaces to write
    ///
    /// # Return
    /// Ok or Err
    ///
    /// # Errors
    /// Returns an error if writing to the underlying `Write` implementation fails or if there is
    /// not enough space.
    pub fn write_spaces(&mut self, count: usize, w: &mut dyn Write) -> SizedLayoutResult {
        //self.write_internal(&" ".repeat(count), w)
        self.write_repeated(' ', count, w)
    }

    /// Writes `ch` `count` times to the output.
    ///
    /// The method fails if `count` is greater than the available columns.
    ///
    /// # Parameters
    /// - `ch` - The character to write
    /// - `count` - The number of spaces to write
    ///
    /// # Return
    /// Ok or Err
    ///
    /// # Errors
    /// Returns an error if writing to the underlying `Write` implementation fails or if there is
    /// not enough space.
    pub fn write_repeated(
        &mut self,
        ch: char,
        count: usize,
        w: &mut dyn Write,
    ) -> SizedLayoutResult {
        // Precondition: text must fit
        // Main objectives:
        // 1. Handle clipping
        // 2. Write style transitions

        // Consume columns
        self.consume_width(count)?;

        if let Some(clip) = self.clip_range(self.col - count, count) {
            // If not already done, write initial style
            if !self.style_written {
                self.style_written = true;
                self.style.render(w)?;
            }

            // Write the repeated character inside the cip
            for _ in 0..clip.end - clip.start {
                w.write_char(ch)?;
            }
        }
        Ok(count)
    }

    fn write_internal(&mut self, text: &str, w: &mut dyn Write) -> SizedLayoutResult {
        // Precondition: text must fit
        // Main objectives:
        // 1. Handle clipping
        // 2. Write style transitions

        // Consume columns
        if text.is_empty() {
            return Ok(0);
        }
        let len = text.display_len();
        if len == 0 {
            self.update_style(self.style.with_text(text), w)?;
            return Ok(0);
        }
        self.consume_width(len)?;

        if let Some(clip) = self.clip_range(self.col - len, len) {
            // If not already done, write initial style
            if !self.style_written {
                self.style_written = true;
                self.style.render(w)?;
            }

            // Handle part before clip, if any
            let rest = if clip.start > 0 {
                let (before_clip, rest) = text.display_split_at(clip.start);
                self.update_style(self.style.with_text(before_clip), w)?;
                rest
            } else {
                text
            };

            if clip.end < len {
                // Write the part in clip
                let (inclip, after_clip) = rest.display_split_at(clip.end - clip.start);
                w.write_str(inclip)?;
                self.style = self.style.with_text(inclip);

                // Update the style for the part after the clip
                self.update_style(self.style.with_text(after_clip), w)?;
            } else {
                w.write_str(rest)?;
                self.style = self.style.with_text(rest);
            }
        } else {
            // Content completely outside clipping - We only need to take care about the style
            self.update_style(self.style.with_text(text), w)?;
        }
        Ok(len)
    }

    /// Marks `count` columns as consumed and increments [`col()`](BaseLayoutWriter::col)
    /// accordingly.
    ///
    /// If there are insufficient columns available, the method fails. This method
    /// is called internally by all methods that emit characters and must be called by all methods
    /// that emit characters on their own.
    ///
    /// # Parameters
    /// * `count` - The number of columns to consume
    ///
    /// # Returns
    /// `Ok` if there were enough columns, or `Err` if not. If `Ok` the returned size is the number
    /// of columns logically consumed.
    ///
    /// # Errors
    /// Returns an error  if there is not enough space.
    pub fn consume_width(&mut self, count: usize) -> VoidLayoutResult {
        if count > self.available_width() {
            return Err(Error);
        }
        self.col += count;
        Ok(())
    }

    fn update_style(&mut self, new_style: Style, w: &mut dyn Write) -> VoidLayoutResult {
        if self.style_written {
            // We have to write the style transition
            let transition = Transition::new(self.style, new_style);
            transition.render(w)?;
        }
        self.style = new_style;
        Ok(())
    }

    fn clip_range(&self, col: usize, len: usize) -> Option<Range<usize>> {
        let rect = Rect::new(col, self.row, Dimension::new(len, 1)).intersect_relative(self.clip);
        if rect.is_empty() {
            None
        } else {
            Some(rect.x_range())
        }
    }
}

/// Represents a void result of a layout operation
pub type VoidLayoutResult = std::fmt::Result;

/// Represents a string result of a layout operation
pub type StrLayoutResult<'text> = Result<&'text str, Error>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ext::Effect;

    #[derive(PartialEq, Debug)]
    struct BaseLayoutWriterState {
        col: usize,
        row: usize,
        style: Style,
        style_written: bool,
    }

    impl BaseLayoutWriterState {
        fn new(col: usize, row: usize, style: Style, style_written: bool) -> Self {
            Self {
                col,
                row,
                style,
                style_written,
            }
        }

        fn apply(&self, writer: &mut BaseLayoutWriter) {
            writer.col = self.col;
            writer.row = self.row;
            writer.style = self.style;
            writer.style_written = self.style_written;
        }
    }

    impl From<&BaseLayoutWriter<'_>> for BaseLayoutWriterState {
        fn from(value: &BaseLayoutWriter) -> Self {
            Self::new(value.col, value.row, value.style, value.style_written)
        }
    }

    #[test]
    fn base_layout_writer_write_str_wrap() {
        let options = LayoutOptions::new(Dimension::new(10, 5), false, WrapMode::default(), None);
        let mut writer = BaseLayoutWriter::new(&options);

        let mut out = String::new();

        assert_eq!(
            writer.write_str("0123456789abcdef", WrapMode::Wrap, &mut out),
            Ok("abcdef")
        );
        assert_eq!(out, "0123456789");
    }

    #[test]
    fn base_layout_writer_write_str_truncate() {
        let options = LayoutOptions::new(Dimension::new(10, 5), false, WrapMode::default(), None);
        let mut writer = BaseLayoutWriter::new(&options);

        // Normal case
        let mut out = String::new();
        assert_eq!(
            writer.write_str("0123456789abcdef", WrapMode::Truncate("..."), &mut out),
            Ok("")
        );
        assert_eq!(out, "0123456...");

        // One character left for truncation indicator
        let mut out = String::new();
        writer.col = 8;
        assert_eq!(
            writer.write_str("0123456789abcdef", WrapMode::Truncate("..."), &mut out),
            Ok("")
        );
        assert_eq!(out, "0.");

        // No character left for truncation indicator
        let mut out = String::new();
        writer.col = 9;
        assert_eq!(
            writer.write_str("0123456789abcdef", WrapMode::Truncate("..."), &mut out),
            Ok("")
        );
        assert_eq!(out, "0");
    }

    #[test]
    fn base_layout_writer_write_str_fit() {
        let options = LayoutOptions::new(Dimension::new(10, 5), false, WrapMode::default(), None);
        let mut writer = BaseLayoutWriter::new(&options);

        let mut out = String::new();

        assert_eq!(
            writer.write_str("0123456789", WrapMode::Wrap, &mut out),
            Ok("")
        );
        assert_eq!(out, "0123456789");
    }

    #[test]
    fn base_layout_writer_write_str_fail() {
        let options = LayoutOptions::new(Dimension::new(10, 5), false, WrapMode::default(), None);
        let mut writer = BaseLayoutWriter::new(&options);
        writer.col = 10;

        let mut out = String::new();
        assert_eq!(
            writer.write_str("test", WrapMode::Wrap, &mut out),
            Err(Error)
        );
        assert_eq!(out, "");
    }

    #[test]
    fn base_layout_writer_end_row_no_fill() {
        let options = LayoutOptions::new(Dimension::new(10, 5), false, WrapMode::default(), None);
        let mut writer = BaseLayoutWriter::new(&options);

        // style_written = false
        let state =
            BaseLayoutWriterState::new(5, 1, Style::default().with_effect(Effect::Bold), false);
        let expected =
            BaseLayoutWriterState::new(0, 2, Style::default().with_effect(Effect::Bold), false);
        let mut out = String::new();
        state.apply(&mut writer);
        assert_eq!(writer.end_row(&mut out), Ok(5));
        assert_eq!(out, "");
        assert_eq!(BaseLayoutWriterState::from(&writer), expected);

        // style_written = true
        let state =
            BaseLayoutWriterState::new(5, 1, Style::default().with_effect(Effect::Bold), true);
        let expected =
            BaseLayoutWriterState::new(0, 2, Style::default().with_effect(Effect::Bold), false);
        let mut out = String::new();
        state.apply(&mut writer);
        assert_eq!(writer.end_row(&mut out), Ok(5));
        assert_eq!(out, "\x1b[0m");
        assert_eq!(BaseLayoutWriterState::from(&writer), expected);
    }

    #[test]
    fn base_layout_writer_end_row_with_fill() {
        let options = LayoutOptions::new(Dimension::new(10, 5), true, WrapMode::default(), None);
        let mut writer = BaseLayoutWriter::new(&options);

        // style_written = false
        let state =
            BaseLayoutWriterState::new(5, 1, Style::default().with_effect(Effect::Bold), false);
        let expected =
            BaseLayoutWriterState::new(0, 2, Style::default().with_effect(Effect::Bold), false);
        let mut out = String::new();
        state.apply(&mut writer);
        assert_eq!(writer.end_row(&mut out), Ok(10));
        assert_eq!(out, "\x1b[1m     \x1b[0m");
        assert_eq!(BaseLayoutWriterState::from(&writer), expected);

        // style_written = true
        let state =
            BaseLayoutWriterState::new(5, 1, Style::default().with_effect(Effect::Bold), true);
        let expected =
            BaseLayoutWriterState::new(0, 2, Style::default().with_effect(Effect::Bold), false);
        let mut out = String::new();
        state.apply(&mut writer);
        assert_eq!(writer.end_row(&mut out), Ok(10));
        assert_eq!(out, "     \x1b[0m");
        assert_eq!(BaseLayoutWriterState::from(&writer), expected);
    }

    #[test]
    fn base_layout_writer_write_spaces() {
        let options = LayoutOptions::new(Dimension::new(10, 5), true, WrapMode::default(), None);
        let mut writer = BaseLayoutWriter::new(&options);

        // Ok
        let state = BaseLayoutWriterState::new(0, 0, Style::default(), false);
        let expected = BaseLayoutWriterState::new(6, 0, Style::default(), true);
        let mut out = String::new();
        state.apply(&mut writer);
        assert_eq!(writer.write_spaces(6, &mut out), Ok(6));
        assert_eq!(out, "      ");
        assert_eq!(BaseLayoutWriterState::from(&writer), expected);

        // Fail
        let state = BaseLayoutWriterState::new(0, 0, Style::default(), false);
        let expected = BaseLayoutWriterState::new(0, 0, Style::default(), false);
        let mut out = String::new();
        state.apply(&mut writer);
        assert_eq!(writer.write_spaces(12, &mut out), Err(Error));
        assert_eq!(out, "");
        assert_eq!(BaseLayoutWriterState::from(&writer), expected);
    }

    #[test]
    fn base_layout_writer_write_internal_no_output() {
        let options = LayoutOptions::new(
            Dimension::new(10, 5),
            true,
            WrapMode::default(),
            Some(Rect::new(1, 1, Dimension::empty())),
        );
        let mut writer = BaseLayoutWriter::new(&options);

        // style_written = false
        let state =
            BaseLayoutWriterState::new(0, 0, Style::default().with_effect(Effect::Bold), false);
        let expected = BaseLayoutWriterState::new(
            6,
            0,
            Style::default()
                .with_effect(Effect::Bold)
                .with_effect(Effect::Dim)
                .with_effect(Effect::Italic)
                .with_effect(Effect::Underline),
            false,
        );
        let mut out = String::new();
        state.apply(&mut writer);
        assert_eq!(
            writer.write_internal("\x1b[2mab\x1b[3mcd\x1b[4mef", &mut out),
            Ok(6)
        );
        assert_eq!(out, "");
        assert_eq!(BaseLayoutWriterState::from(&writer), expected);

        // style_written = true
        let state =
            BaseLayoutWriterState::new(0, 0, Style::default().with_effect(Effect::Bold), true);
        let expected = BaseLayoutWriterState::new(
            6,
            0,
            Style::default()
                .with_effect(Effect::Bold)
                .with_effect(Effect::Dim)
                .with_effect(Effect::Italic)
                .with_effect(Effect::Underline),
            true,
        );
        let mut out = String::new();
        state.apply(&mut writer);
        assert_eq!(
            writer.write_internal("\x1b[2mab\x1b[3mcd\x1b[4mef", &mut out),
            Ok(6)
        );
        assert_eq!(out, "\x1b[2;3;4m");
        assert_eq!(BaseLayoutWriterState::from(&writer), expected);
    }

    #[test]
    fn base_layout_writer_write_internal_part_output() {
        let options = LayoutOptions::new(
            Dimension::new(10, 5),
            true,
            WrapMode::default(),
            Some(Rect::new(4, 2, Dimension::new(4, 2))),
        );
        let mut writer = BaseLayoutWriter::new(&options);

        // style_written = false
        let state =
            BaseLayoutWriterState::new(3, 2, Style::default().with_effect(Effect::Bold), false);
        let expected = BaseLayoutWriterState::new(
            9,
            2,
            Style::default()
                .with_effect(Effect::Bold)
                .with_effect(Effect::Dim)
                .with_effect(Effect::Italic)
                .with_effect(Effect::Underline),
            true,
        );
        let mut out = String::new();
        state.apply(&mut writer);
        assert_eq!(
            writer.write_internal("\x1b[2mab\x1b[3mcd\x1b[4mef", &mut out),
            Ok(6)
        );
        assert_eq!(out, "\x1b[1m\x1b[2mb\x1b[3mcd\x1b[4me");
        assert_eq!(BaseLayoutWriterState::from(&writer), expected);

        // style_written = true
        let state =
            BaseLayoutWriterState::new(3, 2, Style::default().with_effect(Effect::Bold), true);
        let expected = BaseLayoutWriterState::new(
            9,
            2,
            Style::default()
                .with_effect(Effect::Bold)
                .with_effect(Effect::Dim)
                .with_effect(Effect::Italic)
                .with_effect(Effect::Underline),
            true,
        );
        let mut out = String::new();
        state.apply(&mut writer);
        assert_eq!(
            writer.write_internal("\x1b[2mab\x1b[3mcd\x1b[4mef", &mut out),
            Ok(6)
        );
        assert_eq!(out, "\x1b[2mb\x1b[3mcd\x1b[4me");
        assert_eq!(BaseLayoutWriterState::from(&writer), expected);
    }

    #[test]
    fn base_layout_writer_write_internal_full_output() {
        let options = LayoutOptions::new(Dimension::new(10, 5), true, WrapMode::default(), None);
        let mut writer = BaseLayoutWriter::new(&options);

        // style_written = false
        let state =
            BaseLayoutWriterState::new(3, 2, Style::default().with_effect(Effect::Bold), false);
        let expected = BaseLayoutWriterState::new(
            9,
            2,
            Style::default()
                .with_effect(Effect::Bold)
                .with_effect(Effect::Dim)
                .with_effect(Effect::Italic)
                .with_effect(Effect::Underline),
            true,
        );
        let mut out = String::new();
        state.apply(&mut writer);
        assert_eq!(
            writer.write_internal("\x1b[2mab\x1b[3mcd\x1b[4mef", &mut out),
            Ok(6)
        );
        assert_eq!(out, "\x1b[1m\x1b[2mab\x1b[3mcd\x1b[4mef");
        assert_eq!(BaseLayoutWriterState::from(&writer), expected);

        // style_written = true
        let state =
            BaseLayoutWriterState::new(3, 2, Style::default().with_effect(Effect::Bold), true);
        let expected = BaseLayoutWriterState::new(
            9,
            2,
            Style::default()
                .with_effect(Effect::Bold)
                .with_effect(Effect::Dim)
                .with_effect(Effect::Italic)
                .with_effect(Effect::Underline),
            true,
        );
        let mut out = String::new();
        state.apply(&mut writer);
        assert_eq!(
            writer.write_internal("\x1b[2mab\x1b[3mcd\x1b[4mef", &mut out),
            Ok(6)
        );
        assert_eq!(out, "\x1b[2mab\x1b[3mcd\x1b[4mef");
        assert_eq!(BaseLayoutWriterState::from(&writer), expected);
    }

    #[test]
    fn base_layout_writer_consume_width() {
        let options = LayoutOptions::new(Dimension::new(10, 5), true, WrapMode::default(), None);
        let mut writer = BaseLayoutWriter::new(&options);

        // Succces
        let state = BaseLayoutWriterState::new(3, 2, Style::default(), false);
        let expected = BaseLayoutWriterState::new(9, 2, Style::default(), false);
        state.apply(&mut writer);
        assert_eq!(writer.consume_width(6), Ok(()));
        assert_eq!(BaseLayoutWriterState::from(&writer), expected);

        // Fail
        let state = BaseLayoutWriterState::new(3, 2, Style::default(), false);
        let expected = BaseLayoutWriterState::new(3, 2, Style::default(), false);
        state.apply(&mut writer);
        assert_eq!(writer.consume_width(8), Err(Error));
        assert_eq!(BaseLayoutWriterState::from(&writer), expected);
    }

    #[test]
    fn base_layout_writer_update_style() {
        let options = LayoutOptions::new(Dimension::new(10, 5), true, WrapMode::default(), None);
        let mut writer = BaseLayoutWriter::new(&options);

        // No Style transition
        let state =
            BaseLayoutWriterState::new(0, 0, Style::default().with_effect(Effect::Bold), false);
        let expected =
            BaseLayoutWriterState::new(0, 0, Style::default().with_effect(Effect::Bold), false);
        let mut out = String::new();
        state.apply(&mut writer);
        assert_eq!(
            writer.update_style(Style::default().with_effect(Effect::Bold), &mut out),
            Ok(())
        );
        assert_eq!(out, "");
        assert_eq!(BaseLayoutWriterState::from(&writer), expected);

        // Style transition, nothing written yet
        let state =
            BaseLayoutWriterState::new(0, 0, Style::default().with_effect(Effect::Bold), false);
        let expected =
            BaseLayoutWriterState::new(0, 0, Style::default().with_effect(Effect::Italic), false);
        let mut out = String::new();
        state.apply(&mut writer);
        assert_eq!(
            writer.update_style(Style::default().with_effect(Effect::Italic), &mut out),
            Ok(())
        );
        assert_eq!(out, "");
        assert_eq!(BaseLayoutWriterState::from(&writer), expected);

        // Style transition, sth already written
        let state =
            BaseLayoutWriterState::new(0, 0, Style::default().with_effect(Effect::Bold), true);
        let expected =
            BaseLayoutWriterState::new(0, 0, Style::default().with_effect(Effect::Italic), true);
        let mut out = String::new();
        state.apply(&mut writer);
        assert_eq!(
            writer.update_style(Style::default().with_effect(Effect::Italic), &mut out),
            Ok(())
        );
        assert_eq!(out, "\x1b[22;3m");
        assert_eq!(BaseLayoutWriterState::from(&writer), expected);
    }

    #[test]
    fn base_layout_writer_clip_range() {
        let options = LayoutOptions::new(Dimension::new(10, 5), true, WrapMode::default(), None);
        let mut writer = BaseLayoutWriter::new(&options);

        // No clipping
        writer.row = 3;
        assert_eq!(writer.clip_range(0, 10), Some(0..10));
        assert_eq!(writer.clip_range(1, 8), Some(0..8));
        assert_eq!(writer.clip_range(3, 6), Some(0..6));
        writer.row = 5;
        assert_eq!(writer.clip_range(0, 10), None);
        assert_eq!(writer.clip_range(1, 8), None);
        assert_eq!(writer.clip_range(3, 6), None);

        // Clipping
        writer.clip = Rect::new(2, 1, Dimension::new(5, 3));
        writer.row = 3;
        assert_eq!(writer.clip_range(0, 10), Some(2..7));
        assert_eq!(writer.clip_range(1, 8), Some(1..6));
        assert_eq!(writer.clip_range(3, 6), Some(0..4));
    }
}
