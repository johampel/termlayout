use crate::ext::DisplayStr;

/// A tree decoration is used to visualize the tree relationships.
/// Technically, this consists of four strings that are prefixed before each line is rendered.
/// This is best visualized by an example:
/// ```text
/// "├─ " - Prefix for the first line of a normal node
/// "│  " - Prefix for the next lines of a normal node
/// "└─ " - Prefix for the first line of the last node of a parent node
/// "   " - Prefix for the next lines of the last node of a parent node
/// ```
///
/// # Syntax of a decoration specification
/// These prefix strings are specified via a decoration specification: This specification consists
/// of four lines, each representing one of the prefixes. All lines must have the same length and
/// end with a marker (typically 'C') representing the node content. For example:
/// ```text
/// concat!(
///     "├─ C\n",
///     "│  C\n",
///     "└─ C\n",
///     "   C\n",
/// );
/// ```
/// is the specification string for the decoration shown above.
pub struct TreeDecoration {
    last: DecorationPrefix,
    prev: DecorationPrefix,
}

struct DecorationPrefix {
    first: String,
    next: String,
}

impl TreeDecoration {
    /// Creates a new [`TreeDecoration`] using the marker "C".
    ///
    /// # Parameters
    /// - `spec`: The specification string for the decoration, see
    ///   [struct documentation](TreeDecoration) for more details
    ///
    /// # Returns
    /// The `TreeDecoration` instance created from the specification
    ///
    /// # Errors
    /// If the specification string is invalid
    pub fn from_spec<T>(spec: T) -> Result<Self, String>
    where
        T: AsRef<str>,
    {
        Self::from_spec_custom(spec, 'C')
    }

    /// Creates a new [`TreeDecoration`] using the given marker.
    ///
    /// # Parameters
    /// - `spec`: The specification string for the decoration, see
    ///   [struct documentation](TreeDecoration) for more details
    /// - `marker`: The marker character to use for marking the nodes.
    ///
    /// # Returns
    /// The `TreeDecoration` instance created from the specification
    ///
    /// # Errors
    /// If the specification string is invalid
    pub fn from_spec_custom<T>(spec: T, marker: char) -> Result<Self, String>
    where
        T: AsRef<str>,
    {
        let spec = spec.as_ref();
        let lines = spec.lines().collect::<Vec<&str>>();
        if lines.len() != 4 {
            return Err(format!(
                "invalid tree decoration spec: {} - expected 4 lines, but got {}",
                spec,
                lines.len()
            ));
        }
        for (index, line) in lines.iter().enumerate() {
            if !line.ends_with(marker) {
                return Err(format!(
                    "invalid tree decoration spec: {} - line {} does not end with marker '{}'",
                    spec,
                    index + 1,
                    marker
                ));
            }
            if index > 0 && line.display_len() != lines[index - 1].display_len() {
                return Err(format!(
                    "invalid tree decoration spec: {} - line {} does has different length",
                    spec,
                    index + 1,
                ));
            }
        }

        let prev = DecorationPrefix {
            first: lines[0][..lines[0].len() - marker.len_utf8()].to_string(),
            next: lines[1][..lines[1].len() - marker.len_utf8()].to_string(),
        };
        let last = DecorationPrefix {
            first: lines[2][..lines[2].len() - marker.len_utf8()].to_string(),
            next: lines[3][..lines[3].len() - marker.len_utf8()].to_string(),
        };

        Ok(Self { last, prev })
    }

    /// Returns a `TreeDecoration` that consists of ticks and indentation to visualize the tree
    /// relationships. A typical tree looks like
    /// ```text
    /// root
    /// - Node 1
    /// - Node 2
    ///   - Node 3
    /// ```
    ///
    /// # Parameters
    /// - `tick`: The character that represents the "tick", in the example above it is a "-"
    ///
    /// # Returns
    /// The decoration
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn ticks(tick: char) -> Self {
        Self::from_spec(format!(
            concat!(
                "{} C\n", //
                "  C\n",  //
                "{} C\n", //
                "  C"
            ),
            tick, tick
        ))
        .unwrap()
    }

