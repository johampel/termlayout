use crate::ext::{
    BaseLayoutWriter, BoxedLayoutWriter, DisplayLines, DisplayStr, FormattedLayout, LayoutWriter,
    SizedLayoutResult, StrLayoutResult, Style, TakeBackIterator,
};
use crate::ext::{BoxedFormattedLayout, box_formatted_layout, rc_layout};
use crate::{Dimension, Layout, LayoutContext, LayoutOptions, MeasureMode, Measurements, WrapMode};
use std::any::Any;
use std::cmp::max;
use std::fmt::Write;

/// A widget that displays text line-by-line with horizontal alignment.
///
/// The content is represented as a string (which may contain ANSI control sequences for
/// terminal styling). Each newline character (`\n`) is treated as a line break.
/// The widget supports left, center, and right alignment.
///
/// # Example
/// ```rust
/// use termlayout::*;
/// use termlayout::widgets::Lines;
///
/// let lines = r#"These are some sample lines.
/// The Lines Layout can display them line-wise;
/// It applies wrapping and alignment."#;
///
/// let layout = Lines::left(lines);
/// assert_eq!(format!("{}", layout.layout_with_wrap_mode(20, WrapMode::default_truncate())),
/// concat!(
///     "These are some samp…\n",
///     "The Lines Layout ca…\n",
///     "It applies wrapping…\n",
/// ));
/// assert_eq!(format!("{}", layout.layout_with_wrap_mode(20, WrapMode::Wrap)),
/// concat!(
///     "These are some sampl\n",
///     "e lines.\n",
///     "The Lines Layout can\n",
///     " display them line-w\n",
///     "ise;\n",
///     "It applies wrapping \n",
///     "and alignment.\n",
/// ));
///
/// let layout = Lines::right(lines);
/// assert_eq!(format!("{}", layout.layout_with_wrap_mode(35, WrapMode::default_truncate())),
/// concat!(
///     "       These are some sample lines.\n",
///     "The Lines Layout can display them …\n",
///     " It applies wrapping and alignment.\n",
/// ));
/// ```
#[derive(Debug)]
pub struct Lines {
    /// The alignment of the lines.
    pub alignment: LinesAlignment,

    /// The content to be displayed in the layout.
    pub content: String,

    /// Defines the line trimming behavior, which by default trims the trailing whitespace
    ///   of each line.
    pub trimming: LinesTrimming,

    /// The initial style to be applied to the lines
    pub initial_style: Option<Style>,
}

impl Lines {
    /// Creates a new [`Lines`] layout with the given alignment.
    ///
    /// # Parameters
    /// - `alignment`: The horizontal alignment of the content.
    /// - `trimming`: The line trimming behavior
    /// - `initial_style`: The initial style to be applied to the lines
    /// - `content`: The content to be displayed in the layout.
    ///
    /// # Returns
    /// The new [`Lines`] layout.
    pub fn new<T: Into<String>>(
        alignment: LinesAlignment,
        trimming: LinesTrimming,
        initial_style: Option<Style>,
        content: T,
    ) -> Self {
        Self {
            alignment,
            trimming,
            initial_style,
            content: content.into(),
        }
    }

    /// Creates a new [`Lines`] layout with left alignment.
    ///
    /// # Parameters
    /// - `content`: The content to be displayed in the layout.
    ///
    /// # Returns
    /// The new [`Lines`] layout.
    pub fn left<T: Into<String>>(content: T) -> Self {
        Self::new(
            LinesAlignment::Left,
            LinesTrimming::default(),
            None,
            content,
        )
    }

    /// Creates a new [`Lines`] layout with left alignment and the given initial style.
    ///
    /// # Parameters
    /// - `initial_style`: The initial style to be applied to the lines
    /// - `content`: The content to be displayed in the layout.
    ///
    /// # Returns
    /// The new [`Lines`] layout.
    pub fn left_with_style<T: Into<String>>(initial_style: Style, content: T) -> Self {
        Self::new(
            LinesAlignment::Left,
            LinesTrimming::default(),
            Some(initial_style),
            content,
        )
    }

