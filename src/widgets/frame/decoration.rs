use crate::Dimension;
use crate::ext::DisplayStr;
use crate::widgets::LinesAlignment;
use std::collections::HashMap;

/// Represents the decoration of a [`Frame`](super::Frame).
/// The decoration consists of two information:
///
/// 1. The characters to draw the frame around the content. This is typically specified via a
///    specification string that visualizes the the frame. In the specification string the letter
///    'C' represents the content, all other letters represent the frame. For example, the string
///    "┌─┐\n│C│\n└─┘" represents a single line round the content. The string "│C" would represent
///    a frame that consists just of a line on the left side of the content.
///
/// 2. The placement of the title.
///    The placement of the title is specified via a [`TitlePlacement`] struct, which contains the
///    vertical and horizontal alignment of the title.
///
/// # Example
/// ```rust
///
/// use termlayout::Layout;
/// use termlayout::widgets::{Frame, FrameDecoration, Paragraph, TitlePlacement};
///
/// let decoration = FrameDecoration::from_spec(concat!(
///     "+---+\n",
///     "| C |\n",
///     "+---+",""))
///     .unwrap().with_title_placement(TitlePlacement::default());
/// let frame = Frame::new(
///     decoration,
///     Some("The title".to_string()),
///     Paragraph::left("The content inside the frame"));
///
/// assert_eq!(format!("{}", frame.layout(15)), concat!(
///     "+-The title---+\n",
///     "| The content |\n",
///     "| inside the  |\n",
///     "| frame       |\n",
///     "+-------------+\n"
/// ));
/// ```
#[derive(Debug)]
pub struct FrameDecoration {
    /// The characters to draw the frame around the content. This is basically a map from
    /// [`FrameDecorationKey`] to a [`String`].
    pub frame: HashMap<FrameDecorationKey, String>,

    /// The placement of the title, represented as a [`TitlePlacement`].
    pub title_placement: TitlePlacement,
}

impl FrameDecoration {
    fn new(frame: HashMap<FrameDecorationKey, String>) -> Self {
        Self {
            frame,
            title_placement: TitlePlacement::default(),
        }
    }

    /// Creates a [`FrameDecoration`] having a single line around the content.
    /// The frame looks like:
    /// ```text
    /// ┌───────────┐
    /// │The content│
    /// └───────────┘
    /// ```
    ///
    /// # Returns
    /// The single-lined `FrameDecoration`
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn boxed() -> Self {
        Self::from_spec(concat!(
            "┌─┐\n", //
            "│C│\n", //
            "└─┘"
        ))
        .unwrap()
    }

    /// Creates a [`FrameDecoration`] having a double line around the content.
    /// The frame looks like:
    /// ```text
    /// ╔═══════════╗
    /// ║The content║
    /// ╚═══════════╝
    /// ```
    ///
    /// # Returns
    /// The double-lined `FrameDecoration`
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn double_boxed() -> Self {
        Self::from_spec(concat!("╔═╗\n", "║C║\n", "╚═╝")).unwrap()
    }

    /// Creates a new [`FrameDecoration`] from a specification string.
    /// The specification string consists of up to three lines, already visualizing the frame. In
    /// the specification string, the letter "C" is a placeholder for the content - all other
    /// characters represent the frame. One frame element (for example the top border) must be at
    /// most one line high.
    ///
    /// # Parameters
    /// - `spec`: The specification string
    ///
    /// # Returns
    /// A `FrameDecoration` or an `Err`.
    ///
    /// # Errors
    /// In case the specification string contains an incorrect specification
    ///
    /// # Examples
    /// Some examples for the spec string are:
    /// - No border at all:
    ///     ```text
    ///     "C"
    ///     ```
    /// - A border with a single line around the content
    ///     ```text
    ///     concat!(
    ///      "┌─┐\n",
    ///      "│C│\n",
    ///      "└─┘\n"
    ///     )
    ///     ```
    /// - Just a vertical line on the left side of the content:
    ///     ```text
    ///     "│ C"
    ///     ```
    pub fn from_spec(spec: &str) -> Result<Self, String> {
        FrameDecorationParser::parse('C', spec)
    }

