use std::fmt::{Display, Formatter, Write};
use std::ops::{Bound, RangeBounds};

/// A trait providing methods for string operations that respect terminal control sequences.
/// This trait is implemented for `str` and provides methods for calculating display length,
/// slicing, and splitting text while ignoring ANSI escape sequences.
pub trait DisplayStr: AsRef<str> {
    /// Returns an [`Iterator`] for [`Fragment`]s.
    /// `self` is seen as a sequence of plain text and control sequences. This iterator returns
    /// them as a sequence of [`Fragment`]s.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::ext::*;
    ///
    /// let text = "Hello, \x1b[1mwörld!\x1b[0m";
    ///
    /// let mut iter = text.display_fragments();
    ///
    /// assert_eq!(iter.next(), Some(Fragment::Plain("Hello, ")));
    /// assert_eq!(iter.next(), Some(Fragment::ControlSequence("1")));
    /// assert_eq!(iter.next(), Some(Fragment::Plain("wörld!")));
    /// assert_eq!(iter.next(), Some(Fragment::ControlSequence("0")));
    /// assert_eq!(iter.next(), None);
    /// ```
    fn display_fragments(&self) -> impl Iterator<Item = Fragment<'_>> {
        FragmentIter::new(self.as_ref()).map(|(_, f)| f)
    }

    /// Returns an iterator that iterates through the text line wise.
    ///
    /// # Returns
    /// An according iterator
    ///
    /// # Example
    /// ```rust
    /// use termlayout::ext::*;
    ///
    /// let content = "abc\n   def   \nghi\n\njkl mno\n";
    /// let mut lines = content.display_lines();
    ///
    /// assert_eq!(lines.next(), Some("abc"));
    /// assert_eq!(lines.next(), Some("   def   "));
    /// assert_eq!(lines.next(), Some("ghi"));
    /// assert_eq!(lines.next(), Some(""));
    /// assert_eq!(lines.next(), Some("jkl mno"));
    /// assert_eq!(lines.next(), None);
    /// ```
    fn display_lines(&'_ self) -> DisplayLines<'_> {
        DisplayLines::new(self.as_ref())
    }

    /// Returns an iterator that iterates through the text word wise.
    ///
    /// # Returns
    /// An according iterator
    ///
    /// # Example
    /// ```rust
    /// use termlayout::ext::*;
    ///
    /// let content = "  abc\ndef   \nghi   jkl  mno\n";
    /// let mut lines = content.display_words();
    ///
    /// assert_eq!(lines.next(), Some("abc"));
    /// assert_eq!(lines.next(), Some("def"));
    /// assert_eq!(lines.next(), Some("ghi"));
    /// assert_eq!(lines.next(), Some("jkl"));
    /// assert_eq!(lines.next(), Some("mno"));
    /// assert_eq!(lines.next(), None);
    /// ```
    fn display_words(&'_ self) -> DisplayWords<'_> {
        DisplayWords::new(self.as_ref())
    }

    /// Returns an [`Iterator`] for [`Fragment`]s and their byte index.
    /// `self` is seen as a sequence of plain text and control sequences. This iterator returns
    /// them as [`Fragment`]s.
    ///
    /// # Returns
    /// An according iterator
    ///
    /// # Example
    /// ```rust
    /// use termlayout::ext::*;
    ///
    /// let text = "Hello, \x1b[1mwörld!\x1b[0m";
    ///
    /// let mut iter = text.display_fragments_with_index();
    ///
    /// assert_eq!(iter.next(), Some((0, Fragment::Plain("Hello, "))));
    /// assert_eq!(iter.next(), Some((7, Fragment::ControlSequence("1"))));
    /// assert_eq!(iter.next(), Some((11, Fragment::Plain("wörld!"))));
    /// assert_eq!(iter.next(), Some((18, Fragment::ControlSequence("0"))));
    /// assert_eq!(iter.next(), None);
    /// ```
    fn display_fragments_with_index(&self) -> impl Iterator<Item = (usize, Fragment<'_>)> {
        FragmentIter::new(self.as_ref())
    }

    /// Calculates the display length of the text - measured in terminal columns - excluding control
    /// sequences.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::ext::*;
    ///
    /// let text = "Hello, \x1b[1mwörld!\x1b[0m";
    /// assert_eq!(text.display_len(), 13);
    ///
    /// let emoji = "🦀";
    /// assert_eq!(emoji.display_len(), 2);
    /// ```
    fn display_len(&self) -> usize {
        use unicode_width::UnicodeWidthStr;
        self.display_fragments()
            .map(|f| match f {
                Fragment::Plain(x) => UnicodeWidthStr::width(x),
                Fragment::ControlSequence(_) => 0,
            })
            .sum()
    }

    /// Splits the text at the given display position (measured in terminal columns).
    /// When calculating the exact position to split, it counts only the plain text
    /// terminal columns. If a split would occur in the middle of a multi-column character,
    /// the character is included in the second part.
    ///
    /// # Parameters
    /// - `pos`: The display position to split the text at.
    ///
    /// # Returns
    /// A tuple containing the two parts of the text split at the given position.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::ext::*;
    ///
    /// let text = "Hello, \x1b[1mwörld!\x1b[0m";
    ///
    /// assert_eq!(text.display_split_at(8), ("Hello, \x1b[1mw", "örld!\x1b[0m"));
    /// ```
    fn display_split_at(&self, pos: usize) -> (&str, &str) {
        self.as_ref()
            .split_at(self.display_to_byte_index(pos, false).unwrap())
    }

    /// Returns a slice according to its display index range (measured in terminal columns).
    /// A display index counts terminal columns that are not part of control sequences.
    ///
    /// # Parameters
    /// - `range`: The range of display indices to extract the slice for.
    ///
    /// # Returns
    /// The slice.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::ext::*;
    ///
    /// let text = "Hello, \x1b[1mwörld!\x1b[0m";
    ///
    /// assert_eq!(text.display_slice(..), "Hello, \x1b[1mwörld!\x1b[0m");
    /// assert_eq!(text.display_slice(..7), "Hello, \x1b[1m");
    /// assert_eq!(text.display_slice(7..), "\x1b[1mwörld!\x1b[0m");
    /// assert_eq!(text.display_slice(7..10), "\x1b[1mwör");
    /// ```
    fn display_slice<R>(&self, range: R) -> &str
    where
        R: RangeBounds<usize>,
    {
        let len = self.display_len();
        let start_index = match range.start_bound() {
            Bound::Included(&x) => x,
            Bound::Excluded(&x) => x + 1,
            Bound::Unbounded => 0,
        };
        let start = self.display_to_byte_index(start_index, true).unwrap();
        let end_index = match range.end_bound() {
            Bound::Included(&x) => x + 1,
            Bound::Excluded(&x) => x,
            Bound::Unbounded => self.display_len(),
        };
        let end = if end_index < len {
            self.display_to_byte_index(end_index, false).unwrap()
        } else {
            self.as_ref().len()
        };
        &self.as_ref()[start..end]
    }

    /// Converts a display index (terminal columns) to a byte index.
    /// A display index counts terminal columns that are not part of control sequences.
    ///
    /// # Parameters
    /// - `display_index`: The display index to convert.
    /// - `as_start`: Whether to convert the display index as a start index. This influences how
    ///   to deal with control sequences if the display index points to the beginning of a
    ///   control sequence. If `true`, the returned byte index points to the start of the control
    ///   sequence. If `false`, the returned byte index points to the end of the control sequence.
    ///
    /// # Returns
    /// The byte index corresponding to the display index.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::ext::*;
    ///
    /// // byte:    01234567   8901245678   901
    /// // display: 0123456       78901       2
    /// let text = "Hello, \x1b[1mwörld!\x1b[0m";
    ///
    /// assert_eq!(text.display_to_byte_index(0, false), Some(0));
    /// assert_eq!(text.display_to_byte_index(0, true), Some(0));
    /// assert_eq!(text.display_to_byte_index(7, false), Some(11));
    /// assert_eq!(text.display_to_byte_index(7, true), Some(7));
    /// assert_eq!(text.display_to_byte_index(8, false), Some(12));
    /// assert_eq!(text.display_to_byte_index(8, true), Some(12));
    /// ```
    fn display_to_byte_index(&self, display_index: usize, as_start: bool) -> Option<usize> {
        fn simple_graphemes(text: &str) -> Vec<(usize, &str)> {
            use unicode_width::UnicodeWidthChar;
            let mut graphemes = Vec::new();
            let mut start = 0;
            let mut chars = text.char_indices().peekable();
            while let Some((idx, c)) = chars.next() {
                let mut end = idx + c.len_utf8();
                while let Some(&(next_idx, next_c)) = chars.peek() {
                    let next_w = next_c.width().unwrap_or(0);
                    if next_w > 0 {
                        break;
                    }
                    chars.next();
                    end = next_idx + next_c.len_utf8();
                }
                graphemes.push((start, &text[start..end]));
                start = end;
            }
            graphemes
        }

        use unicode_width::UnicodeWidthStr;
        let mut rest = display_index;
        for (ofs, fragment) in self.display_fragments_with_index() {
            match fragment {
                Fragment::Plain(text) => {
                    if rest == 0 && !as_start {
                        return Some(ofs);
                    }
                    for (b_idx, g_str) in simple_graphemes(text) {
                        let w = UnicodeWidthStr::width(g_str);
                        if rest < w {
                            return Some(ofs + b_idx);
                        }
                        rest -= w;
                    }
                }
                Fragment::ControlSequence(_text) => {
                    if rest == 0 && as_start {
                        return Some(ofs);
                    }
                }
            }
        }
        if rest == 0 {
            return Some(self.as_ref().len());
        }
        None
    }
}

impl DisplayStr for str {}

/// Internal iterator for splitting text into fragments.
///
/// This iterator processes text and separates plain text from ANSI control sequences.
pub(crate) struct FragmentIter<'text> {
    /// The remaining text to process
    rest: &'text str,
    /// The current byte offset in the original string
    offset: usize,
}

