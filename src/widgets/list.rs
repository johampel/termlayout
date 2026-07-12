use crate::ext::{
    BaseLayoutWriter, BoxedLayoutWriter, DisplayStr, FormattedLayout, LayoutWithOptions,
    LayoutWriter, SizedLayoutResult,
};
use crate::widgets::lines::LinesTrimming;
use crate::widgets::vertical::FormattedVertical;
use crate::widgets::{Lines, LinesAlignment};
use crate::{
    BoxedFormattedLayout, Dimension, Layout, LayoutOptions, RcLayout, Rect, WrapMode,
    box_formatted_layout, rc_layout,
};
use std::any::Any;
use std::cmp::max;
use std::fmt::Write;

/// A widget that displays a list of items with enumeration markers.
///
/// Similar to the [`crate::widgets::Vertical`] widget, the `items` are organized vertically,
/// but each item is prefixed with a marker that enumerates it.
///
/// The `marker` is a [`ListItemMarker`] that formats this prefix. Depending on the
/// marker type, this can be a fixed string, decimal numbers, Roman numerals, alphabetic
/// characters, or a custom enumeration function.
///
/// # Example
/// ```rust
/// use termlayout::Layout;
/// use termlayout::widgets::{List, ListItemMarker, Paragraph};
///
/// let list = List::new(ListItemMarker::default_fixed(), vec![
///     Paragraph::left("fixed markers enumerate items always with a fixed prefix, such as '-'."),
///     Paragraph::left("decimal markers enumerate like '1. ', '2. ', '3. ', '4. ', etc."),
///     Paragraph::left("alphabetic markers enumerate like 'a) ', 'b) ', 'c) ', 'd) ', etc."),
///     Paragraph::left("roman markers enumerate like 'i. ', 'ii. ', 'iii. ', 'iv. ', etc.")
/// ]);
///
/// assert_eq!(format!("{}", list.layout(40)), concat!(
///     "• fixed markers enumerate items always\n",
///     "  with a fixed prefix, such as '-'.\n",
///     "• decimal markers enumerate like '1. ',\n",
///     "  '2. ', '3. ', '4. ', etc.\n",
///     "• alphabetic markers enumerate like 'a)\n",
///     "  ', 'b) ', 'c) ', 'd) ', etc.\n",
///     "• roman markers enumerate like 'i. ',\n",
///     "  'ii. ', 'iii. ', 'iv. ', etc.\n"
/// ));
/// ```
pub struct List {
    /// The [`ListItemMarker`] used for enumeration
    pub marker: ListItemMarker,

    /// The list of items to be displayed
    pub items: Vec<RcLayout>,
}

impl List {
    /// Creates a new [`List`] layout
    ///
    /// # Parameters
    /// - `marker`: The [`ListItemMarker`] used for enumeration
    /// - `items`: The list of items to be displayed
    ///
    /// # Returns
    /// The `List`
    pub fn new<T>(marker: ListItemMarker, items: Vec<T>) -> Self
    where
        T: Into<RcLayout>,
    {
        Self {
            marker,
            items: items.into_iter().map(Into::into).collect(),
        }
    }

    /// Creates a list with fixed markers.
    ///
    /// # Parameters
    /// - `items`: The list of items to be displayed
    ///
    /// # Returns
    /// The `List`
    ///
    /// # Example
    /// ```rust
    /// use termlayout::Layout;
    /// use termlayout::widgets::{List, ListItemMarker, Paragraph};
    ///
    /// let list = List::fixed(vec![
    ///     Paragraph::left("Item 1"),
    ///     Paragraph::left("Item 2"),
    ///     Paragraph::left("Item 3"),
    /// ]);
    ///
    /// assert_eq!(format!("{}", list.layout(20)), concat!(
    ///     "• Item 1\n",
    ///     "• Item 2\n",
    ///     "• Item 3\n"
    /// ));
    /// ```
    #[must_use]
    pub fn fixed<T>(items: Vec<T>) -> Self
    where
        T: Into<RcLayout>,
    {
        Self::new(ListItemMarker::default_fixed(), items)
    }

