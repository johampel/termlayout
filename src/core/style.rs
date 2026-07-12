use crate::core::str::{DisplayStr, Fragment, FragmentIter};
use std::fmt::{Display, Formatter, Write};
use std::str::Split;

/// Represents a terminal style with colors and text effects.
///
/// The different aspects of the style (effects and colors) are encoded as bits in this integer value.
/// It implements the `Display` trait to format the style as an ANSI terminal control sequence.
///
/// # Example
/// ```rust
/// use termlayout::ext::*;
///
/// let style = Style::default()
///     .with_foreground(Color::Red)
///     .with_effect(Effect::Bold);
///
/// assert_eq!(format!("{}", style), "\x1b[1;31m");
/// ```
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Style(u64);

impl Style {
    const FOREGROUND_SHIFT: usize = 8;
    const FOREGROUND_MASK: u64 = (Color::MASK as u64) << Self::FOREGROUND_SHIFT;
    const BACKGROUND_SHIFT: usize = 36;
    const BACKGROUND_MASK: u64 = (Color::MASK as u64) << Self::BACKGROUND_SHIFT;

    /// Gets the foreground color.
    ///
    /// # Returns
    /// The foreground [`Color`]
    ///
    /// # Examples
    /// ```rust
    /// use termlayout::ext::*;
    ///
    /// let style = Style::default();
    ///
    /// assert_eq!(style.foreground(), Color::Default);
    ///
    /// let style = style.with_foreground(Color::Red);
    /// assert_eq!(style.foreground(), Color::Red);
    /// ```
    #[must_use]
    pub fn foreground(self) -> Color {
        Color::decode(((self.0 & Self::FOREGROUND_MASK) >> Self::FOREGROUND_SHIFT) as u32)
    }

    /// Sets the foreground color.
    ///
    /// # Parameters
    /// - `color`: The new foreground color
    ///
    /// # Returns
    /// The new [`Style`] with the given foreground [`Color`]
    ///
    /// # Examples
    /// ```rust
    /// use termlayout::ext::*;
    ///
    /// let style = Style::default();
    ///
    /// let style = style.with_foreground(Color::Red);
    /// assert_eq!(style.foreground(), Color::Red);
    /// ```
    #[must_use]
    pub fn with_foreground(self, color: Color) -> Self {
        Style(
            (self.0 & !Self::FOREGROUND_MASK)
                | (u64::from(color.encode()) << Self::FOREGROUND_SHIFT),
        )
    }

    /// Gets the background color.
    ///
    /// # Returns
    /// The background [`Color`]
    ///
    /// # Examples
    /// ```rust
    /// use termlayout::ext::*;
    ///
    /// let style = Style::default();
    ///
    /// assert_eq!(style.background(), Color::Default);
    ///
    /// let style = style.with_background(Color::Red);
    /// assert_eq!(style.background(), Color::Red);
    /// ```
    #[must_use]
    pub fn background(self) -> Color {
        Color::decode(((self.0 & Self::BACKGROUND_MASK) >> Self::BACKGROUND_SHIFT) as u32)
    }

    /// Sets the background color.
    ///
    /// # Parameters
    /// - `color`: The new background color
    ///
    /// # Returns
    /// The new [`Style`] with the given background [`Color`]
    ///
    /// # Examples
    /// ```rust
    /// use termlayout::ext::*;
    ///
    /// let style = Style::default();
    ///
    /// let style = style.with_background(Color::Red);
    /// assert_eq!(style.background(), Color::Red);
    /// ```
    #[must_use]
    pub fn with_background(self, color: Color) -> Self {
        Style(
            (self.0 & !Self::BACKGROUND_MASK)
                | (u64::from(color.encode()) << Self::BACKGROUND_SHIFT),
        )
    }