impl<'text> FragmentIter<'text> {
    /// The ANSI CSI (Control Sequence Introducer) start sequence
    pub(crate) const CSI_START: &'static str = "\x1b[";
    /// The ANSI CSI end character for SGR (Select Graphic Rendition) sequences
    pub(crate) const CSI_END: char = 'm';

    /// Creates a new fragment iterator.
    ///
    /// # Parameters
    /// - `rest`: The text to iterate over
    ///
    /// # Returns
    /// A new iterator instance
    fn new(rest: &'text str) -> Self {
        Self { rest, offset: 0 }
    }
}

impl<'text> Iterator for FragmentIter<'text> {
    type Item = (usize, Fragment<'text>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.rest.is_empty() {
            return None;
        }
        match self.rest.split_once(Self::CSI_START) {
            Some(("", after_start)) => {
                if let Some((cs, rest)) = after_start.split_once(Self::CSI_END) {
                    let old_ofs = self.offset;
                    self.rest = rest;
                    self.offset += cs.len() + Self::CSI_END.len_utf8() + Self::CSI_START.len();
                    Some((old_ofs, Fragment::ControlSequence(cs)))
                } else {
                    let old_ofs = self.offset;
                    self.rest = after_start;
                    self.offset += Self::CSI_START.len();
                    Some((old_ofs, Fragment::Plain(Self::CSI_START)))
                }
            }
            Some((before, _)) => {
                let old_ofs = self.offset;
                self.rest = &self.rest[before.len()..];
                self.offset += before.len();
                Some((old_ofs, Fragment::Plain(before)))
            }
            _ => {
                let text = self.rest;
                self.rest = "";
                Some((self.offset, Fragment::Plain(text)))
            }
        }
    }
}