    /// Creates a list with numeric markers.
    ///
    /// # Parameters
    /// - `items`: The list of items to be displayed
    ///
    /// # Returns
    /// The `List`
    ///
    /// # Example
    /// ```rust
    /// use termlayout::Layout;
    /// use termlayout::widgets::{List, ListItemMarker, Paragraph};
    ///
    /// let list = List::numbered(vec![
    ///     Paragraph::left("Item 1"),
    ///     Paragraph::left("Item 2"),
    ///     Paragraph::left("Item 3"),
    /// ]);
    ///
    /// assert_eq!(format!("{}", list.layout(20)), concat!(
    ///     "1. Item 1\n",
    ///     "2. Item 2\n",
    ///     "3. Item 3\n"
    /// ));
    /// ```
    #[must_use]
    pub fn numbered<T>(items: Vec<T>) -> Self
    where
        T: Into<RcLayout>,
    {
        Self::new(ListItemMarker::default_numbered(), items)
    }

    /// Creates a list with alphabetic markers.
    ///
    /// # Parameters
    /// - `items`: The list of items to be displayed
    ///
    /// # Returns
    /// The `List`
    ///
    /// # Example
    /// ```rust
    /// use termlayout::Layout;
    /// use termlayout::widgets::{List, ListItemMarker, Paragraph};
    ///
    /// let list = List::alphabetic(vec![
    ///     Paragraph::left("Item 1"),
    ///     Paragraph::left("Item 2"),
    ///     Paragraph::left("Item 3"),
    /// ]);
    ///
    /// assert_eq!(format!("{}", list.layout(20)), concat!(
    ///     "a) Item 1\n",
    ///     "b) Item 2\n",
    ///     "c) Item 3\n"
    /// ));
    /// ```
    #[must_use]
    pub fn alphabetic<T>(items: Vec<T>) -> Self
    where
        T: Into<RcLayout>,
    {
        Self::new(ListItemMarker::default_alpha(), items)
    }

    /// Creates a list with Roman numeral markers.
    ///
    /// # Parameters
    /// - `items`: The list of items to be displayed
    ///
    /// # Returns
    /// The `List`
    ///
    /// # Example
    /// ```rust
    /// use termlayout::Layout;
    /// use termlayout::widgets::{List, ListItemMarker, Paragraph};
    ///
    /// let list = List::roman(vec![
    ///     Paragraph::left("Item 1"),
    ///     Paragraph::left("Item 2"),
    ///     Paragraph::left("Item 3"),
    /// ]);
    ///
    /// assert_eq!(format!("{}", list.layout(20)), concat!(
    ///     "i.   Item 1\n",
    ///     "ii.  Item 2\n",
    ///     "iii. Item 3\n"
    /// ));
    /// ```
    #[must_use]
    pub fn roman<T>(items: Vec<T>) -> Self
    where
        T: Into<RcLayout>,
    {
        Self::new(ListItemMarker::default_roman(), items)
    }

    fn create_marker(&self, index: usize) -> RcLayout {
        Lines::new(
            self.marker.alignment,
            LinesTrimming::None,
            None,
            self.marker.format(index),
        )
        .into()
    }

    fn calculate_widths(&self, max_width: usize) -> (usize, usize) {
        if max_width > 0 {
            let marker_width = self.marker.max_width(self.items.len());
            let item_width = max(1, max_width.saturating_sub(marker_width));
            (max_width - item_width, item_width)
        } else {
            (0, 0)
        }
    }
}

impl Layout for List {
    fn pref_dim(&self, max_width: usize, wrap_mode: WrapMode) -> Dimension {
        let (marker_width, item_width) = self.calculate_widths(max_width);
        let mut dim = self.items.iter().fold(Dimension::empty(), |acc, item| {
            acc.vertical_union(item.pref_dim(item_width, wrap_mode))
        });
        dim.width += marker_width;
        dim
    }

    fn min_dim(&self) -> Dimension {
        let mut dim = self.items.iter().fold(Dimension::empty(), |acc, item| {
            acc.vertical_union(item.min_dim())
        });
        dim.width += self.marker.max_width(self.items.len());
        dim
    }

