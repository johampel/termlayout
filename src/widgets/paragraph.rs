use crate::ext::{
    BaseLayoutWriter, BoxedLayoutWriter, DisplayStr, DisplayWords, FormattedLayout, LayoutWriter,
    SizedLayoutResult, StrLayoutResult, TakeBackIterator,
};
use crate::ext::{BoxedFormattedLayout, Style, box_formatted_layout, rc_layout};
use crate::{Dimension, Layout, LayoutContext, LayoutOptions, MeasureMode, Measurements, WrapMode};
use std::any::Any;
use std::fmt::Write;

/// A widget that displays text with word wrapping and alignment.
///
/// The content is represented as a string (which may contain ANSI control sequences for
/// terminal styling). The text is split into words and each line is filled with as many
/// words as possible. The widget supports left, center, right, and block alignment.
///
/// # Example
/// ```rust
/// use termlayout::*;
/// use termlayout::widgets::Paragraph;
///
/// let lines = r#"These are some sample words.
/// The Paragraph-Layout can display them word-wise."#;
///
/// let layout = Paragraph::left(lines);
/// assert_eq!(format!("{}", layout.layout_with_wrap_mode(20, WrapMode::default_truncate())),
/// concat!(
///     "These are some\n",
///     "sample words. The\n",
///     "Paragraph-Layout can\n",
///     "display them\nword-wise.\n",
/// ));
/// assert_eq!(format!("{}", layout.layout_with_wrap_mode(15, WrapMode::default_truncate())),
/// concat!(
///     "These are some\n",
///     "sample words.\n",
///     "The\n",
///     "Paragraph-Layo…\n",
///     "can display\n",
///     "them word-wise.\n",
/// ));
/// ```
#[derive(Debug)]
pub struct Paragraph {
    /// The alignment of the lines.
    pub alignment: ParagraphAlignment,

    /// The content to be displayed in the layout.
    pub content: String,

    /// The initial style of the paragraph.
    pub initial_style: Option<Style>,
}

impl Paragraph {
    /// Creates a new [`Paragraph`] layout with the given alignment.
    ///
    /// # Parameters
    /// - `alignment`: The horizontal alignment of the content.
    /// - `content`: The content to be displayed in the layout.
    ///
    /// # Returns
    /// The new [`Paragraph`] layout.
    pub fn new<T: Into<String>>(
        alignment: ParagraphAlignment,
        initial_style: Option<Style>,
        content: T,
    ) -> Self {
        Self {
            alignment,
            initial_style,
            content: content.into(),
        }
    }

    /// Creates a new [`Paragraph`] layout with left alignment.
    ///
    /// # Parameters
    /// - `content`: The content to be displayed in the layout.
    ///
    /// # Returns
    /// The new [`Paragraph`] layout.
    pub fn left<T: Into<String>>(content: T) -> Self {
        Self::new(ParagraphAlignment::Left, None, content)
    }

    /// Creates a new [`Paragraph`] layout with left alignment and the given initial style.
    ///
    /// # Parameters
    /// - `initial_style`: The initial style of the paragraph.
    /// - `content`: The content to be displayed in the layout.
    ///
    /// # Returns
    /// The new [`Paragraph`] layout.
    pub fn left_with_style<T: Into<String>>(initial_style: Style, content: T) -> Self {
        Self::new(ParagraphAlignment::Left, Some(initial_style), content)
    }

    /// Creates a new [`Paragraph`] layout with center alignment.
    ///
    /// # Parameters
    /// - `content`: The content to be displayed in the layout.
    ///
    /// # Returns
    /// The new [`Paragraph`] layout.
    pub fn center<T: Into<String>>(content: T) -> Self {
        Self::new(ParagraphAlignment::Center, None, content)
    }

    /// Creates a new [`Paragraph`] layout with center alignment and the given initial style.
    ///
    /// # Parameters
    /// - `initial_style`: The initial style of the paragraph.
    /// - `content`: The content to be displayed in the layout.
    ///
    /// # Returns
    /// The new [`Paragraph`] layout.
    pub fn center_with_style<T: Into<String>>(initial_style: Style, content: T) -> Self {
        Self::new(ParagraphAlignment::Center, Some(initial_style), content)
    }