/// An iterator that iterates through the text line-wise.
///
/// # Example
/// ```rust
/// use termlayout::ext::DisplayStr;
///
/// let content = "These are\n\nsome lines\nto be iterated.\n";
/// let mut iter = content.display_lines(); // Creates this iterator.
///
/// assert_eq!(iter.next(), Some("These are"));
/// assert_eq!(iter.next(), Some(""));
/// assert_eq!(iter.next(), Some("some lines"));
/// assert_eq!(iter.next(), Some("to be iterated."));
/// assert_eq!(iter.next(), None);
/// ```
pub struct DisplayLines<'text> {
    rest: &'text str,
}

impl<'text> DisplayLines<'text> {
    fn new(rest: &'text str) -> Self {
        Self { rest }
    }
}

impl<'text> Iterator for DisplayLines<'text> {
    type Item = &'text str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rest.is_empty() {
            return None;
        }
        if let Some(ofs) = self.rest.find('\n') {
            let text = &self.rest[..ofs];
            self.rest = &self.rest[ofs + 1..];
            Some(text)
        } else {
            let text = self.rest;
            self.rest = "";
            Some(text)
        }
    }
}

/// An iterator that iterates through the text word-wise.
pub struct DisplayWords<'text> {
    rest: &'text str,
}

impl<'text> DisplayWords<'text> {
    fn new(rest: &'text str) -> Self {
        Self { rest: rest.trim() }
    }
}