    fn layout_strict(&'_ self, options: LayoutOptions) -> BoxedFormattedLayout<'_> {
        let (marker_width, item_width) = self.calculate_widths(options.dim.width);
        let mut row = 0;
        let content = self
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let item_dim = item.pref_dim(item_width, options.wrap_mode);
                let item_options = options.intersect(Rect::new(marker_width, row, item_dim), false);
                let line_options = options
                    .intersect(
                        Rect::new(0, row, Dimension::new(options.dim.width, item_dim.height)),
                        true,
                    )
                    .with_normalized_horizontal_clip();
                let marker_options = options.intersect(
                    Rect::new(0, row, Dimension::new(marker_width, item_dim.height)),
                    true,
                );
                let marker = self.create_marker(index);
                row += item_dim.height;
                FormattedListItem::new(
                    line_options,
                    LayoutWithOptions::of(marker, marker_options).into(),
                    item.layout_strict(item_options),
                )
                .into()
            })
            .collect();
        FormattedVertical::new(content, options.with_normalized_horizontal_clip()).into()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

rc_layout!(List);

struct FormattedListItem<'fmt> {
    options: LayoutOptions,
    marker: BoxedFormattedLayout<'fmt>,
    item: BoxedFormattedLayout<'fmt>,
}

impl<'fmt> FormattedListItem<'fmt> {
    fn new(
        options: LayoutOptions,
        marker: BoxedFormattedLayout<'fmt>,
        item: BoxedFormattedLayout<'fmt>,
    ) -> Self {
        Self {
            options,
            marker,
            item,
        }
    }
}

impl FormattedLayout for FormattedListItem<'_> {
    fn options(&self) -> &LayoutOptions {
        &self.options
    }

    fn new_writer(&'_ self) -> BoxedLayoutWriter<'_> {
        Box::new(ListItemWriter::new(
            self.marker.new_writer(),
            self.item.new_writer(),
            &self.options,
        ))
    }
}

box_formatted_layout!(FormattedListItem);

struct ListItemWriter<'wrt> {
    base: BaseLayoutWriter<'wrt>,
    marker: BoxedLayoutWriter<'wrt>,
    item: BoxedLayoutWriter<'wrt>,
}

impl<'wrt> ListItemWriter<'wrt> {
    fn new(
        marker: BoxedLayoutWriter<'wrt>,
        item: BoxedLayoutWriter<'wrt>,
        options: &'wrt LayoutOptions,
    ) -> Self {
        Self {
            base: BaseLayoutWriter::new(options),
            marker,
            item,
        }
    }
}

impl<'wrt> LayoutWriter<'wrt> for ListItemWriter<'wrt> {
    fn options(&self) -> &'wrt LayoutOptions {
        self.base.options()
    }

    fn write_row(&mut self, w: &mut dyn Write) -> SizedLayoutResult {
        self.base.write_row(self.marker.as_mut(), w)?;
        self.base.write_row(self.item.as_mut(), w)?;
        self.base.end_row(w)
    }
}

/// Defines how to display and enumerate the items in a [`List`].
/// The display format of a marker is defined via a "spec", a string that defines the format of
/// the display string and how it is aligned. The enumeration of the items is done by the
/// [`ListItemEnumerator`].
///
/// The `ListItemEnumerator` defines only the enumeration string of the item, without any additional
/// formatting. For example, `ListItemEnumerator::Decimal` generates just the numbers "1", "2", etc.
/// But in the final list, the item marker has further information, such as the alignment and
/// optional a suffix and prefix. So with the suffix ". ", the final item marker string becomes
/// "1. ", "2. ", etc.
///
/// # Syntax of the spec string.
/// The formal syntax of the spec string is: `` `<alignment><prefix><enumeration><suffix>` ``, whereas:
/// `` `<alignment>` `` is optional and can be one of `` `<` `` (left alignment), `` `>` `` (right alignment), `` `|` ``
/// (center alignment), or empty (default, left alignment). `` `<prefix>` `` and `` `<suffix>` `` are optional;
/// `` `<enumeration>` `` is mandatory and always `X` (the concrete enumerator is specified separately).
///
/// So "<X. " is a valid spec string, meaning that the marker will be left aligned, with no prefix
/// and the suffix ". ".
///
/// *Note*: The implementation assumes that neither the prefix, the suffix nor the enumerator produce
/// new lines.
#[derive(Debug, PartialEq)]
pub struct ListItemMarker {
    /// The alignment of the marker.
    alignment: LinesAlignment,