    /// Returns a new [`Style`] without the given `effect` set.
    ///
    /// # Parameters
    /// - `effect`: The [`Effect`] to add
    ///
    /// # Returns
    /// A new style having the given effect not set
    ///
    /// # Examples
    /// ```rust
    /// use termlayout::ext::*;
    ///
    /// let bold_and_italic_style = Style::default()
    ///     .with_effect(Effect::Bold)
    ///     .with_effect(Effect::Italic);
    ///
    /// let bold_style = bold_and_italic_style.without_effect(Effect::Italic);
    /// let empty_style = bold_style.without_effect(Effect::Bold);
    ///
    /// assert_eq!(empty_style.has_effect(Effect::Bold), false);
    /// assert_eq!(empty_style.has_effect(Effect::Italic), false);
    /// assert_eq!(bold_style.has_effect(Effect::Bold), true);
    /// assert_eq!(bold_style.has_effect(Effect::Italic), false);
    /// assert_eq!(bold_and_italic_style.has_effect(Effect::Bold), true);
    /// assert_eq!(bold_and_italic_style.has_effect(Effect::Italic), true);
    /// ```
    #[must_use]
    pub fn without_effect(self, effect: Effect) -> Self {
        Style(self.0 & !(effect as u64))
    }

    /// Returns a new [`Style`] with the given `effect` set.
    ///
    /// # Parameters
    /// - `effect`: The [`Effect`] to adde
    ///
    /// # Returns
    /// A new style having the given effect set
    ///
    /// # Examples
    /// ```rust
    /// use termlayout::ext::*;
    ///
    /// let empty_style = Style::default();
    ///
    /// let bold_style = empty_style.with_effect(Effect::Bold);
    /// let bold_and_italic_style = bold_style.with_effect(Effect::Italic);
    ///
    /// assert_eq!(empty_style.has_effect(Effect::Bold), false);
    /// assert_eq!(empty_style.has_effect(Effect::Italic), false);
    /// assert_eq!(bold_style.has_effect(Effect::Bold), true);
    /// assert_eq!(bold_style.has_effect(Effect::Italic), false);
    /// assert_eq!(bold_and_italic_style.has_effect(Effect::Bold), true);
    /// assert_eq!(bold_and_italic_style.has_effect(Effect::Italic), true);
    /// ```
    #[must_use]
    pub fn with_effect(self, effect: Effect) -> Self {
        Self(self.0 | (effect as u64))
    }

    /// Checks whether the given `effect` is set.
    ///
    /// # Parameters
    /// - `effect`: The [`Effect`] to check
    ///
    /// # Returns
    /// `true`, if set
    ///
    /// # Examples
    /// ```rust
    /// use termlayout::ext::*;
    ///
    /// let style = Style::default();
    ///
    /// assert_eq!(style.has_effect(Effect::Bold), false);
    ///
    /// let style = style.with_effect(Effect::Bold);
    /// assert_eq!(style.has_effect(Effect::Bold), true);
    /// ```
    #[must_use]
    pub fn has_effect(self, effect: Effect) -> bool {
        (self.0 & (effect as u64)) == effect as u64
    }

    /// Applies all control sequences found in `text` to this [`Style`].
    ///
    /// # Parameters
    /// - `text`: The text to apply control sequences from
    ///
    /// # Returns
    /// The new `Style`
    ///
    /// # Example
    /// ```rust
    /// use termlayout::ext::*;
    ///
    /// let style = Style::default().with_foreground(Color::Red);
    ///
    /// let style = style.with_text("Hello \x1b[1mWorld\x1b[43m!");
    ///
    /// assert_eq!(
    ///     style,
    ///     Style::default()
    ///         .with_foreground(Color::Red)
    ///         .with_background(Color::Yellow)
    ///         .with_effect(Effect::Bold));
    /// ```
    #[must_use]
    pub fn with_text<'text, T>(self, text: T) -> Self
    where
        T: Into<&'text str>,
    {
        text.into()
            .display_fragments()
            .fold(self, |style, fragment| style.with_fragment(&fragment))
    }