    /// Creates a new [`FrameDecoration`] with the given [`TitlePlacement`].
    ///
    /// # Parameters
    /// - `title`: The `TitlePlacement` to use
    ///
    /// # Returns
    /// A new instance with the given `TitlePlacement`
    #[must_use]
    pub fn with_title_placement(self, title: TitlePlacement) -> Self {
        Self {
            title_placement: title,
            ..self
        }
    }

    /// Calculates the [`Dimension`] of the frame given the [`Dimension`] of the content.
    ///
    /// # Parameters
    /// - `content_dim`: The [`Dimension`] of the content
    /// - `has_title`: Whether the frame has a title or not. This might influence the
    ///   calculation depending on the active [`TitlePlacement`]
    ///
    /// # Returns
    /// The [`Dimension`] of the frame
    ///
    /// # Example
    /// ```rust
    ///
    /// use termlayout::Dimension;
    /// use termlayout::widgets::{FrameDecoration, TitlePlacement};
    ///
    /// let decoration = FrameDecoration::boxed()
    ///                         .with_title_placement(TitlePlacement::default().with_inside(true));
    /// let content_dim = Dimension::new(10, 5);
    ///
    /// assert_eq!(decoration.frame_dim(content_dim, false), Dimension::new(12, 7));
    /// assert_eq!(decoration.frame_dim(content_dim, true), Dimension::new(12, 8));
    /// ```
    #[must_use]
    pub fn frame_dim(&self, content_dim: Dimension, has_title: bool) -> Dimension {
        Dimension::new(
            content_dim.width + self.get_left_margin() + self.get_right_margin(),
            content_dim.height + self.get_top_margin(has_title) + self.get_bottom_margin(has_title),
        )
    }

    /// Gets the string representing the frame decoration for the given `key`.
    ///
    /// # Parameters
    /// - `key`: A [`FrameDecorationKey`] describing which part of the frame is queried
    ///
    /// # Returns
    /// The according string or `None`
    #[must_use]
    pub fn get_frame(&self, key: FrameDecorationKey) -> Option<&str> {
        self.frame.get(&key).map(String::as_str)
    }

    /// Gets the left margin of the frame.
    /// This is the number of columns required to display the frame decoration on the left side.
    ///
    /// # Returns
    /// The left margin
    #[must_use]
    pub fn get_left_margin(&self) -> usize {
        self.longest_frame(FrameDecorationKey::is_west)
    }

    /// Gets the right margin of the frame.
    /// This is the number of columns required to display the frame decoration on the right side.
    ///
    /// # Returns
    /// The right margin
    #[must_use]
    pub fn get_right_margin(&self) -> usize {
        self.longest_frame(FrameDecorationKey::is_east)
    }

    /// Gets the top margin of the frame.
    /// This is the number of columns required to display the frame decoration on the top side.
    ///
    /// # Parameters
    /// - `has_title`: Whether the frame has a title
    ///
    /// # Returns
    /// The top margin
    #[must_use]
    pub fn get_top_margin(&self, has_title: bool) -> usize {
        usize::from(self.has_frame(FrameDecorationKey::is_north))
            + usize::from(has_title && !self.title_placement.bottom && self.title_placement.inside)
    }

    /// Gets the bottom margin of the frame.
    /// This is the number of columns required to display the frame decoration on the bottom side.
    ///
    /// # Parameters
    /// - `has_title`: Whether the frame has a title
    ///
    /// # Returns
    /// The bottom margin
    #[must_use]
    pub fn get_bottom_margin(&self, has_title: bool) -> usize {
        usize::from(self.has_frame(FrameDecorationKey::is_south))
            + usize::from(has_title && self.title_placement.bottom && self.title_placement.inside)
    }

    fn longest_frame<P>(&self, predicate: P) -> usize
    where
        P: Fn(&FrameDecorationKey) -> bool,
    {
        self.frame
            .iter()
            .filter(|(k, _)| predicate(k))
            .map(|(_, v)| v.display_len())
            .max()
            .unwrap_or(0)
    }

    /// Checks whether the decoration contains a frame segment matching the given predicate.
    ///
    /// # Parameters
    ///- `predicate`: A Function that accepts a [`FrameDecorationKey`] and returns a `bool`
    ///
    /// # Returns
    /// `true`, if the decoration contains a frame segment matching the given predicate.
    /// Otherwise `false`.
    #[must_use]
    pub fn has_frame<P>(&self, predicate: P) -> bool
    where
        P: Fn(&FrameDecorationKey) -> bool,
    {
        self.frame.iter().any(|(k, _)| predicate(k))
    }
}