    /// Creates a new [`Paragraph`] layout with right alignment.
    ///
    /// # Parameters
    /// - `content`: The content to be displayed in the layout.
    ///
    /// # Returns
    /// The new [`Paragraph`] layout.
    pub fn right<T: Into<String>>(content: T) -> Self {
        Self::new(ParagraphAlignment::Right, None, content)
    }

    /// Creates a new [`Paragraph`] layout with right alignment and the given initial style.
    ///
    /// # Parameters
    /// - `initial_style`: The initial style of the paragraph.
    /// - `content`: The content to be displayed in the layout.
    ///
    /// # Returns
    /// The new [`Paragraph`] layout.
    pub fn right_with_style<T: Into<String>>(initial_style: Style, content: T) -> Self {
        Self::new(ParagraphAlignment::Right, Some(initial_style), content)
    }

    /// Creates a new [`Paragraph`] layout with block alignment.
    ///
    /// # Parameters
    /// - `content`: The content to be displayed in the layout.
    ///
    /// # Returns
    /// The new [`Paragraph`] layout.
    pub fn block<T: Into<String>>(content: T) -> Self {
        Self::new(ParagraphAlignment::Block, None, content)
    }

    /// Creates a new [`Paragraph`] layout with block alignment and the given initial style.
    ///
    /// # Parameters
    /// - `initial_style`: The initial style of the paragraph.
    /// - `content`: The content to be displayed in the layout.
    ///
    /// # Returns
    /// The new [`Paragraph`] layout.
    pub fn block_with_style<T: Into<String>>(initial_style: Style, content: T) -> Self {
        Self::new(ParagraphAlignment::Block, Some(initial_style), content)
    }

    fn longest_word(&self) -> usize {
        self.content
            .display_words()
            .map(DisplayStr::display_len)
            .max()
            .unwrap_or(0)
    }

    fn calculate_dim(&self, max_width: usize, fixed_width: bool, wrap_mode: WrapMode) -> Dimension {
        if max_width == 0 {
            return Dimension::empty();
        }
        let mut cols = 0;
        let mut rows = 0;
        let mut line_len = 0;
        let mut words = TakeBackIterator::new(self.content.display_words());
        if fixed_width {
            cols = max_width
        }
        while let Some(word) = words.next() {
            let word_len = word.display_len();
            if line_len == 0 && word_len > 0 {
                // First word in line
                rows += 1;
                if word_len > max_width {
                    // Word does not fit -> truncate or wrap
                    cols = max_width;
                    if matches!(wrap_mode, WrapMode::Wrap) {
                        let rest = word.display_slice(max_width..);
                        if !rest.is_empty() {
                            words.take_back(rest);
                        }
                    }
                } else {
                    line_len = word_len;
                }
            } else {
                // Not first word in line
                if line_len + word_len + 1 > max_width {
                    // Word does not fit -> update cols and begin new line
                    cols = cols.max(line_len);
                    line_len = 0;
                    words.take_back(word);
                } else {
                    // Word fits on current line
                    line_len += word_len;
                    if word_len > 0 {
                        line_len += 1;
                    }
                }
            }
        }

        Dimension::new(cols.max(line_len), rows)
    }
}

impl Layout for Paragraph {
    fn measure(&self, mode: MeasureMode) -> Measurements {
        match mode {
            MeasureMode::Min => {
                self.measure(MeasureMode::pref(self.longest_word(), WrapMode::default()))
            }
            MeasureMode::Pref {
                max_width,
                wrap_mode,
            } => self.calculate_dim(max_width, false, wrap_mode).into(),
            MeasureMode::FixedWidth { width, wrap_mode } => {
                self.calculate_dim(width, true, wrap_mode).into()
            }
            MeasureMode::Exact { dimension, .. } => dimension.into(),
        }
    }