    fn with_fragment(self, fragment: &Fragment<'_>) -> Self {
        fn to_u8(iter: &mut Split<char>) -> Option<u8> {
            iter.next().and_then(|s| s.parse().ok())
        }

        fn to_custom_color(iter: &mut Split<char>) -> Option<Color> {
            match iter.next() {
                Some("5") => Some(Color::Custom8(to_u8(iter)?)),
                Some("2") => {
                    let r = to_u8(iter)?;
                    let g = to_u8(iter)?;
                    let b = to_u8(iter)?;
                    Some(Color::Custom24(r, g, b))
                }
                _ => None,
            }
        }

        let mut new_style = self;
        let mut param_iter = match fragment {
            Fragment::ControlSequence(cs) => cs.split(';'),
            Fragment::Plain(_) => return self,
        };

        while let Some(param) = param_iter.next() {
            new_style = match param {
                "0" => Style::default(),
                "1" => new_style.with_effect(Effect::Bold),
                "2" => new_style.with_effect(Effect::Dim),
                "3" => new_style.with_effect(Effect::Italic),
                "4" => new_style.with_effect(Effect::Underline),
                "5" => new_style.with_effect(Effect::Blink),
                "7" => new_style.with_effect(Effect::Inverse),
                "8" => new_style.with_effect(Effect::Hidden),
                "9" => new_style.with_effect(Effect::Strikethrough),
                "22" => new_style
                    .without_effect(Effect::Bold)
                    .without_effect(Effect::Dim),
                "23" => new_style.without_effect(Effect::Italic),
                "24" => new_style.without_effect(Effect::Underline),
                "25" => new_style.without_effect(Effect::Blink),
                "27" => new_style.without_effect(Effect::Inverse),
                "28" => new_style.without_effect(Effect::Hidden),
                "29" => new_style.without_effect(Effect::Strikethrough),
                "30" => new_style.with_foreground(Color::Black),
                "31" => new_style.with_foreground(Color::Red),
                "32" => new_style.with_foreground(Color::Green),
                "33" => new_style.with_foreground(Color::Yellow),
                "34" => new_style.with_foreground(Color::Blue),
                "35" => new_style.with_foreground(Color::Magenta),
                "36" => new_style.with_foreground(Color::Cyan),
                "37" => new_style.with_foreground(Color::White),
                "38" => match to_custom_color(&mut param_iter) {
                    Some(color) => new_style.with_foreground(color),
                    None => return self,
                },
                "39" => new_style.with_foreground(Color::Default),
                "40" => new_style.with_background(Color::Black),
                "41" => new_style.with_background(Color::Red),
                "42" => new_style.with_background(Color::Green),
                "43" => new_style.with_background(Color::Yellow),
                "44" => new_style.with_background(Color::Blue),
                "45" => new_style.with_background(Color::Magenta),
                "46" => new_style.with_background(Color::Cyan),
                "47" => new_style.with_background(Color::White),
                "48" => match to_custom_color(&mut param_iter) {
                    Some(color) => new_style.with_background(color),
                    None => return self,
                },
                "49" => new_style.with_background(Color::Default),
                // Invalid parameter -> Ignore the entire sequence
                _ => return self,
            }
        }

        new_style
    }

    /// Creates a [`Transition`] for the given text.
    /// The `before` part of the returned instance is the same as this, the `after` part is
    /// the same as this but with the given text applied
    ///
    /// # Parameters
    /// - `text`: The [`DisplayStr`] to calculate the transition for
    ///
    /// # Returns
    /// A [`Transition`] instance with the given text applied
    ///
    /// # Example
    /// ```rust
    /// use termlayout::ext::*;
    ///
    /// let style = Style::default().with_foreground(Color::Red);
    ///
    /// let transition = style.transition_for("Hello, \x1b[1mWorld!");
    ///
    /// assert_eq!(transition.before, style);
    /// assert_eq!(transition.after, style.with_effect(Effect::Bold));
    /// ```
    pub fn transition_for<'text, T>(self, text: T) -> Transition
    where
        T: Into<&'text str>,
    {
        Transition::new(self, self.with_text(text.into()))
    }

    /// Writes self to the given writer
    ///
    /// # Parameters
    /// - `w`: The writer to write the style to
    ///
    /// # Returns
    /// The formatting result
    ///
    /// # Errors
    /// If writing to the writer fails
    pub fn render(self, w: &mut dyn Write) -> std::fmt::Result {
        Transition::from(self).render(w)
    }
}

impl<T> From<T> for Style
where
    T: Into<u64>,
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

impl Display for Style {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.render(f)
    }
}