/// Identifies a frame decoration segment.
/// A frame consists of different segments, for example, for the left top edge, the line at the
/// bottom and so on. This enumeration allows identifying them.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum FrameDecorationKey {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl FrameDecorationKey {
    /// Checks whether the key is somehow related to the north of the frame decoration.
    ///
    /// # Returns
    /// `true`, if related to the north of the frame
    #[must_use]
    pub fn is_north(&self) -> bool {
        matches!(
            self,
            FrameDecorationKey::North
                | FrameDecorationKey::NorthWest
                | FrameDecorationKey::NorthEast
        )
    }

    /// Checks whether the key is somehow related to the east of the frame decoration.
    ///
    /// # Returns
    /// `true`, if related to the east of the frame
    #[must_use]
    pub fn is_east(&self) -> bool {
        matches!(
            self,
            FrameDecorationKey::East
                | FrameDecorationKey::NorthEast
                | FrameDecorationKey::SouthEast
        )
    }

    /// Checks whether the key is somehow related to the west of the frame decoration.
    ///
    /// # Returns
    /// `true`, if related to the west of the frame
    #[must_use]
    pub fn is_west(&self) -> bool {
        matches!(
            self,
            FrameDecorationKey::West
                | FrameDecorationKey::NorthWest
                | FrameDecorationKey::SouthWest
        )
    }

    /// Checks whether the key is somehow related to the south of the frame decoration.
    ///
    /// # Returns
    /// `true`, if related to the south of the frame
    #[must_use]
    pub fn is_south(&self) -> bool {
        matches!(
            self,
            FrameDecorationKey::South
                | FrameDecorationKey::SouthWest
                | FrameDecorationKey::SouthEast
        )
    }
}

/// Defines the title placement of a [`FrameDecoration`].
/// The placement is defined in terms of the horizontal align (using a [`LinesAlignment`]) and the
/// vertical alignment (using two flags indicating whether the title is at the top or the bottom
/// and whether it is placed onto the frame or inside it).
///
/// Usually, instances of this struct are created with the `default()` method and then it is
/// customized by one or more of the `with_*()` methods.
#[derive(Debug)]
pub struct TitlePlacement {
    /// The horzontal alignment of the title.
    pub alignment: LinesAlignment,

    /// Flag indicating whether the title is inside the frame, or not (integrated into the
    /// frame decoration).
    pub inside: bool,

    /// Flag indicating whether the title is placed at the top or bottom of the frame.
    pub bottom: bool,
}

impl Default for TitlePlacement {
    fn default() -> Self {
        Self {
            alignment: LinesAlignment::Left,
            inside: false,
            bottom: false,
        }
    }
}

impl TitlePlacement {
    /// Creates a new instance with the given horizontal alignment.
    ///
    /// # Parameters
    /// - `alignment`: The horizontal alignment of the title, represented as a [`LinesAlignment`].
    ///
    /// # Returns
    /// A new instance of [`TitlePlacement`] with the given alignment.
    #[must_use]
    pub fn with_alignment(self, alignment: LinesAlignment) -> Self {
        Self { alignment, ..self }
    }

    /// Creates a new instance with the given `inside` flag.
    /// If the flag is `true`, then the title will be placed inside the frame, otherwise onto the
    /// frame.
    ///
    /// # Parameters
    /// - `inside`: The `inside` flag.
    ///
    /// # Returns
    /// A new instance of [`TitlePlacement`] with the given `inside` flag.
    #[must_use]
    pub fn with_inside(self, inside: bool) -> Self {
        Self { inside, ..self }
    }

    /// Creates a new instance with the given `bottom` flag.
    /// If the flag is `true`, then the title will be placed at the bottom of the frame, otherwise
    /// at the top of the frame.
    ///
    /// # Parameters
    /// - `bottom`: The `bottom` flag.
    ///
    /// # Returns
    /// A new instance of [`TitlePlacement`] with the given `bottom` flag.
    #[must_use]
    pub fn with_bottom(self, bottom: bool) -> Self {
        Self { bottom, ..self }
    }
}

struct FrameDecorationParser<'a> {
    lines: Vec<&'a str>,
    marker_pos: (usize, usize),
}

