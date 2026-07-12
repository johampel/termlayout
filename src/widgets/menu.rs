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

/// Represents a menu of items with associated keys.
/// Similar to the [`List`](crate::widgets::List) widget, the `items` are organized vertically,
/// but each item has an associated key character that is displayed as a marker.
///
/// # Example
/// ```rust
/// use termlayout::Layout;
/// use termlayout::widgets::{Menu, MenuItem, Paragraph};
///
/// let menu = Menu::new(vec![
///     MenuItem::new('1', Paragraph::left("First option")),
///     MenuItem::new('2', Paragraph::left("Second option")),
///     MenuItem::new('3', Paragraph::left("Third option")),
/// ]);
///
/// assert_eq!(format!("{}", menu.layout(40)), concat!(
///     "\x1b[1m[1]\x1b[0m First option\n",
///     "\x1b[1m[2]\x1b[0m Second option\n",
///     "\x1b[1m[3]\x1b[0m Third option\n"
/// ));
/// ```
pub struct Menu {
    /// The list of menu items to be displayed
    pub items: Vec<MenuItem>,

    /// The marker configuration for menu items
    pub marker: MenuItemMarker,
}

impl Menu {
    /// Creates a new [`Menu`] layout with default marker
    ///
    /// # Parameters
    /// - `items`: The list of menu items to be displayed
    ///
    /// # Returns
    /// The `Menu`
    ///
    /// # Example
    /// ```rust
    /// use termlayout::Layout;
    /// use termlayout::widgets::{Menu, MenuItem, Paragraph};
    ///
    /// let menu = Menu::new(vec![
    ///     MenuItem::new('a', Paragraph::left("Option A")),
    ///     MenuItem::new('b', Paragraph::left("Option B")),
    /// ]);
    ///
    /// assert_eq!(format!("{}", menu.layout(20)), concat!(
    ///     "\x1b[1m[a]\x1b[0m Option A\n",
    ///     "\x1b[1m[b]\x1b[0m Option B\n"
    /// ));
    /// ```
    pub fn new<T>(items: Vec<T>) -> Self
    where
        T: Into<MenuItem>,
    {
        Self {
            items: items.into_iter().map(Into::into).collect(),
            marker: MenuItemMarker::default(),
        }
    }

    /// Creates a new [`Menu`] layout with custom marker
    ///
    /// # Parameters
    /// - `marker`: The marker configuration to use
    /// - `items`: The list of menu items to be displayed
    ///
    /// # Returns
    /// The `Menu`
    ///
    /// # Example
    /// ```rust
    /// use termlayout::Layout;
    /// use termlayout::widgets::{Menu, MenuItem, MenuItemMarker, Paragraph};
    ///
    /// let marker = MenuItemMarker::from_spec("(X) ").unwrap();
    /// let menu = Menu::with_marker(marker, vec![
    ///     MenuItem::new('a', Paragraph::left("Option A")),
    ///     MenuItem::new('b', Paragraph::left("Option B")),
    /// ]);
    ///
    /// assert_eq!(format!("{}", menu.layout(20)), concat!(
    ///     "(a) Option A\n",
    ///     "(b) Option B\n"
    /// ));
    /// ```
    pub fn with_marker<T>(marker: MenuItemMarker, items: Vec<T>) -> Self
    where
        T: Into<MenuItem>,
    {
        Self {
            items: items.into_iter().map(Into::into).collect(),
            marker,
        }
    }

    fn create_marker(&self, item: &MenuItem) -> RcLayout {
        Lines::new(
            LinesAlignment::Left,
            LinesTrimming::None,
            None,
            self.marker.format(item.key),
        )
        .into()
    }

    fn calculate_widths(&self, max_width: usize) -> (usize, usize) {
        if max_width > 0 {
            let marker_width = self.marker.width();
            let item_width = max(1, max_width.saturating_sub(marker_width));
            (marker_width, item_width)
        } else {
            (0, 0)
        }
    }
}

impl Layout for Menu {
    fn pref_dim(&self, max_width: usize, wrap_mode: WrapMode) -> Dimension {
        let (marker_width, item_width) = self.calculate_widths(max_width);
        let mut dim = self.items.iter().fold(Dimension::empty(), |acc, item| {
            acc.vertical_union(item.text.pref_dim(item_width, wrap_mode))
        });
        dim.width += marker_width;
        dim
    }

    fn min_dim(&self) -> Dimension {
        let mut dim = self.items.iter().fold(Dimension::empty(), |acc, item| {
            acc.vertical_union(item.text.min_dim())
        });
        dim.width += self.marker.width();
        dim
    }