/// Represents a color for terminal styling.
/// A color is either the [default](Color::Default), one of the predefined colors (black, red,...)
/// or one of the custom colors ([8-bit indexed](Color::Custom8) or [24-bit RGB](Color::Custom24)).
/// Using the [`encode`](Color::encode) and [`decode`](Color::decode) methods, a color can be
/// converted from/to a 28-bit integer, which can be used within a [`Style`]
///
/// `Color` also implements the [`Display`] trait, which allows it to be formatted as a string as
/// part of a control sequence.
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
#[allow(missing_docs)]
pub enum Color {
    Default = 0,
    Black = 1,
    Red = 2,
    Green = 3,
    Yellow = 4,
    Blue = 5,
    Magenta = 6,
    Cyan = 7,
    White = 8,
    Custom8(u8),
    Custom24(u8, u8, u8),
}

impl Color {
    const VALUE_SHIFT: usize = 4;
    const TYPE_MASK: u32 = 0x0000_000f;
    const MASK: u32 = 0x0fff_ffff;

    /// Encode the [`Color`] into a 28 bit integer value.
    ///
    /// # Returns
    /// The 28 bit integer value
    ///
    #[must_use]
    pub fn encode(self) -> u32 {
        match self {
            Color::Default => 0,
            Color::Black => 1,
            Color::Red => 2,
            Color::Green => 3,
            Color::Yellow => 4,
            Color::Blue => 5,
            Color::Magenta => 6,
            Color::Cyan => 7,
            Color::White => 8,
            Color::Custom8(v) => u32::from(v) << Self::VALUE_SHIFT | 9,
            Color::Custom24(r, g, b) => {
                (u32::from(r) << 16 | u32::from(g) << 8 | u32::from(b)) << Self::VALUE_SHIFT
                    | 0x000a
            }
        }
    }

    /// Decodes a 28 bit integer value into a [`Color`]
    /// If the integer does not refer to a valid color, the default color is returned
    ///
    /// # Parameter
    /// - `value`: The value to convert
    ///
    /// # Return
    /// The `Color`
    ///
    #[must_use]
    pub fn decode(value: u32) -> Color {
        match value & Self::TYPE_MASK {
            1 => Color::Black,
            2 => Color::Red,
            3 => Color::Green,
            4 => Color::Yellow,
            5 => Color::Blue,
            6 => Color::Magenta,
            7 => Color::Cyan,
            8 => Color::White,
            9 => Color::Custom8(((value >> Self::VALUE_SHIFT) & 0xff) as u8),
            10 => Color::Custom24(
                ((value >> (16 + Self::VALUE_SHIFT)) & 0xff) as u8,
                ((value >> (8 + Self::VALUE_SHIFT)) & 0xff) as u8,
                ((value >> Self::VALUE_SHIFT) & 0xff) as u8,
            ),
            _ => Color::Default,
        }
    }

    /// Writes self to the given writer
    ///
    /// # Parameters
    /// - `w`: The writer to write the style to
    ///
    /// # Returns
    /// The formatting result
    ///
    /// # Errors
    /// `std::fmt::Error`: If writing to the writer fails
    pub fn render(self, w: &mut dyn Write) -> std::fmt::Result {
        match self {
            Color::Default => w.write_char('9'),
            Color::Black => w.write_char('0'),
            Color::Red => w.write_char('1'),
            Color::Green => w.write_char('2'),
            Color::Yellow => w.write_char('3'),
            Color::Blue => w.write_char('4'),
            Color::Magenta => w.write_char('5'),
            Color::Cyan => w.write_char('6'),
            Color::White => w.write_char('7'),
            Color::Custom8(v) => w.write_fmt(format_args!("8;5;{v}")),
            Color::Custom24(r, g, b) => w.write_fmt(format_args!("8;2;{r};{g};{b}")),
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.render(f)
    }
}

/// Represents an effect for terminal styling.
/// Effects can be added or removed to [`Style`]s individually, so from a theoretical point of view
/// it is possible to add any combination of effects to a style, although some combination might
/// make no functional sense.
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
#[allow(missing_docs)]
pub enum Effect {
    None = 0x00,
    Bold = 0x01,
    Dim = 0x02,
    Italic = 0x04,
    Underline = 0x08,
    Blink = 0x10,
    Inverse = 0x20,
    Hidden = 0x40,
    Strikethrough = 0x80,
}

impl Effect {
    /// An array containing all available effects.
    pub const ALL: [Effect; 8] = [
        Effect::Bold,
        Effect::Dim,
        Effect::Italic,
        Effect::Underline,
        Effect::Blink,
        Effect::Inverse,
        Effect::Hidden,
        Effect::Strikethrough,
    ];