    /// Creates a new [`Lines`] layout with center alignment.
    ///
    /// # Parameters
    /// - `content`: The content to be displayed in the layout.
    ///
    /// # Returns
    /// The new [`Lines`] layout.
    pub fn center<T: Into<String>>(content: T) -> Self {
        Self::new(
            LinesAlignment::Center,
            LinesTrimming::default(),
            None,
            content,
        )
    }

    /// Creates a new [`Lines`] layout with center alignment and the given initial style.
    ///
    /// # Parameters
    /// - `initial_style`: The initial style to be applied to the lines
    /// - `content`: The content to be displayed in the layout.
    ///
    /// # Returns
    /// The new [`Lines`] layout.
    pub fn center_with_style<T: Into<String>>(initial_style: Style, content: T) -> Self {
        Self::new(
            LinesAlignment::Center,
            LinesTrimming::default(),
            Some(initial_style),
            content,
        )
    }

    /// Creates a new [`Lines`] layout with right alignment.
    ///
    /// # Parameters
    /// - `content`: The content to be displayed in the layout.
    ///
    /// # Returns
    /// The new [`Lines`] layout.
    pub fn right<T: Into<String>>(content: T) -> Self {
        Self::new(
            LinesAlignment::Right,
            LinesTrimming::default(),
            None,
            content,
        )
    }

    /// Creates a new [`Lines`] layout with right alignment and the given initial style.
    ///
    /// # Parameters
    /// - `initial_style`: The initial style to be applied to the lines
    /// - `content`: The content to be displayed in the layout.
    ///
    /// # Returns
    /// The new [`Lines`] layout.
    pub fn right_with_style<T: Into<String>>(initial_style: Style, content: T) -> Self {
        Self::new(
            LinesAlignment::Right,
            LinesTrimming::default(),
            Some(initial_style),
            content,
        )
    }

    fn lines_dim(&self, max_width: usize, fixed_width: bool, wrap_mode: WrapMode) -> Dimension {
        if max_width == 0 {
            return Dimension::empty();
        }

        let mut dim = self
            .content
            .display_lines()
            .map(|line| self.trimming.apply(line))
            .map(|line| wrap_mode.dimension_for(max_width, max(1, line.display_len())))
            .fold(Dimension::empty(), |acc, dim| acc.vertical_union(dim));
        if fixed_width {
            dim.width = max_width;
        }
        dim
    }
}

impl Default for Lines {
    fn default() -> Self {
        Self::new(LinesAlignment::Left, LinesTrimming::default(), None, "")
    }
}

impl Layout for Lines {
    fn measure(&self, mode: MeasureMode) -> Measurements {
        match mode {
            MeasureMode::Min => self.measure(MeasureMode::pref(usize::MAX, WrapMode::Truncate(""))),
            MeasureMode::Pref {
                max_width,
                wrap_mode,
            } => self.lines_dim(max_width, false, wrap_mode).into(),
            MeasureMode::FixedWidth { width, wrap_mode } => {
                self.lines_dim(width, true, wrap_mode).into()
            }
            MeasureMode::Exact { dimension, .. } => dimension.into(),
        }
    }