    /// Returns a `TreeDecoration` that consists of lines to visualize the tree relationships.
    /// A typical tree looks like
    /// ```text
    /// root
    /// ├─ Node 1
    /// └─ Node 2
    ///    └─ Node 3
    /// ```
    ///
    /// # Parameters
    /// - `width`: The width the the tree decoration measured in how many "-" characters to use.
    ///   So the actual width is `width + 2`
    ///
    /// # Returns
    /// The decoration
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn lines(width: usize) -> Self {
        let lines = "─".repeat(width) + " ";
        let spaces = " ".repeat(width + 1);
        Self::from_spec(format!(
            concat!(
                "├{}C\n", //
                "│{}C\n", //
                "└{}C\n", //
                " {}C"
            ),
            lines, spaces, lines, spaces
        ))
        .unwrap()
    }

    /// Returns the length of the decoration prefix.
    /// Since all decoration prefixes have the same length, this is basically the length of
    /// the decoration for one tree level.
    ///
    /// # Returns
    /// The length of the decoration prefix.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::widgets::TreeDecoration;
    /// let deco = TreeDecoration::lines(2);
    ///
    /// assert_eq!(deco.prefix_len(), 4);
    /// ```
    #[must_use]
    pub fn prefix_len(&self) -> usize {
        self.prev.first.display_len()
    }

    /// Returns the prefix for the first line and for the next lines for a tree node
    ///
    /// # Parameters
    /// - `is_last`: Whether the tree node is the last child of its parent
    ///
    /// # Returns
    /// A pair of strings representing the prefix for the first line and for the next lines
    #[must_use]
    pub fn prefixes(&self, is_last: bool) -> (&str, &str) {
        if is_last {
            (&self.last.first, &self.last.next)
        } else {
            (&self.prev.first, &self.prev.next)
        }
    }
}

impl Default for TreeDecoration {
    fn default() -> Self {
        Self::lines(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree_decoration_indent() {
        let deco = TreeDecoration::ticks('-');
        assert_eq!(deco.prev.first, "- ");
        assert_eq!(deco.prev.next, "  ");
        assert_eq!(deco.last.first, "- ");
        assert_eq!(deco.last.next, "  ");
    }

    #[test]
    fn tree_decoration_lines() {
        let deco = TreeDecoration::lines(2);
        assert_eq!(deco.prev.first, "├── ");
        assert_eq!(deco.prev.next, "│   ");
        assert_eq!(deco.last.first, "└── ");
        assert_eq!(deco.last.next, "    ");
    }

    #[test]
    fn tree_decoration_from_spec_valid() {
        let spec = "A+C\nB+C\nD+C\nE+C";
        let deco = TreeDecoration::from_spec(spec).unwrap();
        assert_eq!(deco.prev.first, "A+");
        assert_eq!(deco.prev.next, "B+");
        assert_eq!(deco.last.first, "D+");
        assert_eq!(deco.last.next, "E+");
    }

    #[test]
    fn tree_decoration_from_spec_custom_valid() {
        let spec = "A+X\nB+X\nD+X\nE+X";
        let deco = TreeDecoration::from_spec_custom(spec, 'X').unwrap();
        assert_eq!(deco.prev.first, "A+");
        assert_eq!(deco.prev.next, "B+");
        assert_eq!(deco.last.first, "D+");
        assert_eq!(deco.last.next, "E+");
    }

    #[test]
    fn tree_decoration_from_spec_invalid_lines() {
        let spec = "A+C\nB+C\nD+C";
        assert!(TreeDecoration::from_spec(spec).is_err());
    }

    #[test]
    fn tree_decoration_from_spec_missing_marker() {
        let spec = "A+C\nB+C\nD+\nE+C";
        assert!(TreeDecoration::from_spec(spec).is_err());
    }

    #[test]
    fn tree_decoration_from_spec_different_lengths() {
        let spec = "A+C\nBB+C\nD+C\nE+C";
        assert!(TreeDecoration::from_spec(spec).is_err());
    }
}