    /// The ANSI control sequence parameters used to set the corresponding effects.
    pub const SET_PARAM: [&'static str; 8] = ["1", "2", "3", "4", "5", "7", "8", "9"];
    /// The ANSI control sequence parameters used to reset the corresponding effects.
    pub const RESET_PARAM: [&'static str; 8] = ["22", "22", "23", "24", "25", "27", "28", "29"];
}

/// Represents the transition between two [`Style`]s.
/// The main aim of this trait is to provide a way to transition between styles in a terminal
/// by emitting terminal escape sequences. Therefore, the [`Display`] trait is the most central
/// part of this struct.
///
/// # Example
/// ```rust
/// use termlayout::ext::*;
///
/// let before = Style::default()
///     .with_foreground(Color::Red);
/// let after = Style::default()
///     .with_effect(Effect::Bold);
///
/// let transition = Transition::new(before, after);
///
/// assert_eq!(format!("{}", transition), "\x1b[1;39m");
/// ```
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Transition {
    /// The [`Style`] before the transition
    pub before: Style,

    /// The [`Style`] after the transition
    pub after: Style,
}

impl Transition {
    /// Creates a new `Transition` instance with the specified `before` and `after` styles.
    ///
    /// # Parameters
    /// - `before`: The `Style` representing the starting state of the transition.
    /// - `after`: The `Style` representing the ending state of the transition.
    ///
    /// # Returns
    /// A new `Transition` instance initialized with the given `before` and `after` styles.
    #[must_use]
    pub fn new(before: Style, after: Style) -> Self {
        Transition { before, after }
    }

    /// Inverts the current `Transition` by swapping the `before` and `after` states.
    ///
    /// # Returns
    /// A new `Transition` instance where the `before` state becomes the `after` state
    /// and the `after` state becomes the `before` state.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::ext::*;
    ///
    /// let before = Style::default().with_effect(Effect::Bold);
    /// let after = Style::default().without_effect(Effect::Hidden);
    /// let transition = Transition::new(before, after);
    ///
    /// let inverted = transition.invert();
    ///
    /// assert_eq!(inverted.before, after);
    /// assert_eq!(inverted.after, before);
    /// ```
    #[must_use]
    pub fn invert(&self) -> Transition {
        Transition::new(self.after, self.before)
    }

    /// Checks if the current state is empty by comparing the `before` and `after` attributes.
    ///
    /// # Returns
    /// * `true` - if the `before` and `after` attributes are equal.
    /// * `false` - if the `before` and `after` attributes are not equal.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.before == self.after
    }

    /// Writes self to the given writer
    ///
    /// # Parameters
    /// - `w`: The writer to write the style to
    ///
    /// # Returns
    /// The formatting result
    ///
    /// # Errors
    /// Returns a `std::fmt::Error` if writing to the writer fails.
    pub fn render(&self, w: &mut dyn Write) -> std::fmt::Result {
        fn append_param(w: &mut dyn Write, param: &str, first: &mut bool) -> std::fmt::Result {
            if *first {
                *first = false;
            } else {
                w.write_char(';')?;
            }
            w.write_str(param)
        }

        // If no transition is needed, do nothing
        if self.is_empty() {
            return Ok(());
        }

        w.write_str(FragmentIter::CSI_START)?;

        // If going back to default style, emit a global reset sequence
        if self.after == Style::default() {
            w.write_char('0')?;
            return w.write_char(FragmentIter::CSI_END);
        }

        let mut first_param = true;

        // Handle effects
        for (index, &effect) in Effect::ALL.iter().enumerate() {
            let before = self.before.has_effect(effect);
            let after = self.after.has_effect(effect);
            if before == after {
                continue;
            }
            if after {
                append_param(w, Effect::SET_PARAM[index], &mut first_param)?;
            } else {
                append_param(w, Effect::RESET_PARAM[index], &mut first_param)?;
            }
        }

        // Handle foreground color
        if self.before.foreground() != self.after.foreground() {
            append_param(w, "3", &mut first_param)?;
            self.after.foreground().render(w)?;
        }

        // Handle background color
        if self.before.background() != self.after.background() {
            append_param(w, "4", &mut first_param)?;
            self.after.background().render(w)?;
        }

        w.write_char(FragmentIter::CSI_END)
    }
}

impl Display for Transition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.render(f)
    }
}