    fn layout_strict(&'_ self, options: LayoutOptions) -> BoxedFormattedLayout<'_> {
        let (marker_width, item_width) = self.calculate_widths(options.dim.width);
        let mut row = 0;
        let content = self
            .items
            .iter()
            .map(|item| {
                let item_dim = item.text.pref_dim(item_width, options.wrap_mode);
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
                let marker = self.create_marker(item);
                row += item_dim.height;
                FormattedMenuItem::new(
                    line_options,
                    LayoutWithOptions::of(marker, marker_options).into(),
                    item.text.layout_strict(item_options),
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

rc_layout!(Menu);

/// Represents a single item in a [`Menu`].
/// A `MenuItem` has a key character and associated text content.
#[derive(Clone)]
pub struct MenuItem {
    /// The key character for this menu item
    pub key: char,

    /// The text content of the menu item
    pub text: RcLayout,
}

impl MenuItem {
    /// Creates a new [`MenuItem`].
    ///
    /// # Parameters
    /// - `key`: The key character for this menu item
    /// - `text`: The text content of the menu item
    ///
    /// # Returns
    /// A new instance
    ///
    /// # Example
    /// ```rust
    /// use termlayout::widgets::{MenuItem, Paragraph};
    ///
    /// let item = MenuItem::new('a', Paragraph::left("Option A"));
    ///
    /// assert_eq!(item.key, 'a');
    /// ```
    pub fn new<T>(key: char, text: T) -> Self
    where
        T: Into<RcLayout>,
    {
        Self {
            key,
            text: text.into(),
        }
    }
}

struct FormattedMenuItem<'fmt> {
    options: LayoutOptions,
    marker: BoxedFormattedLayout<'fmt>,
    item: BoxedFormattedLayout<'fmt>,
}

impl<'fmt> FormattedMenuItem<'fmt> {
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

impl FormattedLayout for FormattedMenuItem<'_> {
    fn options(&self) -> &LayoutOptions {
        &self.options
    }

    fn new_writer(&'_ self) -> BoxedLayoutWriter<'_> {
        Box::new(MenuItemWriter::new(
            self.marker.new_writer(),
            self.item.new_writer(),
            &self.options,
        ))
    }
}

box_formatted_layout!(FormattedMenuItem);

struct MenuItemWriter<'wrt> {
    base: BaseLayoutWriter<'wrt>,
    marker: BoxedLayoutWriter<'wrt>,
    item: BoxedLayoutWriter<'wrt>,
}

impl<'wrt> MenuItemWriter<'wrt> {
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

impl<'wrt> LayoutWriter<'wrt> for MenuItemWriter<'wrt> {
    fn options(&self) -> &'wrt LayoutOptions {
        self.base.options()
    }

    fn write_row(&mut self, w: &mut dyn Write) -> SizedLayoutResult {
        self.base.write_row(self.marker.as_mut(), w)?;
        self.base.write_row(self.item.as_mut(), w)?;
        self.base.end_row(w)
    }
}

/// Defines how to display the marker for menu items.
/// The marker consists of a prefix, the key character placeholder, and a suffix.
/// The format is specified via a spec string like "\[X] " where X is the placeholder
/// for the key character.
///
/// # Example
/// ```rust
/// use termlayout::widgets::MenuItemMarker;
///
/// let marker = MenuItemMarker::from_spec("[X] ").unwrap();
/// assert_eq!(marker.format('a'), "[a] ");
///
/// let marker = MenuItemMarker::from_spec("(X) ").unwrap();
/// assert_eq!(marker.format('1'), "(1) ");
///
/// let marker = MenuItemMarker::from_spec("X. ").unwrap();
/// assert_eq!(marker.format('a'), "a. ");
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct MenuItemMarker {
    /// The prefix before the key character
    prefix: String,

    /// The suffix after the key character
    suffix: String,
}

impl MenuItemMarker {
    /// Creates a new [`MenuItemMarker`] from a spec string.
    /// The spec string must contain exactly one 'X' character which serves as the
    /// placeholder for the menu item key.
    ///
    /// # Parameters
    /// - `spec`: The spec string (e.g., "\[X] ", "(X) ", "X. ")
    ///
    /// # Returns
    /// A new `MenuItemMarker` or an error if the spec is invalid
    ///
    /// # Errors
    /// Returns an error if:
    /// - The spec string is empty
    /// - The spec string doesn't contain exactly one 'X'
    ///
    /// # Example
    /// ```rust
    /// use termlayout::widgets::MenuItemMarker;
    ///
    /// let marker = MenuItemMarker::from_spec("[X] ").unwrap();
    /// assert_eq!(marker.format('a'), "[a] ");
    ///
    /// assert!(MenuItemMarker::from_spec("").is_err());
    /// assert!(MenuItemMarker::from_spec("XX").is_err());
    /// assert!(MenuItemMarker::from_spec("no marker").is_err());
    /// ```
    #[allow(clippy::missing_panics_doc)]
    pub fn from_spec(spec: &str) -> Result<Self, String> {
        if spec.is_empty() {
            return Err("spec string cannot be empty".to_string());
        }

        let marker_pos = spec.find('X');
        if marker_pos.is_none() {
            return Err("spec string must contain exactly one 'X' placeholder".to_string());
        }

        let marker_pos = marker_pos.unwrap();
        if spec[marker_pos + 1..].contains('X') {
            return Err("spec string must contain exactly one 'X' placeholder".to_string());
        }

        let prefix = spec[..marker_pos].to_string();
        let suffix = spec[marker_pos + 1..].to_string();

        Ok(Self { prefix, suffix })
    }

    /// Returns the width of the marker in characters.
    ///
    /// # Returns
    /// The total width of prefix + key + suffix
    ///
    /// # Example
    /// ```rust
    /// use termlayout::widgets::MenuItemMarker;
    ///
    /// let marker = MenuItemMarker::from_spec("[X] ").unwrap();
    /// assert_eq!(marker.width(), 4);
    ///
    /// let marker = MenuItemMarker::from_spec("(X) ").unwrap();
    /// assert_eq!(marker.width(), 4);
    ///
    /// let marker = MenuItemMarker::from_spec("X. ").unwrap();
    /// assert_eq!(marker.width(), 3);
    /// ```
    #[must_use]
    pub fn width(&self) -> usize {
        self.prefix.display_len() + 1 + self.suffix.display_len()
    }

    /// Formats the marker with the given key character.
    ///
    /// # Parameters
    /// - `key`: The key character to insert into the marker
    ///
    /// # Returns
    /// The formatted marker string
    ///
    /// # Example
    /// ```rust
    /// use termlayout::widgets::MenuItemMarker;
    ///
    /// let marker = MenuItemMarker::from_spec("[X] ").unwrap();
    /// assert_eq!(marker.format('a'), "[a] ");
    /// assert_eq!(marker.format('1'), "[1] ");
    /// ```
    #[must_use]
    pub fn format(&self, key: char) -> String {
        format!("{}{}{}", self.prefix, key, self.suffix)
    }
}

impl Default for MenuItemMarker {
    fn default() -> Self {
        Self {
            prefix: "\x1b[1m[".to_string(),
            suffix: "]\x1b[0m ".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::Paragraph;

    #[test]
    fn menu_item_marker_from_spec_valid() {
        let marker = MenuItemMarker::from_spec("[X] ").unwrap();
        assert_eq!(marker.prefix, "[");
        assert_eq!(marker.suffix, "] ");
        assert_eq!(marker.width(), 4);
        assert_eq!(marker.format('a'), "[a] ");

        let marker = MenuItemMarker::from_spec("(X) ").unwrap();
        assert_eq!(marker.prefix, "(");
        assert_eq!(marker.suffix, ") ");
        assert_eq!(marker.width(), 4);
        assert_eq!(marker.format('1'), "(1) ");

        let marker = MenuItemMarker::from_spec("X. ").unwrap();
        assert_eq!(marker.prefix, "");
        assert_eq!(marker.suffix, ". ");
        assert_eq!(marker.width(), 3);
        assert_eq!(marker.format('a'), "a. ");

        let marker = MenuItemMarker::from_spec("-> X: ").unwrap();
        assert_eq!(marker.prefix, "-> ");
        assert_eq!(marker.suffix, ": ");
        assert_eq!(marker.width(), 6);
        assert_eq!(marker.format('x'), "-> x: ");
    }

    #[test]
    fn menu_item_marker_from_spec_invalid() {
        assert!(MenuItemMarker::from_spec("").is_err());
        assert!(MenuItemMarker::from_spec("XX").is_err());
        assert!(MenuItemMarker::from_spec("no marker").is_err());
        assert!(MenuItemMarker::from_spec("X X").is_err());
    }

    #[test]
    fn menu_item_marker_default() {
        let marker = MenuItemMarker::default();
        assert_eq!(marker.format('a'), "\x1b[1m[a]\x1b[0m ");
        assert_eq!(marker.width(), 4);
    }

    #[test]
    fn menu_min_dim() {
        let menu = Menu::new(vec![
            MenuItem::new('1', Paragraph::left("First option")),
            MenuItem::new('2', Paragraph::left("Second option")),
            MenuItem::new('3', Paragraph::left("Third option")),
        ]);

        // Paragraphs wrap based on longest word: "Second" = 6 chars + marker 4 chars = 10 width
        // Each paragraph wraps to 2 lines, so 3 items * 2 lines = 6 height
        assert_eq!(menu.min_dim(), Dimension::new(10, 6));
    }

    #[test]
    fn menu_pref_dim() {
        let menu = Menu::new(vec![
            MenuItem::new('1', Paragraph::left("First option")),
            MenuItem::new('2', Paragraph::left("Second option")),
            MenuItem::new('3', Paragraph::left("Third option")),
        ]);

        assert_eq!(menu.pref_dim(20, WrapMode::Wrap), Dimension::new(17, 3));
        assert_eq!(menu.pref_dim(10, WrapMode::Wrap), Dimension::new(10, 6));
    }

    #[test]
    fn menu_layout() {
        let menu = Menu::new(vec![
            MenuItem::new('1', Paragraph::left("First option")),
            MenuItem::new('2', Paragraph::left("Second option")),
            MenuItem::new('3', Paragraph::left("Third option")),
        ]);

        // No Clip
        let options = LayoutOptions::new(Dimension::new(20, 5), false, WrapMode::default(), None);
        let layout = menu.layout_strict(options);
        let result = format!("{layout}");
        assert_eq!(
            result,
            concat!(
                "\x1b[1m[1]\x1b[0m First option\n",
                "\x1b[1m[2]\x1b[0m Second option\n",
                "\x1b[1m[3]\x1b[0m Third option\n",
                "\n",
                "\n"
            )
        );

        // With fill
        let options = LayoutOptions::new(Dimension::new(20, 5), true, WrapMode::default(), None);
        let layout = menu.layout_strict(options);
        let result = format!("{layout}");
        assert_eq!(
            result,
            concat!(
                "\x1b[1m[1]\x1b[0m First option    \n",
                "\x1b[1m[2]\x1b[0m Second option   \n",
                "\x1b[1m[3]\x1b[0m Third option    \n",
                "                    \n",
                "                    \n"
            )
        );
    }

    #[test]
    fn menu_layout_with_clip() {
        let menu = Menu::new(vec![
            MenuItem::new('1', Paragraph::left("First option")),
            MenuItem::new('2', Paragraph::left("Second option")),
            MenuItem::new('3', Paragraph::left("Third option")),
        ]);

        let options = LayoutOptions::new(
            Dimension::new(20, 5),
            true,
            WrapMode::default(),
            Some(Rect::new(2, 1, Dimension::new(10, 2))),
        );
        let layout = menu.layout_strict(options);
        let result = format!("{layout}");
        assert_eq!(
            result,
            concat!("\x1b[1m]\x1b[0m Second o\n", "\x1b[1m]\x1b[0m Third op\n")
        );
    }

    #[test]
    fn menu_layout_wrap() {
        let menu = Menu::new(vec![
            MenuItem::new(
                'a',
                Paragraph::left("This is a longer option that will wrap"),
            ),
            MenuItem::new('b', Paragraph::left("Short")),
        ]);

        let options = LayoutOptions::new(Dimension::new(15, 6), false, WrapMode::Wrap, None);
        let layout = menu.layout_strict(options);
        let result = format!("{layout}");
        assert_eq!(
            result,
            concat!(
                "\x1b[1m[a]\x1b[0m This is a\n",
                "    longer\n",
                "    option that\n",
                "    will wrap\n",
                "\x1b[1m[b]\x1b[0m Short\n",
                "\n"
            )
        );
    }

    #[test]
    fn menu_with_custom_marker() {
        let marker = MenuItemMarker::from_spec("(X) ").unwrap();
        let menu = Menu::with_marker(
            marker,
            vec![
                MenuItem::new('1', Paragraph::left("First option")),
                MenuItem::new('2', Paragraph::left("Second option")),
            ],
        );

        let options = LayoutOptions::new(Dimension::new(20, 3), false, WrapMode::Wrap, None);
        let layout = menu.layout_strict(options);
        let result = format!("{layout}");
        assert_eq!(
            result,
            concat!("(1) First option\n", "(2) Second option\n", "\n")
        );
    }

    #[test]
    fn menu_with_short_marker() {
        let marker = MenuItemMarker::from_spec("X. ").unwrap();
        let menu = Menu::with_marker(
            marker,
            vec![
                MenuItem::new('a', Paragraph::left("First")),
                MenuItem::new('b', Paragraph::left("Second")),
            ],
        );

        assert_eq!(menu.min_dim(), Dimension::new(9, 2));

        let options = LayoutOptions::new(Dimension::new(15, 3), false, WrapMode::Wrap, None);
        let layout = menu.layout_strict(options);
        let result = format!("{layout}");
        assert_eq!(result, concat!("a. First\n", "b. Second\n", "\n"));
    }
}