    /// The prefix of the marker.
    prefix: String,

    /// The suffix of the marker.
    suffix: String,

    /// The concrete enumeration style.
    enumerator: ListItemEnumerator,

    /// The index of the first item in the list.
    first_index: usize,
}

impl ListItemMarker {
    /// Returns a new `ListItemMarker` with the same settings as this one but the given `style`
    /// This function is typically used right after `default` or `with_spec` to build a new marker.
    ///
    /// # Parameters
    /// - `style`: The [`ListItemEnumerator`]
    ///
    /// # Returns
    /// A new instance with the given `style`
    ///
    /// ```rust
    /// use termlayout::widgets::{ListItemEnumerator, ListItemMarker};
    ///
    /// let marker = ListItemMarker::default()
    ///     .with_enumerator(ListItemEnumerator::LowerAlpha);
    ///
    /// assert_eq!(marker.format(1), "b ");
    /// ```
    #[must_use]
    pub fn with_enumerator(self, style: ListItemEnumerator) -> Self {
        Self {
            enumerator: style,
            ..self
        }
    }

    /// Returns a new `ListItemMarker` with the same settings as this one but the given `first_index`.
    ///
    ///
    /// # Parameters
    /// - `first_index`: The index of the first item in the list. This is added to the "natural"
    ///   index of each item.
    ///
    /// # Returns
    /// A new instance with the given `first_index`.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::widgets::{ListItemEnumerator, ListItemMarker};
    ///
    /// let marker = ListItemMarker::default_numbered()
    ///     .with_first_index(5);
    ///
    /// assert_eq!(marker.format(0), "6. ");
    /// ```
    #[must_use]
    pub fn with_first_index(self, first_index: usize) -> Self {
        Self {
            first_index,
            ..self
        }
    }

    /// Returns a new `ListItemMarker` with the same settings as this one but the given `spec`
    /// This function is typically used right after `default` or `with_style` to build a new marker.
    ///
    /// # Parameters
    /// - `spec`: The spec string as described in the [class documentation](ListItemMarker)
    ///
    /// # Returns
    /// A new instance with the given `style` or an error
    ///
    /// # Errors
    /// If `spec` is not a valid spec-string
    pub fn try_with_spec(self, spec: &str) -> Result<Self, String> {
        ListItemMakerParser::parse('X', self.enumerator, spec)
    }

    /// Returns a new `ListItemMarker` with the same settings as this one but the given `spec`.
    /// This function is typically used right after `default` or `with_style` to build a new marker.
    ///
    /// # Parameters
    /// - `spec`: The spec string as described in the [class documentation](ListItemMarker)
    ///
    /// # Returns
    /// A new instance with the given `style`
    ///
    /// # Example
    /// ```rust
    ///
    /// use termlayout::widgets::{ListItemEnumerator, ListItemMarker};
    ///
    /// let marker = ListItemMarker::default()
    ///     .with_spec(" X...");
    ///
    /// assert_eq!(marker.format(1), " •...");
    /// ```
    ///
    /// # Panics
    /// If `spec` is not a valid spec-string
    #[must_use]
    pub fn with_spec(self, spec: &str) -> Self {
        self.try_with_spec(spec).unwrap()
    }

    /// Creates a default numeric marker
    /// This is a right-aligned marker that results in numeric items like "1. ", "2. ", "3. ", etc.
    ///
    /// # Returns
    /// The marker instance
    #[must_use]
    pub fn default_numbered() -> Self {
        Self::default()
            .with_enumerator(ListItemEnumerator::Decimal)
            .with_spec(">X. ")
    }

    /// Creates a default alpha marker
    /// This is a left-aligned marker that results in alphabetic items like "a) ", "b) ", "c) ", etc.
    ///
    /// # Returns
    /// The marker instance
    #[must_use]
    pub fn default_alpha() -> Self {
        Self::default()
            .with_enumerator(ListItemEnumerator::LowerAlpha)
            .with_spec("<X) ")
    }