impl<'a> FrameDecorationParser<'a> {
    fn parse(content_marker: char, spec: &'a str) -> Result<FrameDecoration, String> {
        let mut this = Self {
            lines: spec.lines().collect(),
            marker_pos: (0, 0),
        };
        this.find_marker(content_marker)?;

        Ok(this.build_decoration())
    }

    fn find_marker(&mut self, content_marker: char) -> Result<(), String> {
        if self.lines.is_empty() || self.lines.len() > 3 {
            return Err("spec must have 1-3 lines".to_string());
        }
        self.marker_pos = (self.lines.len(), 0);

        for (line_index, line) in self.lines.iter().enumerate() {
            if let Some(col_index) = line.find(content_marker) {
                let col_index = line[..col_index].display_len();
                if self.marker_pos.0 != self.lines.len() {
                    return Err("spec contains more than one line with a marker".to_string());
                }
                self.marker_pos = (line_index, col_index);
            }
        }

        if self.marker_pos.0 > 1
            || self.marker_pos.0 + 2 < self.lines.len()
            || self.marker_pos.0 == self.lines.len()
        {
            return Err("line with marker misplaced or not present".to_string());
        }
        Ok(())
    }

    fn build_decoration(&self) -> FrameDecoration {
        let mut frame = HashMap::new();

        let col = self.marker_pos.1;

        if self.marker_pos.0 > 0 {
            if col > 0 {
                frame.insert(
                    FrameDecorationKey::NorthWest,
                    self.lines[0].display_slice(..col).to_string(),
                );
            }
            frame.insert(
                FrameDecorationKey::North,
                self.lines[0].display_slice(col..=col).to_string(),
            );
            if col < self.lines[0].display_len() - 1 {
                frame.insert(
                    FrameDecorationKey::NorthEast,
                    self.lines[0].display_slice(col + 1..).to_string(),
                );
            }
        }

        let row = self.marker_pos.0;
        if col > 0 {
            frame.insert(
                FrameDecorationKey::West,
                self.lines[row].display_slice(..col).to_string(),
            );
        }
        if col < self.lines[row].display_len() - 1 {
            frame.insert(
                FrameDecorationKey::East,
                self.lines[row].display_slice(col + 1..).to_string(),
            );
        }

        let row = self.marker_pos.0 + 1;
        if row < self.lines.len() {
            if col > 0 {
                frame.insert(
                    FrameDecorationKey::SouthWest,
                    self.lines[row].display_slice(..col).to_string(),
                );
            }
            frame.insert(
                FrameDecorationKey::South,
                self.lines[row].display_slice(col..=col).to_string(),
            );
            if col < self.lines[row].display_len() - 1 {
                frame.insert(
                    FrameDecorationKey::SouthEast,
                    self.lines[row].display_slice(col + 1..).to_string(),
                );
            }
        }

        FrameDecoration::new(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame_value(decoration: &FrameDecoration, key: FrameDecorationKey) -> Option<&str> {
        decoration.frame.get(&key).map(String::as_str)
    }

    #[test]
    fn frame_decoration_parser_parse_single_line_marker_only() {
        let decoration = FrameDecorationParser::parse('C', "C").unwrap();

        assert_eq!(decoration.frame.len(), 0);
    }

    #[test]
    fn frame_decoration_parser_parse_single_line_with_sides() {
        let decoration = FrameDecorationParser::parse('C', "<C>").unwrap();

        assert_eq!(decoration.frame.len(), 2);
        assert_eq!(
            frame_value(&decoration, FrameDecorationKey::West),
            Some("<")
        );
        assert_eq!(
            frame_value(&decoration, FrameDecorationKey::East),
            Some(">")
        );
        assert_eq!(frame_value(&decoration, FrameDecorationKey::North), None);
        assert_eq!(frame_value(&decoration, FrameDecorationKey::South), None);
    }

    #[test]
    fn frame_decoration_parser_parse_top_and_middle() {
        let decoration = FrameDecorationParser::parse('C', "/-\\\n|C|").unwrap();

        assert_eq!(decoration.frame.len(), 5);
        assert_eq!(
            frame_value(&decoration, FrameDecorationKey::NorthWest),
            Some("/")
        );
        assert_eq!(
            frame_value(&decoration, FrameDecorationKey::North),
            Some("-")
        );
        assert_eq!(
            frame_value(&decoration, FrameDecorationKey::NorthEast),
            Some("\\")
        );
        assert_eq!(
            frame_value(&decoration, FrameDecorationKey::West),
            Some("|")
        );
        assert_eq!(
            frame_value(&decoration, FrameDecorationKey::East),
            Some("|")
        );
        assert_eq!(frame_value(&decoration, FrameDecorationKey::South), None);
    }

    #[test]
    fn frame_decoration_parser_parse_middle_and_bottom() {
        let decoration = FrameDecorationParser::parse('C', "|C|\n\\-/").unwrap();

        assert_eq!(decoration.frame.len(), 5);
        assert_eq!(
            frame_value(&decoration, FrameDecorationKey::West),
            Some("|")
        );
        assert_eq!(
            frame_value(&decoration, FrameDecorationKey::East),
            Some("|")
        );
        assert_eq!(
            frame_value(&decoration, FrameDecorationKey::SouthWest),
            Some("\\")
        );
        assert_eq!(
            frame_value(&decoration, FrameDecorationKey::South),
            Some("-")
        );
        assert_eq!(
            frame_value(&decoration, FrameDecorationKey::SouthEast),
            Some("/")
        );
        assert_eq!(frame_value(&decoration, FrameDecorationKey::North), None);
    }

    #[test]
    fn frame_decoration_parser_parse_full_frame() {
        let decoration = FrameDecorationParser::parse('C', "/-\\\n|C|\n\\-/").unwrap();

        assert_eq!(decoration.frame.len(), 8);
        assert_eq!(
            frame_value(&decoration, FrameDecorationKey::NorthWest),
            Some("/")
        );
        assert_eq!(
            frame_value(&decoration, FrameDecorationKey::North),
            Some("-")
        );
        assert_eq!(
            frame_value(&decoration, FrameDecorationKey::NorthEast),
            Some("\\")
        );
        assert_eq!(
            frame_value(&decoration, FrameDecorationKey::West),
            Some("|")
        );
        assert_eq!(
            frame_value(&decoration, FrameDecorationKey::East),
            Some("|")
        );
        assert_eq!(
            frame_value(&decoration, FrameDecorationKey::SouthWest),
            Some("\\")
        );
        assert_eq!(
            frame_value(&decoration, FrameDecorationKey::South),
            Some("-")
        );
        assert_eq!(
            frame_value(&decoration, FrameDecorationKey::SouthEast),
            Some("/")
        );
    }

    #[test]
    fn frame_decoration_from_spec_uses_default_content_marker() {
        let decoration = FrameDecoration::from_spec("[C]").unwrap();

        assert_eq!(
            frame_value(&decoration, FrameDecorationKey::West),
            Some("[")
        );
        assert_eq!(
            frame_value(&decoration, FrameDecorationKey::East),
            Some("]")
        );
    }

    #[test]
    fn frame_decoration_parser_parse_fail_wrong_line_count() {
        assert_eq!(
            FrameDecorationParser::parse('C', "").unwrap_err(),
            "spec must have 1-3 lines".to_string()
        );

        assert_eq!(
            FrameDecorationParser::parse('C', "xxx\nxCx\nxxx\nxxx").unwrap_err(),
            "spec must have 1-3 lines".to_string()
        );
    }

    #[test]
    fn frame_decoration_parser_parse_fail_marker_in_more_than_one_line() {
        assert_eq!(
            FrameDecorationParser::parse('C', "C\nC").unwrap_err(),
            "spec contains more than one line with a marker".to_string()
        );

        assert_eq!(
            FrameDecorationParser::parse('C', "xxx\nxCx\nxxC").unwrap_err(),
            "spec contains more than one line with a marker".to_string()
        );
    }

    #[test]
    fn frame_decoration_parser_parse_fail_marker_missing_or_misplaced() {
        assert_eq!(
            FrameDecorationParser::parse('C', "---").unwrap_err(),
            "line with marker misplaced or not present".to_string()
        );

        assert_eq!(
            FrameDecorationParser::parse('C', "xCx\n---\n---").unwrap_err(),
            "line with marker misplaced or not present".to_string()
        );

        assert_eq!(
            FrameDecorationParser::parse('C', "---\n---\nxCx").unwrap_err(),
            "line with marker misplaced or not present".to_string()
        );
    }
}