impl<'text> Iterator for DisplayWords<'text> {
    type Item = &'text str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rest.is_empty() {
            return None;
        }
        if let Some(ofs) = self.rest.find(char::is_whitespace) {
            let text = &self.rest[..ofs];
            self.rest = self.rest[ofs + 1..].trim_start();
            Some(text)
        } else {
            let text = self.rest;
            self.rest = "";
            Some(text)
        }
    }
}

/// Represents a fragment of a text.
/// Instances of this type are usually generated by the [`DisplayStr::display_fragments`] method.
#[derive(Debug, PartialEq)]
pub enum Fragment<'text> {
    ///A plain text fragment, which is a sequence of UTF8 characters without any control characters.
    Plain(&'text str),

    /// A control sequence fragment, which is a sequence excluding the control sequence start
    /// ('\x1b[') and end ('m') characters.
    ControlSequence(&'text str),
}

impl Fragment<'_> {
    pub(crate) fn render<W: Write>(&self, w: &mut W) -> std::fmt::Result {
        match self {
            Fragment::Plain(x) => w.write_str(x),
            Fragment::ControlSequence(x) => {
                w.write_str(FragmentIter::CSI_START)?;
                w.write_str(x)?;
                w.write_char(FragmentIter::CSI_END)
            }
        }
    }
}