    fn layout_with_context(&'_ self, context: LayoutContext) -> BoxedFormattedLayout<'_> {
        FormattedParagraph::new(self, self.initial_style, context.into()).into()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

rc_layout!(Paragraph);

/// Represents horizontal alignment options for paragraphs
///
/// Represents horizontal alignment options for a [`Paragraph`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ParagraphAlignment {
    /// Aligns the content to the left.
    Left,

    /// Centers the content horizontally.
    Center,

    /// Aligns the content to the right.
    Right,

    /// Aligns the content to both the left and right by adjusting spacing between words.
    Block,
}

impl ParagraphAlignment {
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
    /// use termlayout::widgets::ParagraphAlignment;
    ///
    /// assert_eq!(ParagraphAlignment::Left.indent(5), 0);
    /// assert_eq!(ParagraphAlignment::Center.indent(5), 2);
    /// assert_eq!(ParagraphAlignment::Right.indent(5), 5);
    /// ```
    #[must_use]
    pub fn indent(&self, space: usize) -> usize {
        match self {
            ParagraphAlignment::Center => space / 2,
            ParagraphAlignment::Right => space,
            _ => 0,
        }
    }
}

struct FormattedParagraph<'fmt> {
    content: &'fmt Paragraph,
    options: LayoutOptions,
    initial_style: Option<Style>,
}

impl<'fmt> FormattedParagraph<'fmt> {
    fn new(content: &'fmt Paragraph, initial_style: Option<Style>, options: LayoutOptions) -> Self {
        Self {
            content,
            options,
            initial_style,
        }
    }
}

impl FormattedLayout for FormattedParagraph<'_> {
    fn options(&self) -> &LayoutOptions {
        &self.options
    }

    fn new_writer(&'_ self) -> BoxedLayoutWriter<'_> {
        Box::new(ParagraphWriter::new(
            self.content.alignment,
            self.initial_style,
            &self.content.content,
            &self.options,
        ))
    }
}

box_formatted_layout!(FormattedParagraph);

struct ParagraphWriter<'wrt> {
    base: BaseLayoutWriter<'wrt>,
    alignment: ParagraphAlignment,
    initial_style: Option<Style>,
    content: TakeBackIterator<DisplayWords<'wrt>>,
}

impl<'wrt> ParagraphWriter<'wrt> {
    fn new(
        alignment: ParagraphAlignment,
        initial_style: Option<Style>,
        content: &'wrt str,
        options: &'wrt LayoutOptions,
    ) -> Self {
        Self {
            base: BaseLayoutWriter::new(options),
            initial_style,
            content: TakeBackIterator::new(content.display_words()),
            alignment,
        }
    }

    fn collect_line(&mut self) -> Option<(usize, Vec<&'wrt str>)> {
        let mut words = Vec::new();
        let mut len = 0;
        let max_len = self.base.available_width();
        while len < max_len {
            if let Some(word) = self.content.next() {
                let word_len = word.display_len();
                let required_len = if !words.is_empty() && word_len > 0 {
                    word_len + 1
                } else {
                    word_len
                };
                if words.is_empty() {
                    len += word_len;
                    words.push(word);
                } else if len + required_len <= max_len {
                    len += required_len;
                    words.push(word);
                } else {
                    if !word.is_empty() {
                        self.content.take_back(word);
                    }
                    break;
                }
            } else {
                break;
            }
        }
        if words.is_empty() {
            None
        } else {
            Some((len, words))
        }
    }

    fn write_line(
        &mut self,
        words: &[&'wrt str],
        len: usize,
        w: &mut dyn Write,
    ) -> StrLayoutResult<'wrt> {
        let mut spaces = self.base.max_width().saturating_sub(len);
        let mut count = words.len();
        let mut rest = "";
        let mut prev_word_len = 0;
        self.base.write_spaces(self.alignment.indent(spaces), w)?;

        for (index, word) in words.iter().enumerate() {
            let word_len = word.display_len();
            if index > 0 && prev_word_len > 0 && word_len > 0 {
                count -= 1;
                let spaces = if self.alignment == ParagraphAlignment::Block {
                    let s = spaces / count;
                    spaces -= s;
                    s + 1
                } else {
                    1
                };
                self.base.write_spaces(spaces, w)?;
            }
            rest = self.base.write_str(word, self.base.wrap_mode(), w)?;
            prev_word_len = word_len;
        }

        Ok(rest)
    }
}