    /// Creates a default roman marker
    /// This is a left-aligned marker that results in roman numbers like "i. ", "ii. ", "iii. ", etc.
    ///
    /// # Returns
    /// The marker instance
    #[must_use]
    pub fn default_roman() -> Self {
        Self::default()
            .with_enumerator(ListItemEnumerator::LowerRoman)
            .with_spec("<X. ")
    }

    /// Creates a default fixed marker
    /// This is a left-aligned marker that results always in the string "• "
    ///
    /// # Returns
    /// The marker instance
    #[must_use]
    pub fn default_fixed() -> Self {
        Self::default()
    }

    /// Calculate the maximum width required to display all items in the list
    ///
    /// # Parameters
    /// - `item_count`: The number of items in the list
    ///
    /// # Returns
    /// The maximum width required to display all items in the list
    ///
    /// # Example
    /// ```rust
    /// use termlayout::widgets::ListItemMarker;
    ///
    /// assert_eq!(ListItemMarker::default_numbered().max_width(1), 3);
    /// assert_eq!(ListItemMarker::default_numbered().max_width(10), 4);
    /// assert_eq!(ListItemMarker::default_numbered().max_width(100), 5);
    /// ```
    #[must_use]
    pub fn max_width(&self, item_count: usize) -> usize {
        self.prefix.len()
            + self.enumerator.max_width(self.first_index, item_count)
            + self.suffix.len()
    }

    /// Formats the marker for the given `index`
    ///
    /// # Parameters
    /// - `index`: The index of the item to format
    ///
    /// # Returns
    /// The formatted marker string
    ///
    /// # Example
    /// ```rust
    /// use termlayout::widgets::ListItemMarker;
    ///
    /// assert_eq!(ListItemMarker::default_numbered().format(0), "1. ");
    /// assert_eq!(ListItemMarker::default_alpha().format(1), "b) ");
    /// assert_eq!(ListItemMarker::default_roman().format(2), "iii. ");
    /// ```
    #[must_use]
    pub fn format(&self, index: usize) -> String {
        format!(
            "{}{}{}",
            self.prefix,
            self.enumerator.format(index + self.first_index),
            self.suffix
        )
    }
}

impl Default for ListItemMarker {
    fn default() -> Self {
        Self {
            alignment: LinesAlignment::Left,
            prefix: String::new(),
            enumerator: ListItemEnumerator::Fixed("•"),
            suffix: String::from(" "),
            first_index: 0,
        }
    }
}

/// Defines how the list items are numbered.
/// This is used from within the [`ListItemMarker`] and encapsulates the logic for generating the
/// marker ids. For example, the style `Decimal` will generate "1", "2", etc. for the items.
///
#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(unpredictable_function_pointer_comparisons)]
pub enum ListItemEnumerator {
    /// The marker will always be the same `str`.
    Fixed(&'static str),

    /// The marker will be a decimal number starting from 1.
    Decimal,

    /// The marker will be a roman number starting with from "i".
    LowerRoman,

    /// The marker will be a roman number starting with from "I".
    UpperRoman,

    /// The marker will be a sequence of lowercase letters starting from 'a'.
    LowerAlpha,

    /// The marker will be a sequence of uppercase letters starting from 'A'.
    UpperAlpha,

    /// The marker will be generated by a custom `function`.
    Custom(fn(usize) -> String),
}

impl ListItemEnumerator {
    const ROMAN_SYMBOLS: [(usize, &str); 13] = [
        (1000, "m"),
        (900, "cm"),
        (500, "d"),
        (400, "cd"),
        (100, "c"),
        (90, "xc"),
        (50, "l"),
        (40, "xl"),
        (10, "x"),
        (9, "ix"),
        (5, "v"),
        (4, "iv"),
        (1, "i"),
    ];