    fn layout_with_context(&'_ self, context: LayoutContext) -> BoxedFormattedLayout<'_> {
        FormattedLines::new(&self, context.into()).into()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

rc_layout!(Lines);

/// Defines the trimming behavior for lines.
/// The `LinesTrimming` is applied to each line of text before layout.The following values exist:
///
/// # Variants
/// - `None`: the lines are taken as they are
/// - `Both`: The leading and trailing whitespaces are removed before the line is being laid out
/// - `Leading`: The leading whitespaces are removed before the line is being laid out
/// - `Trailing`: The trailing whitespaces are removed before the line is being laid out
///
/// The `Trailing` variant is the default trimming behavior.
#[derive(Clone, Copy, Debug, Default)]
pub enum LinesTrimming {
    None,
    Both,
    Leading,
    #[default]
    Trailing,
}

impl LinesTrimming {
    fn apply(self, s: &str) -> &str {
        match self {
            LinesTrimming::None => s,
            LinesTrimming::Both => s.trim(),
            LinesTrimming::Leading => s.trim_start(),
            LinesTrimming::Trailing => s.trim_end(),
        }
    }
}

/// Represents horizontal alignment options for lines
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LinesAlignment {
    /// Aligns the content to the left.
    Left,

    /// Centers the content horizontally.
    Center,

    /// Aligns the content to the right.
    Right,
}

impl LinesAlignment {
    /// Calculates the number of columns on the left side of some text, if there rea `space` columns
    /// left to distribute for alignment.
    ///
    /// # Parameters
    /// * `space`: Number of columns to distribute
    ///
    /// # Returns
    /// Number of columns to be placed left of the text
    ///
    /// # Examples
    /// ```rust
    /// use termlayout::widgets::LinesAlignment;
    ///
    /// assert_eq!(LinesAlignment::Left.indent(5), 0);
    /// assert_eq!(LinesAlignment::Center.indent(5), 2);
    /// assert_eq!(LinesAlignment::Right.indent(5), 5);
    /// ```
    #[must_use]
    pub fn indent(&self, space: usize) -> usize {
        match self {
            LinesAlignment::Left => 0,
            LinesAlignment::Center => space / 2,
            LinesAlignment::Right => space,
        }
    }
}

struct FormattedLines<'fmt> {
    content: &'fmt Lines,
    options: LayoutOptions,
}

impl<'fmt> FormattedLines<'fmt> {
    fn new(content: &'fmt Lines, options: LayoutOptions) -> Self {
        Self { content, options }
    }
}

impl FormattedLayout for FormattedLines<'_> {
    fn options(&self) -> &LayoutOptions {
        &self.options
    }

    fn new_writer(&'_ self) -> BoxedLayoutWriter<'_> {
        Box::new(LinesWriter::new(
            self.content.alignment,
            self.content.trimming,
            self.content.initial_style,
            &self.content.content,
            &self.options,
        ))
    }
}

box_formatted_layout!(FormattedLines);

struct LinesWriter<'wrt> {
    base: BaseLayoutWriter<'wrt>,
    alignment: LinesAlignment,
    trimming: LinesTrimming,
    initial_style: Option<Style>,
    content: TakeBackIterator<DisplayLines<'wrt>>,
}

impl<'wrt> LinesWriter<'wrt> {
    fn new(
        alignment: LinesAlignment,
        trimming: LinesTrimming,
        initial_style: Option<Style>,
        content: &'wrt str,
        options: &'wrt LayoutOptions,
    ) -> Self {
        Self {
            base: BaseLayoutWriter::new(options),
            content: TakeBackIterator::new(content.display_lines()),
            initial_style,
            alignment,
            trimming,
        }
    }

    fn write_line(&mut self, line: &'wrt str, w: &mut dyn Write) -> StrLayoutResult<'wrt> {
        let len = line.display_len();
        let spaces = self.base.max_width().saturating_sub(len);
        self.base.write_spaces(self.alignment.indent(spaces), w)?;
        self.base.write_str(line, self.base.wrap_mode(), w)
    }
}

impl<'wrt> LayoutWriter<'wrt> for LinesWriter<'wrt> {
    fn options(&self) -> &'wrt LayoutOptions {
        self.base.options()
    }