impl<'wrt> LayoutWriter<'wrt> for ParagraphWriter<'wrt> {
    fn options(&self) -> &'wrt LayoutOptions {
        self.base.options()
    }

    fn write_row(&mut self, w: &mut dyn Write) -> SizedLayoutResult {
        if self.base.row() == 0
            && let Some(style) = &self.initial_style
        {
            self.base.write_style(*style, w)?;
        }
        if let Some((len, words)) = self.collect_line() {
            let rest = self.write_line(&words, len, w)?;
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
    use crate::Rect;

    #[test]
    fn paragraph_measure_exact() {
        let content = "abcdefgh abcd fgh ab de gh";
        let paragraph = Paragraph::left(content);
        assert_eq!(
            paragraph.measure(MeasureMode::exact(Dimension::new(12, 14))).dim,
            Dimension::new(12, 14)
        );
    }

    #[test]
    fn paragraph_measure_fixed_width() {
        let content = "abcdefgh abcd fgh ab de gh";
        let paragraph = Paragraph::left(content);
        assert_eq!(
            paragraph.measure(MeasureMode::fixed_width(14, WrapMode::Wrap)).dim,
            Dimension::new(14, 2)
        );
        assert_eq!(
            paragraph.measure(MeasureMode::fixed_width(14, WrapMode::default_truncate())).dim,
            Dimension::new(14, 2)
        );
        assert_eq!(
            paragraph.measure(MeasureMode::fixed_width(9, WrapMode::Wrap)).dim,
            Dimension::new(9, 3)
        );
        assert_eq!(
            paragraph.measure(MeasureMode::fixed_width(9, WrapMode::default_truncate())).dim,
            Dimension::new(9, 3)
        );
    }

    #[test]
    fn paragraph_measure_pref_fit() {
        let content = "abcdefgh abcd fgh ab de gh";
        let paragraph = Paragraph::left(content);
        assert_eq!(
            paragraph.measure(MeasureMode::pref(14, WrapMode::Wrap)).dim,
            Dimension::new(13, 2)
        );
        assert_eq!(
            paragraph.measure(MeasureMode::pref(14, WrapMode::default_truncate())).dim,
            Dimension::new(13, 2)
        );
        assert_eq!(
            paragraph.measure(MeasureMode::pref(8, WrapMode::Wrap)).dim,
            Dimension::new(8, 3)
        );
        assert_eq!(
            paragraph.measure(MeasureMode::pref(8, WrapMode::default_truncate())).dim,
            Dimension::new(8, 3)
        );
    }

    #[test]
    fn paragraph_measure_pref_wrap() {
        let content = "abcdefgh abcd fgh ab de gh";
        let paragraph = Paragraph::left(content);

        assert_eq!(
            paragraph.measure(MeasureMode::pref(7, WrapMode::Wrap)).dim,
            Dimension::new(7, 4)
        );
        assert_eq!(
            paragraph.measure(MeasureMode::pref(5, WrapMode::Wrap)).dim,
            Dimension::new(5, 6)
        );
    }

    #[test]
    fn paragraph_measure_pref_truncate() {
        let content = "abcdefgh abcd fgh ab de gh";
        let paragraph = Paragraph::left(content);

        assert_eq!(
            paragraph.measure(MeasureMode::pref(7, WrapMode::default_truncate())).dim,
            Dimension::new(7, 4)
        );
        assert_eq!(
            paragraph.measure(MeasureMode::pref(5, WrapMode::default_truncate())).dim,
            Dimension::new(5, 5)
        );
    }

    #[test]
    fn paragraph_measure_min() {
        let content = "abcdefgh abcd fgh ab de gh";
        let paragraph = Paragraph::left(content);

        assert_eq!(paragraph.measure(MeasureMode::Min).dim, Dimension::new(8, 3));
    }

    #[test]
    fn formatted_paragraph_display_truncate_no_clip_no_fill() {
        // Arrange
        let content = "abcdefgh abcd fgh ab de gh 1 2 3 4 5 7 8";
        let options = LayoutOptions::default()
            .with_wrap_mode(WrapMode::Truncate("..."))
            .with_dim(Dimension::new(6, 8));

        // Left alignment
        let paragraph = Paragraph::left(content);
        let result = format!("{}", paragraph.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "abc...\n", //
                "abcd\n",   //
                "fgh ab\n", //
                "de gh\n",  //
                "1 2 3\n",  //
                "4 5 7\n",  //
                "8\n",      //
                "\n",       //
            )
        );

        // Center alignment
        let paragraph = Paragraph::center(content);
        let result = format!("{}", paragraph.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "abc...\n", //
                " abcd\n",  //
                "fgh ab\n", //
                "de gh\n",  //
                "1 2 3\n",  //
                "4 5 7\n",  //
                "  8\n",    //
                "\n",       //
            )
        );

        // Center alignment
        let paragraph = Paragraph::right(content);
        let result = format!("{}", paragraph.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "abc...\n", //
                "  abcd\n", //
                "fgh ab\n", //
                " de gh\n", //
                " 1 2 3\n", //
                " 4 5 7\n", //
                "     8\n", //
                "\n",       //
            )
        );

        // Block alignment
        let paragraph = Paragraph::block(content);
        let result = format!("{}", paragraph.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "abc...\n", //
                "abcd\n",   //
                "fgh ab\n", //
                "de  gh\n", //
                "1 2  3\n", //
                "4 5  7\n", //
                "8\n",      //
                "\n",       //
            )
        );
    }

    #[test]
    fn formatted_paragraph_display_truncate_with_clip_with_fill() {
        // Arrange
        let content = "abcdefgh abcd fgh ab de gh 1 2 3 4 5 7 8";
        let options = LayoutOptions::default()
            .with_wrap_mode(WrapMode::Truncate("..."))
            .with_fill_rows(true)
            .with_clip(Some(Rect::new(1, 2, Dimension::new(3, 4))))
            .with_dim(Dimension::new(6, 8));

        // Left alignment
        let paragraph = Paragraph::left(content);
        let result = format!("{}", paragraph.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "gh \n", //
                "e g\n", //
                " 2 \n", //
                " 5 \n", //
            )
        );

        // Center alignment
        let paragraph = Paragraph::center(content);
        let result = format!("{}", paragraph.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "gh \n", //
                "e g\n", //
                " 2 \n", //
                " 5 \n", //
            )
        );

        // Center alignment
        let paragraph = Paragraph::right(content);
        let result = format!("{}", paragraph.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "gh \n", //
                "de \n", //
                "1 2\n", //
                "4 5\n", //
            )
        );

        // Block alignment
        let paragraph = Paragraph::block(content);
        let result = format!("{}", paragraph.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "gh \n", //
                "e  \n", //
                " 2 \n", //
                " 5 \n", //
            )
        );
    }

    #[test]
    fn formatted_paragraph_display_wrap_no_clip_no_fill() {
        // Arrange
        let content = "abcdefgh abcd fgh ab de gh 1 2 3 4 5 7 8";
        let options = LayoutOptions::default().with_dim(Dimension::new(6, 9));

        // Left alignment
        let paragraph = Paragraph::left(content);
        let result = format!("{}", paragraph.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "abcdef\n", //
                "gh\n",     //
                "abcd\n",   //
                "fgh ab\n", //
                "de gh\n",  //
                "1 2 3\n",  //
                "4 5 7\n",  //
                "8\n",      //
                "\n",       //
            )
        );

        // Center alignment
        let paragraph = Paragraph::center(content);
        let result = format!("{}", paragraph.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "abcdef\n", //
                "  gh\n",   //
                " abcd\n",  //
                "fgh ab\n", //
                "de gh\n",  //
                "1 2 3\n",  //
                "4 5 7\n",  //
                "  8\n",    //
                "\n",       //
            )
        );

        // Center alignment
        let paragraph = Paragraph::right(content);
        let result = format!("{}", paragraph.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "abcdef\n", //
                "    gh\n", //
                "  abcd\n", //
                "fgh ab\n", //
                " de gh\n", //
                " 1 2 3\n", //
                " 4 5 7\n", //
                "     8\n", //
                "\n",       //
            )
        );

        // Block alignment
        let paragraph = Paragraph::block(content);
        let result = format!("{}", paragraph.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "abcdef\n", //
                "gh\n",     //
                "abcd\n",   //
                "fgh ab\n", //
                "de  gh\n", //
                "1 2  3\n", //
                "4 5  7\n", //
                "8\n",      //
                "\n",       //
            )
        );
    }

    #[test]
    fn formatted_paragraph_display_wrap_with_clip_with_fill() {
        // Arrange
        let content = "abcdefgh abcd fgh ab de gh 1 2 3 4 5 7 8";
        let options = LayoutOptions::default()
            .with_fill_rows(true)
            .with_dim(Dimension::new(6, 9))
            .with_clip(Some(Rect::new(1, 2, Dimension::new(3, 4))));

        // Left alignment
        let paragraph = Paragraph::left(content);
        let result = format!("{}", paragraph.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "bcd\n", //
                "gh \n", //
                "e g\n", //
                " 2 \n", //
            )
        );

        // Center alignment
        let paragraph = Paragraph::center(content);
        let result = format!("{}", paragraph.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "abc\n", //
                "gh \n", //
                "e g\n", //
                " 2 \n", //
            )
        );

        // Center alignment
        let paragraph = Paragraph::right(content);
        let result = format!("{}", paragraph.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                " ab\n", //
                "gh \n", //
                "de \n", //
                "1 2\n", //
            )
        );

        // Block alignment
        let paragraph = Paragraph::block(content);
        let result = format!("{}", paragraph.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "bcd\n", //
                "gh \n", //
                "e  \n", //
                " 2 \n", //
            )
        );
    }

    #[test]
    fn formatted_paragraph_display_fit_no_clip_no_fill() {
        // Arrange
        let content = "abcdefgh abcd fgh ab de gh 1 2 3 4 5 7 8";
        let options = LayoutOptions::default().with_dim(Dimension::new(8, 6));

        // Left alignment
        let paragraph = Paragraph::left(content);
        let result = format!("{}", paragraph.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "abcdefgh\n", //
                "abcd fgh\n", //
                "ab de gh\n", //
                "1 2 3 4\n",  //
                "5 7 8\n",    //
                "\n",         //
            )
        );

        // Center alignment
        let paragraph = Paragraph::center(content);
        let result = format!("{}", paragraph.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "abcdefgh\n", //
                "abcd fgh\n", //
                "ab de gh\n", //
                "1 2 3 4\n",  //
                " 5 7 8\n",   //
                "\n",         //
            )
        );

        // Center alignment
        let paragraph = Paragraph::right(content);
        let result = format!("{}", paragraph.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "abcdefgh\n", //
                "abcd fgh\n", //
                "ab de gh\n", //
                " 1 2 3 4\n", //
                "   5 7 8\n", //
                "\n",         //
            )
        );

        // Block alignment
        let paragraph = Paragraph::block(content);
        let result = format!("{}", paragraph.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "abcdefgh\n", //
                "abcd fgh\n", //
                "ab de gh\n", //
                "1 2 3  4\n", //
                "5  7   8\n", //
                "\n",         //
            )
        );
    }

    #[test]
    fn formatted_paragraph_display_fit_no_clip_with_fill() {
        // Arrange
        let content = "abcdefgh abcd fgh ab de gh 1 2 3 4 5 7 8";
        let options = LayoutOptions::default()
            .with_fill_rows(true)
            .with_dim(Dimension::new(8, 6));

        // Left alignment
        let paragraph = Paragraph::left(content);
        let result = format!("{}", paragraph.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "abcdefgh\n", //
                "abcd fgh\n", //
                "ab de gh\n", //
                "1 2 3 4 \n", //
                "5 7 8   \n", //
                "        \n", //
            )
        );

        // Center alignment
        let paragraph = Paragraph::center(content);
        let result = format!("{}", paragraph.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "abcdefgh\n", //
                "abcd fgh\n", //
                "ab de gh\n", //
                "1 2 3 4 \n", //
                " 5 7 8  \n", //
                "        \n", //
            )
        );

        // Center alignment
        let paragraph = Paragraph::right(content);
        let result = format!("{}", paragraph.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "abcdefgh\n", //
                "abcd fgh\n", //
                "ab de gh\n", //
                " 1 2 3 4\n", //
                "   5 7 8\n", //
                "        \n", //
            )
        );

        // Block alignment
        let paragraph = Paragraph::block(content);
        let result = format!("{}", paragraph.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "abcdefgh\n", //
                "abcd fgh\n", //
                "ab de gh\n", //
                "1 2 3  4\n", //
                "5  7   8\n", //
                "        \n", //
            )
        );
    }

    #[test]
    fn formatted_paragraph_display_fit_with_clip_no_fill() {
        // Arrange
        let content = "abcdefgh abcd fgh ab de gh 1 2 3 4 5 7 8";
        let options = LayoutOptions::default()
            .with_dim(Dimension::new(8, 6))
            .with_clip(Some(Rect::new(2, 1, Dimension::new(5, 4))));

        // Left alignment
        let paragraph = Paragraph::left(content);
        let result = format!("{}", paragraph.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "cd fg\n", //
                " de g\n", //
                "2 3 4\n", //
                "7 8\n",   //
            )
        );

        // Center alignment
        let paragraph = Paragraph::center(content);
        let result = format!("{}", paragraph.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "cd fg\n", //
                " de g\n", //
                "2 3 4\n", //
                " 7 8\n",  //
            )
        );

        // Center alignment
        let paragraph = Paragraph::right(content);
        let result = format!("{}", paragraph.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "cd fg\n", //
                " de g\n", //
                " 2 3 \n", //
                " 5 7 \n", //
            )
        );

        // Block alignment
        let paragraph = Paragraph::block(content);
        let result = format!("{}", paragraph.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "cd fg\n", //
                " de g\n", //
                "2 3  \n", //
                " 7   \n", //
            )
        );
    }

    #[test]
    fn formatted_paragraph_display_fit_with_clip_with_fill() {
        // Arrange
        let content = "abcdefgh abcd fgh ab de gh 1 2 3 4 5 7 8";
        let options = LayoutOptions::default()
            .with_dim(Dimension::new(8, 6))
            .with_fill_rows(true)
            .with_clip(Some(Rect::new(2, 1, Dimension::new(5, 4))));

        // Left alignment
        let paragraph = Paragraph::left(content);
        let result = format!("{}", paragraph.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "cd fg\n", //
                " de g\n", //
                "2 3 4\n", //
                "7 8  \n", //
            )
        );

        // Center alignment
        let paragraph = Paragraph::center(content);
        let result = format!("{}", paragraph.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "cd fg\n", //
                " de g\n", //
                "2 3 4\n", //
                " 7 8 \n", //
            )
        );

        // Center alignment
        let paragraph = Paragraph::right(content);
        let result = format!("{}", paragraph.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "cd fg\n", //
                " de g\n", //
                " 2 3 \n", //
                " 5 7 \n", //
            )
        );

        // Block alignment
        let paragraph = Paragraph::block(content);
        let result = format!("{}", paragraph.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "cd fg\n", //
                " de g\n", //
                "2 3  \n", //
                " 7   \n", //
            )
        );
    }

    #[test]
    fn formatted_paragraph_display_with_style() {
        use crate::ext::{Color, Effect, Style};

        let options = LayoutOptions::default().with_dim(Dimension::new(10, 3));
        let style = Style::default()
            .with_foreground(Color::Red)
            .with_effect(Effect::Bold);

        // Test left alignment with style
        let paragraph = Paragraph::left_with_style(style, "abc def ghi");
        let result = format!("{}", paragraph.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "\x1b[1;31mabc def\x1b[0m\n", //
                "\x1b[1;31mghi\x1b[0m\n",     //
                "\n"
            )
        );

        // Test center alignment with style
        let paragraph = Paragraph::center_with_style(style, "abc def ghi");
        let result = format!("{}", paragraph.layout_strict(options));
        assert_eq!(
            result,
            concat!(concat!(
                "\x1b[1;31m abc def\x1b[0m\n", //
                "\x1b[1;31m   ghi\x1b[0m\n",   //
                "\n"
            ))
        );

        // Test right alignment with style
        let paragraph = Paragraph::right_with_style(style, "abc def ghi");
        let result = format!("{}", paragraph.layout_strict(options));
        assert_eq!(
            result,
            concat!(concat!(
                "\x1b[1;31m   abc def\x1b[0m\n", //
                "\x1b[1;31m       ghi\x1b[0m\n", //
                "\n"
            ))
        );

        // Test block alignment with style
        let paragraph = Paragraph::block_with_style(style, "abc def ghi");
        let result = format!("{}", paragraph.layout_strict(options));
        assert_eq!(
            result,
            concat!(concat!(
                "\x1b[1;31mabc    def\x1b[0m\n", //
                "\x1b[1;31mghi\x1b[0m\n",        //
                "\n"
            ))
        );
    }
}