    /// Calculates the maximum width of the enumerator for the given `item_count`.
    ///
    /// # Parameters
    /// - `first_index`: The index of the first item to enumerate, starting from 0.
    /// - `item_count`: The number of items to enumerate.
    ///
    /// # Returns
    /// The maximum width of the enumerator for the given `max_index`.
    #[must_use]
    pub fn max_width(&self, first_index: usize, item_count: usize) -> usize {
        if item_count == 0 {
            return 0;
        }
        match self {
            ListItemEnumerator::Custom(_) => (0..item_count)
                .map(|i| self.format(i + first_index).display_len())
                .max()
                .unwrap_or(0),
            ListItemEnumerator::LowerRoman => (0..item_count)
                .map(|i| self.format(i + first_index).display_len())
                .max()
                .unwrap_or(0),
            ListItemEnumerator::UpperRoman => {
                ListItemEnumerator::UpperRoman.max_width(first_index, item_count)
            }
            _ => self.format(item_count - 1).display_len(),
        }
    }

    /// Returns the string representation of the given `index` using this style.
    /// For example, for `LowerRoman` style,  index 0 will return "i", index 1 will return "ii", etc.
    ///
    /// # Parameters
    /// - `index`: The index to format, bias is `0`
    ///
    /// # Returns
    /// The string representation of the given `index` using this style.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::widgets::ListItemEnumerator;
    ///
    /// assert_eq!(ListItemEnumerator::LowerRoman.format(0), "i");
    /// assert_eq!(ListItemEnumerator::UpperRoman.format(1), "II");
    /// assert_eq!(ListItemEnumerator::Decimal.format(2), "3");
    /// assert_eq!(ListItemEnumerator::LowerAlpha.format(3), "d");
    /// assert_eq!(ListItemEnumerator::UpperAlpha.format(4), "E");
    /// assert_eq!(ListItemEnumerator::Fixed("X").format(5), "X");
    /// ```
    #[must_use]
    pub fn format(&self, index: usize) -> String {
        match self {
            ListItemEnumerator::Fixed(f) => f.to_string(),
            ListItemEnumerator::Decimal => format!("{}", index + 1),
            ListItemEnumerator::LowerRoman => Self::format_roman(index, false),
            ListItemEnumerator::UpperRoman => Self::format_roman(index, true),
            ListItemEnumerator::LowerAlpha => Self::format_alpha(index, false),
            ListItemEnumerator::UpperAlpha => Self::format_alpha(index, true),
            ListItemEnumerator::Custom(f) => f(index),
        }
    }

    fn format_alpha(index: usize, upper: bool) -> String {
        fn append(result: &mut String, start: char, mut index: usize) {
            if index >= 26 {
                append(result, start, index / 26 - 1);
                index %= 26;
            }
            result.push((u8::try_from(start).unwrap() + u8::try_from(index).unwrap()) as char);
        }
        let start = if upper { 'A' } else { 'a' };
        let mut result = String::new();
        append(&mut result, start, index);
        result
    }