    fn write_row(&mut self, w: &mut dyn Write) -> SizedLayoutResult {
        if self.base.row() == 0
            && let Some(style) = self.initial_style
        {
            self.base.write_style(style, w)?;
        }
        if let Some(line) = self.content.next() {
            let line = self.trimming.apply(line);
            let rest = self.write_line(line, w)?;
            if !rest.is_empty() {
                self.content.take_back(rest);
            }
        }
        self.base.end_row(w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Rect, WrapMode};

    #[test]
    fn lines_measure_pref() {
        // Wrap case
        let lines = Lines::left("abc def\nghi\njklm");
        assert_eq!(
            lines.measure(MeasureMode::pref(10, WrapMode::Wrap)).dim,
            Dimension::new(7, 3)
        );
        assert_eq!(
            lines.measure(MeasureMode::pref(5, WrapMode::Wrap)).dim,
            Dimension::new(5, 4)
        );
        assert_eq!(
            lines.measure(MeasureMode::pref(3, WrapMode::Wrap)).dim,
            Dimension::new(3, 6)
        );
        assert_eq!(
            lines.measure(MeasureMode::pref(1, WrapMode::Wrap)).dim,
            Dimension::new(1, 14)
        );
        assert_eq!(
            lines.measure(MeasureMode::pref(0, WrapMode::Wrap)).dim,
            Dimension::new(0, 0)
        );

        // Truncate case
        let lines = Lines::left("abc def\nghi\njklm");
        assert_eq!(
            lines
                .measure(MeasureMode::pref(10, WrapMode::Truncate("...")))
                .dim,
            Dimension::new(7, 3)
        );
        assert_eq!(
            lines
                .measure(MeasureMode::pref(5, WrapMode::Truncate("...")))
                .dim,
            Dimension::new(5, 3)
        );
        assert_eq!(
            lines
                .measure(MeasureMode::pref(3, WrapMode::Truncate("...")))
                .dim,
            Dimension::new(3, 3)
        );
        assert_eq!(
            lines
                .measure(MeasureMode::pref(1, WrapMode::Truncate("...")))
                .dim,
            Dimension::new(1, 3)
        );
        assert_eq!(
            lines
                .measure(MeasureMode::pref(0, WrapMode::Truncate("...")))
                .dim,
            Dimension::new(0, 0)
        );
    }

    #[test]
    fn lines_measure_exact() {
        let lines = Lines::left("abc def\nghi\njklm");
        let measurements = lines.measure(MeasureMode::exact(
            Dimension::new(10, 5),
            WrapMode::default(),
        ));
        assert_eq!(measurements.dim, Dimension::new(10, 5));
        assert_eq!(measurements.specifics.is_none(), true);
    }

    #[test]
    fn lines_measure_min() {
        let lines = Lines::left("abc def\nghi\njklm");
        let measurements = lines.measure(MeasureMode::min());
        assert_eq!(measurements.dim, Dimension::new(7, 3));
        assert_eq!(measurements.specifics.is_none(), true);
    }

    #[test]
    fn formatted_lines_display_truncate_no_clip_no_fill() {
        // Arrange
        let options = LayoutOptions::default()
            .with_fill_rows(false)
            .with_wrap_mode(WrapMode::Truncate("..."))
            .with_dim(Dimension::new(10, 5));

        // Case: left alignment
        let lines = Lines::left("xyz\nabcdefghij\nA long line\nklm");
        let result = format!("{}", lines.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "xyz\n",        //
                "abcdefghij\n", //
                "A long ...\n", //
                "klm\n",        //
                "\n",           //
            )
        );

        // Case: center alignment
        let lines = Lines::center("xyz\nabcdefghij\nA long line\nklm");
        let result = format!("{}", lines.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "   xyz\n",     //
                "abcdefghij\n", //
                "A long ...\n", //
                "   klm\n",     //
                "\n",           //
            )
        );