impl Display for Fragment<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.render(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_len() {
        assert_eq!("abc".display_len(), 3);
        assert_eq!("a\x1b[1mbc".display_len(), 3);
        assert_eq!("\x1b[31mred\x1b[0m".display_len(), 3);
        assert_eq!("wörld".display_len(), 5);
    }

    #[test]
    fn display_fragments() {
        let text = "a\x1b[1mbc";
        let frags: Vec<_> = text.display_fragments().collect();
        assert_eq!(
            frags,
            vec![
                Fragment::Plain("a"),
                Fragment::ControlSequence("1"),
                Fragment::Plain("bc"),
            ]
        );
    }

    #[test]
    fn display_to_byte_index() {
        let text = "a\x1b[1mbc";
        // a -> 0
        // \x1b[1m -> starts at 1, ends at 4 (length 4)
        // b -> 5
        // c -> 6
        assert_eq!(text.display_to_byte_index(0, false), Some(0));
        assert_eq!(text.display_to_byte_index(1, false), Some(5));
        assert_eq!(text.display_to_byte_index(2, false), Some(6));
        assert_eq!(text.display_to_byte_index(3, false), Some(7));
        assert_eq!(text.display_to_byte_index(4, false), None);

        assert_eq!(text.display_to_byte_index(0, true), Some(0));
        assert_eq!(text.display_to_byte_index(1, true), Some(1));
        assert_eq!(text.display_to_byte_index(2, true), Some(6));
        assert_eq!(text.display_to_byte_index(3, true), Some(7));
        assert_eq!(text.display_to_byte_index(4, true), None);
    }

    #[test]
    fn display_split_at() {
        let text = "Hello, \x1b[1mwörld!\x1b[0m";
        let (s1, s2) = text.display_split_at(7);
        assert_eq!(s1, "Hello, \x1b[1m");
        assert_eq!(s2, "wörld!\x1b[0m");
    }

    #[test]
    fn display_slice() {
        let text = "Hello, \x1b[1mwörld!\x1b[0m";
        assert_eq!(text.display_slice(7..10), "\x1b[1mwör");
        assert_eq!(text.display_slice(..7), "Hello, \x1b[1m");
        assert_eq!(text.display_slice(7..), "\x1b[1mwörld!\x1b[0m");
    }

    #[test]
    fn display_lines() {
        let text = "line1\nline2 \n\nline4";
        let lines: Vec<_> = text.display_lines().collect();
        assert_eq!(lines, vec!["line1", "line2 ", "", "line4"]);
    }

    #[test]
    fn display_words() {
        let text = "  word1  word2\nword3  ";
        let words: Vec<_> = text.display_words().collect();
        assert_eq!(words, vec!["word1", "word2", "word3"]);
    }

    #[test]
    fn display_len_unicode() {
        assert_eq!("abc".display_len(), 3);
        assert_eq!("a\x1b[1mbc".display_len(), 3);
        assert_eq!("\x1b[31mred\x1b[0m".display_len(), 3);
        assert_eq!("wörld".display_len(), 5);
        assert_eq!("🦀".display_len(), 2);
        assert_eq!("🦀🦀".display_len(), 4);
        assert_eq!("你好".display_len(), 4);
        assert_eq!("e\u{0301}".display_len(), 1); // e + combining acute accent
    }

    #[test]
    fn display_to_byte_index_unicode() {
        let text = "🦀a";
        // 🦀 starts at 0, length 4 bytes, 2 columns
        // a starts at 4, length 1 byte, 1 column
        assert_eq!(text.display_to_byte_index(0, false), Some(0));
        assert_eq!(text.display_to_byte_index(1, false), Some(0)); // Middle of emoji, should return start of emoji
        assert_eq!(text.display_to_byte_index(2, false), Some(4));
        assert_eq!(text.display_to_byte_index(3, false), Some(5));
    }

    #[test]
    fn display_split_at_unicode() {
        let text = "🦀abc";
        // Split at col 1 (middle of 🦀)
        let (s1, s2) = text.display_split_at(1);
        assert_eq!(s1, "");
        assert_eq!(s2, "🦀abc");

        // Split at col 2 (after 🦀)
        let (s1, s2) = text.display_split_at(2);
        assert_eq!(s1, "🦀");
        assert_eq!(s2, "abc");
    }

    #[test]
    fn display_slice_unicode() {
        let text = "🦀abc";
        assert_eq!(text.display_slice(..2), "🦀");
        assert_eq!(text.display_slice(2..), "abc");
        // If we slice at col 1 (middle of 🦀), it currently returns the whole emoji
        // because display_to_byte_index(1) returns the start of the emoji.
        assert_eq!(text.display_slice(1..2), "🦀");
    }
}