impl<T> From<T> for Transition
where
    T: Into<Style>,
{
    fn from(style: T) -> Self {
        Transition::new(Style::default(), style.into())
    }
}

impl Default for Transition {
    fn default() -> Self {
        Self::from(Style::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn style_with_fragments_effects() {
        let mut style = Style::default();

        // Set effects
        style = style.with_fragment(&Fragment::ControlSequence("1"));
        assert_eq!(style, Style::from(1u64));
        style = style.with_fragment(&Fragment::ControlSequence("2"));
        assert_eq!(style, Style::from(3u64));
        style = style.with_fragment(&Fragment::ControlSequence("3"));
        assert_eq!(style, Style::from(7u64));
        style = style.with_fragment(&Fragment::ControlSequence("4"));
        assert_eq!(style, Style::from(15u64));
        style = style.with_fragment(&Fragment::ControlSequence("5"));
        assert_eq!(style, Style::from(31u64));
        style = style.with_fragment(&Fragment::ControlSequence("7"));
        assert_eq!(style, Style::from(63u64));
        style = style.with_fragment(&Fragment::ControlSequence("8"));
        assert_eq!(style, Style::from(127u64));
        style = style.with_fragment(&Fragment::ControlSequence("9"));
        assert_eq!(style, Style::from(255u64));

        // Set effects
        style = style.with_fragment(&Fragment::ControlSequence("22"));
        assert_eq!(style, Style::from(252u64));
        style = style.with_fragment(&Fragment::ControlSequence("23"));
        assert_eq!(style, Style::from(248u64));
        style = style.with_fragment(&Fragment::ControlSequence("24"));
        assert_eq!(style, Style::from(240u64));
        style = style.with_fragment(&Fragment::ControlSequence("25"));
        assert_eq!(style, Style::from(224u64));
        style = style.with_fragment(&Fragment::ControlSequence("27"));
        assert_eq!(style, Style::from(192u64));
        style = style.with_fragment(&Fragment::ControlSequence("28"));
        assert_eq!(style, Style::from(128u64));
        style = style.with_fragment(&Fragment::ControlSequence("29"));
        assert_eq!(style, Style::from(0u64));
    }

    #[test]
    fn style_with_fragments_foreground() {
        let mut style = Style::default();

        style = style.with_fragment(&Fragment::ControlSequence("31"));
        assert_eq!(style, Style::default().with_foreground(Color::Red));

        style = style.with_fragment(&Fragment::ControlSequence("38;5;17"));
        assert_eq!(style, Style::default().with_foreground(Color::Custom8(17)));

        style = style.with_fragment(&Fragment::ControlSequence("38;2;19;23;29"));
        assert_eq!(
            style,
            Style::default().with_foreground(Color::Custom24(19, 23, 29))
        );

        style = style.with_fragment(&Fragment::ControlSequence("39"));
        assert_eq!(style, Style::default());
    }

    #[test]
    fn style_with_fragments_background() {
        let mut style = Style::default();

        style = style.with_fragment(&Fragment::ControlSequence("41"));
        assert_eq!(style, Style::default().with_background(Color::Red));

        style = style.with_fragment(&Fragment::ControlSequence("48;5;17"));
        assert_eq!(style, Style::default().with_background(Color::Custom8(17)));

        style = style.with_fragment(&Fragment::ControlSequence("48;2;19;23;29"));
        assert_eq!(
            style,
            Style::default().with_background(Color::Custom24(19, 23, 29))
        );

        style = style.with_fragment(&Fragment::ControlSequence("49"));
        assert_eq!(style, Style::default());
    }

    #[test]
    fn color_encode_and_decode() {
        assert_eq!(Color::decode(Color::Default.encode()), Color::Default);
        assert_eq!(Color::decode(Color::Black.encode()), Color::Black);
        assert_eq!(Color::decode(Color::Red.encode()), Color::Red);
        assert_eq!(Color::decode(Color::Green.encode()), Color::Green);
        assert_eq!(Color::decode(Color::Yellow.encode()), Color::Yellow);
        assert_eq!(Color::decode(Color::Blue.encode()), Color::Blue);
        assert_eq!(Color::decode(Color::Magenta.encode()), Color::Magenta);
        assert_eq!(Color::decode(Color::Cyan.encode()), Color::Cyan);
        assert_eq!(Color::decode(Color::White.encode()), Color::White);
        assert_eq!(
            Color::decode(Color::Custom8(17).encode()),
            Color::Custom8(17)
        );
        assert_eq!(
            Color::decode(Color::Custom24(19, 23, 29).encode()),
            Color::Custom24(19, 23, 29)
        );
    }

    #[test]
    fn color_fmt() {
        assert_eq!(format!("{}", Color::Default), "9");
        assert_eq!(format!("{}", Color::Black), "0");
        assert_eq!(format!("{}", Color::Red), "1");
        assert_eq!(format!("{}", Color::Green), "2");
        assert_eq!(format!("{}", Color::Yellow), "3");
        assert_eq!(format!("{}", Color::Blue), "4");
        assert_eq!(format!("{}", Color::Cyan), "6");
        assert_eq!(format!("{}", Color::White), "7");
        assert_eq!(format!("{}", Color::Custom8(17)), "8;5;17");
        assert_eq!(format!("{}", Color::Custom24(19, 23, 29)), "8;2;19;23;29");
    }

    #[test]
    fn transition_fmt() {
        let before = Style::default();
        let after = Style::default();

        assert_eq!(format!("{}", Transition::new(before, after)), "");
        assert_eq!(format!("{}", Transition::new(after, before)), "");

        let before = Style::default().with_effect(Effect::Bold);
        let after = Style::default().with_effect(Effect::Dim);
        assert_eq!(format!("{}", Transition::new(before, after)), "\x1b[22;2m");
        assert_eq!(format!("{}", Transition::new(after, before)), "\x1b[1;22m");

        let before = Style::default().with_effect(Effect::Italic);
        let after = Style::default().with_effect(Effect::Underline);
        assert_eq!(format!("{}", Transition::new(before, after)), "\x1b[23;4m");
        assert_eq!(format!("{}", Transition::new(after, before)), "\x1b[3;24m");

        let before = Style::default().with_effect(Effect::Blink);
        let after = Style::default().with_effect(Effect::Inverse);
        assert_eq!(format!("{}", Transition::new(before, after)), "\x1b[25;7m");
        assert_eq!(format!("{}", Transition::new(after, before)), "\x1b[5;27m");

        let before = Style::default().with_effect(Effect::Hidden);
        let after = Style::default().with_effect(Effect::Strikethrough);
        assert_eq!(format!("{}", Transition::new(before, after)), "\x1b[28;9m");
        assert_eq!(format!("{}", Transition::new(after, before)), "\x1b[8;29m");

        let before = Style::default()
            .with_effect(Effect::Hidden)
            .with_foreground(Color::Red);
        let after = Style::default().with_effect(Effect::Hidden);
        assert_eq!(format!("{}", Transition::new(before, after)), "\x1b[39m");
        assert_eq!(format!("{}", Transition::new(after, before)), "\x1b[31m");

        let before = Style::default()
            .with_effect(Effect::Hidden)
            .with_background(Color::Red);
        let after = Style::default().with_effect(Effect::Hidden);
        assert_eq!(format!("{}", Transition::new(before, after)), "\x1b[49m");
        assert_eq!(format!("{}", Transition::new(after, before)), "\x1b[41m");
    }
}