    fn format_roman(mut index: usize, upper: bool) -> String {
        let mut result = String::new();
        index += 1;
        for &(value, symbol) in &Self::ROMAN_SYMBOLS {
            while index >= value {
                result += symbol;
                index -= value;
            }
        }
        if upper {
            return result.to_uppercase();
        }
        result
    }
}

struct ListItemMakerParser;

impl ListItemMakerParser {
    fn parse(
        marker: char,
        style: ListItemEnumerator,
        spec: &str,
    ) -> Result<ListItemMarker, String> {
        if spec.is_empty() {
            return Err("empty marker spec".to_string());
        }
        let marker_pos = spec
            .find(marker)
            .ok_or_else(|| format!("marker '{marker}' not found in spec"))?;
        let (prefix, suffix) = spec.display_split_at(marker_pos);
        let (alignment, prefix) = match prefix.chars().next() {
            Some('<') => (LinesAlignment::Left, &prefix[1..]),
            Some('>') => (LinesAlignment::Right, &prefix[1..]),
            Some('|') => (LinesAlignment::Center, &prefix[1..]),
            _ => (LinesAlignment::Left, prefix),
        };

        Ok(ListItemMarker {
            alignment,
            prefix: prefix.to_string(),
            enumerator: style,
            suffix: suffix[marker.len_utf8()..].to_string(),
            first_index: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::widgets::list::ListItemMakerParser;
    use crate::widgets::{LinesAlignment, List, ListItemEnumerator, ListItemMarker, Paragraph};
    use crate::{Dimension, Layout, LayoutOptions, Rect, WrapMode};

    fn sample_list() -> List {
        List::new(
            ListItemMarker::default_roman(),
            vec![
                Paragraph::left(
                    "Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua.",
                ),
                Paragraph::left(
                    "Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua.",
                ),
                Paragraph::left(
                    "At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet.",
                ),
            ],
        )
    }
    #[test]
    fn list_min_dim() {
        let list = sample_list();
        let result = list.min_dim();
        assert_eq!(result, Dimension::new(15, 64));
    }

    #[test]
    fn list_pref_dim() {
        let list = sample_list();

        let result = list.pref_dim(100, WrapMode::Wrap);
        assert_eq!(result, Dimension::new(100, 7));
        let result = list.pref_dim(50, WrapMode::Wrap);
        assert_eq!(result, Dimension::new(50, 14));
        let result = list.pref_dim(2, WrapMode::Wrap);
        assert_eq!(result, Dimension::new(2, 447));
        let result = list.pref_dim(1, WrapMode::Wrap);
        assert_eq!(result, Dimension::new(1, 447));

        let result = list.pref_dim(100, WrapMode::default_truncate());
        assert_eq!(result, Dimension::new(100, 7));
        let result = list.pref_dim(10, WrapMode::default_truncate());
        assert_eq!(result, Dimension::new(10, 87));
    }

    #[test]
    fn list_layout() {
        let list = sample_list();

        // No clip, no fill
        let options = LayoutOptions::new(Dimension::new(50, 15), false, WrapMode::default(), None);
        let layout = list.layout_strict(options);
        let result = format!("{layout}");
        assert_eq!(
            result,
            concat!(
                "i.   Lorem ipsum dolor sit amet, consetetur\n",
                "     sadipscing elitr, sed diam nonumy eirmod\n",
                "     tempor invidunt ut labore et dolore magna\n",
                "     aliquyam erat, sed diam voluptua.\n",
                "ii.  Stet clita kasd gubergren, no sea takimata\n",
                "     sanctus est Lorem ipsum dolor sit amet. Lorem\n",
                "     ipsum dolor sit amet, consetetur sadipscing\n",
                "     elitr, sed diam nonumy eirmod tempor invidunt\n",
                "     ut labore et dolore magna aliquyam erat, sed\n",
                "     diam voluptua.\n",
                "iii. At vero eos et accusam et justo duo dolores\n",
                "     et ea rebum. Stet clita kasd gubergren, no\n",
                "     sea takimata sanctus est Lorem ipsum dolor\n",
                "     sit amet.\n",
                "\n"
            )
        );

        // No clip, with fill
        let options = LayoutOptions::new(Dimension::new(50, 15), true, WrapMode::default(), None);
        let layout = list.layout_strict(options);
        let result = format!("{layout}");
        assert_eq!(
            result,
            concat!(
                "i.   Lorem ipsum dolor sit amet, consetetur       \n",
                "     sadipscing elitr, sed diam nonumy eirmod     \n",
                "     tempor invidunt ut labore et dolore magna    \n",
                "     aliquyam erat, sed diam voluptua.            \n",
                "ii.  Stet clita kasd gubergren, no sea takimata   \n",
                "     sanctus est Lorem ipsum dolor sit amet. Lorem\n",
                "     ipsum dolor sit amet, consetetur sadipscing  \n",
                "     elitr, sed diam nonumy eirmod tempor invidunt\n",
                "     ut labore et dolore magna aliquyam erat, sed \n",
                "     diam voluptua.                               \n",
                "iii. At vero eos et accusam et justo duo dolores  \n",
                "     et ea rebum. Stet clita kasd gubergren, no   \n",
                "     sea takimata sanctus est Lorem ipsum dolor   \n",
                "     sit amet.                                    \n",
                "                                                  \n"
            )
        );

        // with clip, with fill
        let options = LayoutOptions::new(
            Dimension::new(50, 15),
            true,
            WrapMode::default(),
            Some(Rect::new(2, 5, Dimension::new(30, 8))),
        );
        let layout = list.layout_strict(options);
        let result = format!("{layout}");
        assert_eq!(
            result,
            concat!(
                "   sanctus est Lorem ipsum dol\n",
                "   ipsum dolor sit amet, conse\n",
                "   elitr, sed diam nonumy eirm\n",
                "   ut labore et dolore magna a\n",
                "   diam voluptua.             \n",
                "i. At vero eos et accusam et j\n",
                "   et ea rebum. Stet clita kas\n",
                "   sea takimata sanctus est Lo\n",
            )
        );
    }

    #[test]
    fn list_item_enumeration_format_fixed() {
        let style = ListItemEnumerator::Fixed("-");

        assert_eq!(style.format(0), "-");
        assert_eq!(style.format(4711), "-");
    }

    #[test]
    fn list_item_enumeration_format_decimal() {
        let style = ListItemEnumerator::Decimal;

        assert_eq!(style.format(0), "1");
        assert_eq!(style.format(4711), "4712");
    }

    #[test]
    fn list_item_enumeration_format_alpha() {
        let style = ListItemEnumerator::LowerAlpha;
        assert_eq!(style.format(0), "a");
        assert_eq!(style.format(25), "z");
        assert_eq!(style.format(26), "aa");
        assert_eq!(style.format(26 + 25), "az");
        assert_eq!(style.format(2 * 26), "ba");
        assert_eq!(style.format(2 * 26 + 25), "bz");
        assert_eq!(style.format(5 * 26 * 26), "dza");
        assert_eq!(style.format(5 * 26 * 26 + 25), "dzz");

        let style = ListItemEnumerator::UpperAlpha;
        assert_eq!(style.format(5 * 26 * 26), "DZA");
    }

    #[test]
    fn list_item_enumeration_format_roman() {
        let style = ListItemEnumerator::LowerRoman;
        assert_eq!(style.format(0), "i");
        assert_eq!(style.format(1), "ii");
        assert_eq!(style.format(2), "iii");
        assert_eq!(style.format(3), "iv");
        assert_eq!(style.format(4), "v");
        assert_eq!(style.format(5), "vi");
        assert_eq!(style.format(6), "vii");
        assert_eq!(style.format(7), "viii");
        assert_eq!(style.format(8), "ix");
        assert_eq!(style.format(9), "x");
        assert_eq!(style.format(10), "xi");
        assert_eq!(style.format(18), "xix");
        assert_eq!(style.format(19), "xx");
        assert_eq!(style.format(24), "xxv");
        assert_eq!(style.format(36), "xxxvii");
        assert_eq!(style.format(38), "xxxix");
        assert_eq!(style.format(48), "xlix");
        assert_eq!(style.format(49), "l");
        assert_eq!(style.format(2048), "mmxlix");

        let style = ListItemEnumerator::UpperRoman;
        assert_eq!(style.format(2048), "MMXLIX");
    }

    #[test]
    fn list_item_enumeration_format_custom() {
        let style = ListItemEnumerator::Custom(|i| format!("{:02}", 3 * i + 1));

        assert_eq!(style.format(0), "01");
        assert_eq!(style.format(1), "04");
        assert_eq!(style.format(2), "07");
    }

    #[test]
    fn list_item_marker_parser_parse_fail() {
        let style = ListItemEnumerator::LowerRoman;

        assert_eq!(
            ListItemMakerParser::parse('X', style, ""),
            Err("empty marker spec".to_string())
        );
        assert_eq!(
            ListItemMakerParser::parse('X', style, "> "),
            Err("marker 'X' not found in spec".to_string())
        );
    }

    #[test]
    fn list_item_marker_parser_parse_ok() {
        let style = ListItemEnumerator::LowerRoman;

        let marker = ListItemMakerParser::parse('X', style, "X").unwrap();
        assert_eq!(marker.alignment, LinesAlignment::Left);
        assert_eq!(marker.prefix, "");
        assert_eq!(marker.suffix, "");
        assert_eq!(marker.enumerator, style);

        let marker = ListItemMakerParser::parse('X', style, ">-X) ").unwrap();
        assert_eq!(marker.alignment, LinesAlignment::Right);
        assert_eq!(marker.prefix, "-");
        assert_eq!(marker.suffix, ") ");
        assert_eq!(marker.enumerator, style);
    }
}