        // Case: right alignment
        let lines = Lines::right("xyz\nabcdefghij\nA long line\nklm");
        let result = format!("{}", lines.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "       xyz\n", //
                "abcdefghij\n", //
                "A long ...\n", //
                "       klm\n", //
                "\n",           //
            )
        );
    }

    #[test]
    fn formatted_lines_display_truncate_with_clip_with_fill() {
        // Arrange
        let options = LayoutOptions::default()
            .with_clip(Some(Rect::new(1, 1, Dimension::new(8, 4))))
            .with_fill_rows(true)
            .with_wrap_mode(WrapMode::Truncate("..."))
            .with_dim(Dimension::new(10, 6));

        // Case: left alignment
        let lines = Lines::left("xyz\nabcdefghij\nA long line\nklm");
        let result = format!("{}", lines.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "bcdefghi\n", //
                " long ..\n", //
                "lm      \n", //
                "        \n", //
            )
        );

        // Case: center alignment
        let lines = Lines::center("xyz\nabcdefghij\nA long line\nklm");
        let result = format!("{}", lines.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "bcdefghi\n", //
                " long ..\n", //
                "  klm   \n", //
                "        \n", //
            )
        );

        // Case: right alignment
        let lines = Lines::right("xyz\nabcdefghij\nA long line\nklm");
        let result = format!("{}", lines.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "bcdefghi\n", //
                " long ..\n", //
                "      kl\n", //
                "        \n", //
            )
        );
    }

    #[test]
    fn formatted_lines_display_wrap_no_clip_no_fill() {
        // Arrange
        let options = LayoutOptions::default()
            .with_fill_rows(false)
            .with_wrap_mode(WrapMode::Wrap)
            .with_dim(Dimension::new(10, 6));

        // Case: left alignment
        let lines = Lines::left("xyz\nabcdefghij\nA long line\nklm");
        let result = format!("{}", lines.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "xyz\n",        //
                "abcdefghij\n", //
                "A long lin\n", //
                "e\n",          //
                "klm\n",        //
                "\n",           //
            )
        );

        // Case: center alignment
        let lines = Lines::center("xyz\nabcdefghij\nA long line\nklm");
        let result = format!("{}", lines.layout_strict(options));
        assert_eq!(
            result,
            concat!(concat!(
                "   xyz\n",     //
                "abcdefghij\n", //
                "A long lin\n", //
                "    e\n",      //
                "   klm\n",     //
                "\n",           //
            ))
        );

        // Case: right alignment
        let lines = Lines::right("xyz\nabcdefghij\nA long line\nklm");
        let result = format!("{}", lines.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "       xyz\n", //
                "abcdefghij\n", //
                "A long lin\n", //
                "         e\n", //
                "       klm\n", //
                "\n",           //
            )
        );
    }

    #[test]
    fn formatted_lines_display_wrap_with_clip_with_fill() {
        // Arrange
        let options = LayoutOptions::default()
            .with_clip(Some(Rect::new(1, 1, Dimension::new(8, 4))))
            .with_fill_rows(true)
            .with_wrap_mode(WrapMode::Wrap)
            .with_dim(Dimension::new(10, 6));

        // Case: left alignment
        let lines = Lines::left("xyz\nabcdefghij\nA long line\nklm");
        let result = format!("{}", lines.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "bcdefghi\n", //
                " long li\n", //
                "        \n", //
                "lm      \n", //
            )
        );

        // Case: center alignment
        let lines = Lines::center("xyz\nabcdefghij\nA long line\nklm");
        let result = format!("{}", lines.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "bcdefghi\n", //
                " long li\n", //
                "   e    \n", //
                "  klm   \n", //
            )
        );

        // Case: right alignment
        let lines = Lines::right("xyz\nabcdefghij\nA long line\nklm");
        let result = format!("{}", lines.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "bcdefghi\n", //
                " long li\n", //
                "        \n", //
                "      kl\n", //
            )
        );
    }

    #[test]
    fn formatted_lines_display_fit_no_clip_no_fill() {
        // Arrange
        let options = LayoutOptions::default().with_dim(Dimension::new(10, 5));

        // Case: left alignment
        let lines = Lines::left("abc def \n  ghi\njkl");
        let result = format!("{}", lines.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "abc def\n", //
                "  ghi\n",   //
                "jkl\n",     //
                "\n",        //
                "\n",
            )
        );

        // Case: center alignment
        let lines = Lines::center("abc def \n  ghi\njkl");
        let result = format!("{}", lines.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                " abc def\n", //
                "    ghi\n",  //
                "   jkl\n",   //
                "\n",         //
                "\n",
            )
        );

        // Case: right alignment
        let lines = Lines::right("abc def \n  ghi\njkl");
        let result = format!("{}", lines.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "   abc def\n", //
                "       ghi\n", //
                "       jkl\n", //
                "\n",           //
                "\n",
            )
        );
    }

    #[test]
    fn formatted_lines_display_fit_no_clip_with_fill() {
        // Arrange
        let options = LayoutOptions::default()
            .with_fill_rows(true)
            .with_dim(Dimension::new(10, 5));

        // Case: left alignment
        let lines = Lines::left("abc def \n  ghi\njkl");
        let result = format!("{}", lines.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "abc def   \n", //
                "  ghi     \n", //
                "jkl       \n", //
                "          \n", //
                "          \n",
            )
        );

        // Case: center alignment
        let lines = Lines::center("abc def \n  ghi\njkl");
        let result = format!("{}", lines.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                " abc def  \n", //
                "    ghi   \n", //
                "   jkl    \n", //
                "          \n", //
                "          \n",
            )
        );

        // Case: right alignment
        let lines = Lines::right("abc def \n  ghi\njkl");
        let result = format!("{}", lines.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "   abc def\n", //
                "       ghi\n", //
                "       jkl\n", //
                "          \n", //
                "          \n",
            )
        );
    }

    #[test]
    fn formatted_lines_display_fit_with_clip_no_fill() {
        // Arrange
        let options = LayoutOptions::default()
            .with_clip(Some(Rect::new(1, 1, Dimension::new(8, 3))))
            .with_dim(Dimension::new(10, 5));

        // Case: left alignment
        let lines = Lines::left("abc def \n  ghi\njkl");
        let result = format!("{}", lines.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                " ghi\n", //
                "kl\n",   //
                "\n",
            )
        );

        // Case: center alignment
        let lines = Lines::center("abc def \n  ghi\njkl");
        let result = format!("{}", lines.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "   ghi\n", //
                "  jkl\n",  //
                "\n",
            )
        );

        // Case: right alignment
        let lines = Lines::right("abc def \n  ghi\njkl");
        let result = format!("{}", lines.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "      gh\n", //
                "      jk\n", //
                "\n",
            )
        );
    }

    #[test]
    fn formatted_lines_display_fit_with_clip_with_fill() {
        // Arrange
        let options = LayoutOptions::default()
            .with_clip(Some(Rect::new(1, 1, Dimension::new(8, 3))))
            .with_fill_rows(true)
            .with_dim(Dimension::new(10, 5));

        // Case: left alignment
        let lines = Lines::left("abc def \n  ghi\njkl");
        let result = format!("{}", lines.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                " ghi    \n", //
                "kl      \n", //
                "        \n",
            )
        );

        // Case: center alignment
        let lines = Lines::center("abc def \n  ghi\njkl");
        let result = format!("{}", lines.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "   ghi  \n", //
                "  jkl   \n", //
                "        \n",
            )
        );

        // Case: right alignment
        let lines = Lines::right("abc def \n  ghi\njkl");
        let result = format!("{}", lines.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "      gh\n", //
                "      jk\n", //
                "        \n",
            )
        );
    }

    #[test]
    fn formatted_lines_display_with_style() {
        use crate::ext::{Color, Effect, Style};

        let options = LayoutOptions::default().with_dim(Dimension::new(10, 3));
        let style = Style::default()
            .with_foreground(Color::Red)
            .with_effect(Effect::Bold);

        // Test left alignment with style
        let lines = Lines::left_with_style(style, "abc\ndef");
        let result = format!("{}", lines.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "\x1b[1;31mabc\x1b[0m\n", //
                "\x1b[1;31mdef\x1b[0m\n", //
                "\n"
            )
        );

        // Test center alignment with style
        let lines = Lines::center_with_style(style, "abc\ndef");
        let result = format!("{}", lines.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "\x1b[1;31m   abc\x1b[0m\n", //
                "\x1b[1;31m   def\x1b[0m\n", //
                "\n"
            )
        );

        // Test right alignment with style
        let lines = Lines::right_with_style(style, "abc\ndef");
        let result = format!("{}", lines.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "\x1b[1;31m       abc\x1b[0m\n", //
                "\x1b[1;31m       def\x1b[0m\n", //
                "\n"
            )
        );
    }
}
