use crate::ext::{Color, DisplayStr, Effect, Style, Transition};
use std::mem;
use std::ops::AddAssign;

/// A builder for building text with styles.
/// A `TextBuilder` allows the creation of styled text by [`appending`](TextBuilder::append) text
/// fragments and [pushing](TextBuilder::push_style) and [popping](TextBuilder::pop_style) [`Style`]
/// information to style the text.
///
/// # Example
/// ```rust
/// use termlayout::ext::TextBuilder;
///
/// let mut builder = TextBuilder::new();
///
/// builder
///     .append("A ")
///     .blue()
///     .append("styled ")
///     .bold()
///     .append("text")
///     .pop_style()
///     .pop_style();
///
/// assert_eq!(builder.as_ref(), "A \x1b[34mstyled \x1b[1mtext\x1b[22m\x1b[0m");
/// ```
pub struct TextBuilder {
    buffer: String,
    style_stack: Vec<Style>,
}

impl Default for TextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TextBuilder {
    /// Creates a new empty [`TextBuilder`].
    ///
    /// # Returns
    /// A new empty [`TextBuilder`].
    ///
    /// # Example
    /// ```rust
    /// use termlayout::ext::{Style, TextBuilder};
    ///
    /// let builder = TextBuilder::new();
    ///
    /// assert_eq!(builder.as_ref(), "");
    /// assert_eq!(builder.current_style(), Style::default());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            style_stack: Vec::new(),
        }
    }

    /// Returns the currently active [`Style`].
    /// The style is the last style that was pushed onto the stack.
    ///
    /// # Returns
    /// The currently active [`Style`].
    ///
    /// # Example
    /// ```rust
    /// use termlayout::ext::{Color, Style, TextBuilder};
    /// let mut builder = TextBuilder::new();
    ///
    /// assert_eq!(builder.current_style(), Style::default());
    ///
    /// builder.red();
    /// assert_eq!(builder.current_style(), Style::default().with_foreground(Color::Red));
    ///
    /// builder.pop_style();
    /// assert_eq!(builder.current_style(), Style::default());
    /// ```
    #[must_use]
    pub fn current_style(&self) -> Style {
        *self.style_stack.last().unwrap_or(&Style::default())
    }

    /// Pushes the `new_style` onto the style stack.
    /// It also emits the characters necessary to transition from the current style to the new style
    /// into the buffer.
    ///
    /// The counterpart is [`pop_style`](TextBuilder::pop_style), which pops from the style stack.
    ///
    /// # Parameters
    /// - `new_style`: The new [`Style`] to push onto the style stack.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::ext::{Effect, Style, TextBuilder};
    ///
    /// let mut builder = TextBuilder::new();
    /// let style = Style::default().with_effect(Effect::Bold);
    /// builder.push_style(style);
    ///
    /// assert_eq!(builder.as_ref(), "\x1b[1m");
    /// assert_eq!(builder.current_style(), style)
    /// ```
    ///
    /// # Panics
    /// Panics if writing the style transition to the internal buffer fails.
    pub fn push_style(&mut self, new_style: Style) -> &mut Self {
        let current_style = self.current_style();
        let transition = Transition::new(current_style, new_style);
        transition.render(&mut self.buffer).unwrap();
        self.style_stack.push(new_style);
        self
    }

    /// Pushes the `new_style` onto the style stack while consuming the builder.
    /// It also emits the characters necessary to transition from the current style to the new style
    /// into the buffer.
    ///
    /// This is the consuming version of [`push_style`](TextBuilder::push_style), which allows
    /// chaining of method calls.
    ///
    /// # Parameters
    /// - `new_style`: The new [`Style`] to push onto the style stack.
    ///
    /// # Returns
    /// The new [`TextBuilder`]
    ///
    /// # Example
    /// ```rust
    /// use termlayout::ext::{Effect, Style, TextBuilder};
    ///
    /// let style = Style::default().with_effect(Effect::Bold);
    /// let builder = TextBuilder::new().with_style(style);
    ///
    /// assert_eq!(builder.as_ref(), "\x1b[1m");
    /// assert_eq!(builder.current_style(), style)
    /// ```
    ///
    /// # Panics
    /// Panics if writing the style transition to the internal buffer fails.
    #[must_use]
    pub fn with_style(mut self, new_style: Style) -> Self {
        self.push_style(new_style);
        self
    }

    /// Pushes a style change on the style stack.
    /// This executes `change` using the [current style](TextBuilder::current_style) and pushes the
    /// resulting style onto the stack.
    ///
    /// # Parameters
    /// - `change`: The function to change the current style
    ///
    /// # Example
    /// ```rust
    /// use termlayout::ext::{Effect, Style, TextBuilder};
    ///
    /// let mut builder = TextBuilder::new();
    /// builder
    ///     .bold()
    ///     .push_style_change(|s| s.with_effect(Effect::Italic));
    /// assert_eq!(builder.as_ref(), "\x1b[1m\x1b[3m");
    /// assert_eq!(builder.current_style(),
    ///     Style::default()
    ///         .with_effect(Effect::Italic)
    ///         .with_effect(Effect::Bold));
    /// ```
    pub fn push_style_change<T>(&mut self, change: T) -> &mut Self
    where
        T: FnOnce(Style) -> Style,
    {
        let current_style = self.current_style();
        let new_style = change(current_style);
        self.push_style(new_style)
    }

    /// Pushes a style change on the style stack with consuming the builder.
    /// This executes `change` using the [current style](TextBuilder::current_style) and pushes the
    /// resulting style onto the stack.
    ///
    /// This is the consuming version of [`push_style`](TextBuilder::push_style), which allows
    /// chaining of method calls.
    ///
    /// # Parameters
    /// - `change`: The function to change the current style
    ///
    /// # Returns
    /// The new [`TextBuilder`]
    ///
    /// # Example
    /// ```rust
    /// use termlayout::ext::{Effect, Style, TextBuilder};
    ///
    /// let builder = TextBuilder::new()
    ///     .with_bold();
    ///
    /// let builder = builder.with_style_change(|s| s.with_effect(Effect::Italic));
    /// assert_eq!(builder.as_ref(), "\x1b[1m\x1b[3m");
    /// assert_eq!(builder.current_style(),
    ///     Style::default()
    ///         .with_effect(Effect::Italic)
    ///         .with_effect(Effect::Bold));
    /// ```
    #[must_use]
    pub fn with_style_change<T>(mut self, change: T) -> Self
    where
        T: FnOnce(Style) -> Style,
    {
        self.push_style_change(change);
        self
    }

    /// Pushes a foreground `color` style change onto the style stack.
    ///
    /// # Parameters
    /// - `color`: The new foreground [`Color`].
    ///
    /// # Example
    /// ```rust
    /// use termlayout::ext::{Color, TextBuilder};
    ///
    /// let mut builder = TextBuilder::new();
    /// builder.push_foreground_color(Color::Red);
    ///
    /// assert_eq!(builder.as_ref(), "\x1b[31m");
    /// ```
    pub fn push_foreground_color(&mut self, color: Color) -> &mut Self {
        self.push_style_change(|s| s.with_foreground(color))
    }

    /// Pushes a foreground `color` style change while consuming the builder.
    ///
    /// # Parameters
    /// - `color`: The new foreground [`Color`].
    ///
    /// # Returns
    /// The updated [`TextBuilder`].
    ///
    /// # Example
    /// ```rust
    /// use termlayout::ext::{Color, TextBuilder};
    ///
    /// let builder = TextBuilder::new().with_foreground_color(Color::Blue);
    ///
    /// assert_eq!(builder.as_ref(), "\x1b[34m");
    /// ```
    #[must_use]
    pub fn with_foreground_color(mut self, color: Color) -> Self {
        self.push_foreground_color(color);
        self
    }

    /// Pushes a background `color` style change onto the style stack.
    ///
    /// # Parameters
    /// - `color`: The new background [`Color`].
    ///
    /// # Example
    /// ```rust
    /// use termlayout::ext::{Color, TextBuilder};
    ///
    /// let mut builder = TextBuilder::new();
    /// builder.push_background_color(Color::Yellow);
    ///
    /// assert_eq!(builder.as_ref(), "\x1b[43m");
    /// ```
    pub fn push_background_color(&mut self, color: Color) -> &mut Self {
        self.push_style_change(|s| s.with_background(color))
    }

    /// Pushes a background `color` style change while consuming the builder.
    ///
    /// # Parameters
    /// - `color`: The new background [`Color`].
    ///
    /// # Returns
    /// The updated [`TextBuilder`].
    ///
    /// # Example
    /// ```rust
    /// use termlayout::ext::{Color, TextBuilder};
    ///
    /// let builder = TextBuilder::new().with_background_color(Color::Magenta);
    ///
    /// assert_eq!(builder.as_ref(), "\x1b[45m");
    /// ```
    #[must_use]
    pub fn with_background_color(mut self, color: Color) -> Self {
        self.push_background_color(color);
        self
    }

    /// Pushes an `effect` style change onto the style stack.
    ///
    /// # Parameters
    /// - `effect`: The new [`Effect`].
    ///
    /// # Example
    /// ```rust
    /// use termlayout::ext::{Effect, TextBuilder};
    ///
    /// let mut builder = TextBuilder::new();
    /// builder.push_effect(Effect::Bold);
    ///
    /// assert_eq!(builder.as_ref(), "\x1b[1m");
    /// ```
    pub fn push_effect(&mut self, effect: Effect) -> &mut Self {
        self.push_style_change(|s| s.with_effect(effect))
    }

    /// Pushes an `effect` style change while consuming the builder.
    ///
    /// # Parameters
    /// - `effect`: The new [`Effect`].
    ///
    /// # Returns
    /// The updated [`TextBuilder`].
    ///
    /// # Example
    /// ```rust
    /// use termlayout::ext::{Effect, TextBuilder};
    ///
    /// let builder = TextBuilder::new().with_effect(Effect::Underline);
    ///
    /// assert_eq!(builder.as_ref(), "\x1b[4m");
    /// ```
    #[must_use]
    pub fn with_effect(mut self, effect: Effect) -> Self {
        self.push_effect(effect);
        self
    }

    /// Pushes default foreground color.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::ext::{Color, TextBuilder};
    ///
    /// let mut builder = TextBuilder::new();
    /// builder.push_foreground_color(Color::Red);
    /// builder.default_foreground();
    ///
    /// assert_eq!(builder.as_ref(), "\x1b[31m\x1b[0m");
    /// ```
    pub fn default_foreground(&mut self) -> &mut Self {
        self.push_foreground_color(Color::Default)
    }

    /// Pushes black foreground color.
    pub fn black(&mut self) -> &mut Self {
        self.push_foreground_color(Color::Black)
    }

    /// Pushes red foreground color.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::ext::TextBuilder;
    ///
    /// let mut builder = TextBuilder::new();
    /// builder.red();
    ///
    /// assert_eq!(builder.as_ref(), "\x1b[31m");
    /// ```
    pub fn red(&mut self) -> &mut Self {
        self.push_foreground_color(Color::Red)
    }

    /// Pushes green foreground color.
    pub fn green(&mut self) -> &mut Self {
        self.push_foreground_color(Color::Green)
    }

    /// Pushes yellow foreground color.
    pub fn yellow(&mut self) -> &mut Self {
        self.push_foreground_color(Color::Yellow)
    }

    /// Pushes blue foreground color.
    pub fn blue(&mut self) -> &mut Self {
        self.push_foreground_color(Color::Blue)
    }

    /// Pushes magenta foreground color.
    pub fn magenta(&mut self) -> &mut Self {
        self.push_foreground_color(Color::Magenta)
    }

    /// Pushes cyan foreground color.
    pub fn cyan(&mut self) -> &mut Self {
        self.push_foreground_color(Color::Cyan)
    }

    /// Pushes white foreground color.
    pub fn white(&mut self) -> &mut Self {
        self.push_foreground_color(Color::White)
    }

    /// Pushes 8-bit foreground color.
    pub fn foreground_custom8(&mut self, value: u8) -> &mut Self {
        self.push_foreground_color(Color::Custom8(value))
    }

    /// Pushes 24-bit foreground color.
    pub fn foreground_rgb(&mut self, r: u8, g: u8, b: u8) -> &mut Self {
        self.push_foreground_color(Color::Custom24(r, g, b))
    }

    /// Pushes bold effect.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::ext::TextBuilder;
    ///
    /// let mut builder = TextBuilder::new();
    /// builder.bold();
    ///
    /// assert_eq!(builder.as_ref(), "\x1b[1m");
    /// ```
    pub fn bold(&mut self) -> &mut Self {
        self.push_effect(Effect::Bold)
    }

    /// Pushes dim effect.
    pub fn dim(&mut self) -> &mut Self {
        self.push_effect(Effect::Dim)
    }

    /// Pushes italic effect.
    pub fn italic(&mut self) -> &mut Self {
        self.push_effect(Effect::Italic)
    }

    /// Pushes underline effect.
    pub fn underline(&mut self) -> &mut Self {
        self.push_effect(Effect::Underline)
    }

    /// Pushes blink effect.
    pub fn blink(&mut self) -> &mut Self {
        self.push_effect(Effect::Blink)
    }

    /// Pushes inverse effect.
    pub fn inverse(&mut self) -> &mut Self {
        self.push_effect(Effect::Inverse)
    }

    /// Pushes hidden effect.
    pub fn hidden(&mut self) -> &mut Self {
        self.push_effect(Effect::Hidden)
    }

    /// Pushes strikethrough effect.
    pub fn strikethrough(&mut self) -> &mut Self {
        self.push_effect(Effect::Strikethrough)
    }

    /// Pushes default foreground color while consuming the builder.
    #[must_use]
    pub fn with_default_foreground(mut self) -> Self {
        self.default_foreground();
        self
    }

    /// Pushes black foreground color while consuming the builder.
    #[must_use]
    pub fn with_black(mut self) -> Self {
        self.black();
        self
    }

    /// Pushes red foreground color while consuming the builder.
    #[must_use]
    pub fn with_red(mut self) -> Self {
        self.red();
        self
    }

    /// Pushes green foreground color while consuming the builder.
    #[must_use]
    pub fn with_green(mut self) -> Self {
        self.green();
        self
    }

    /// Pushes yellow foreground color while consuming the builder.
    #[must_use]
    pub fn with_yellow(mut self) -> Self {
        self.yellow();
        self
    }

    /// Pushes blue foreground color while consuming the builder.
    #[must_use]
    pub fn with_blue(mut self) -> Self {
        self.blue();
        self
    }

    /// Pushes magenta foreground color while consuming the builder.
    #[must_use]
    pub fn with_magenta(mut self) -> Self {
        self.magenta();
        self
    }

    /// Pushes cyan foreground color while consuming the builder.
    #[must_use]
    pub fn with_cyan(mut self) -> Self {
        self.cyan();
        self
    }

    /// Pushes white foreground color while consuming the builder.
    #[must_use]
    pub fn with_white(mut self) -> Self {
        self.white();
        self
    }

    /// Pushes 8-bit foreground color while consuming the builder.
    #[must_use]
    pub fn with_foreground_custom8(mut self, value: u8) -> Self {
        self.foreground_custom8(value);
        self
    }

    /// Pushes 24-bit foreground color while consuming the builder.
    #[must_use]
    pub fn with_foreground_rgb(mut self, r: u8, g: u8, b: u8) -> Self {
        self.foreground_rgb(r, g, b);
        self
    }

    /// Pushes bold effect while consuming the builder.
    #[must_use]
    pub fn with_bold(mut self) -> Self {
        self.bold();
        self
    }

    /// Pushes dim effect while consuming the builder.
    #[must_use]
    pub fn with_dim(mut self) -> Self {
        self.dim();
        self
    }

    /// Pushes italic effect while consuming the builder.
    #[must_use]
    pub fn with_italic(mut self) -> Self {
        self.italic();
        self
    }

    /// Pushes underline effect while consuming the builder.
    #[must_use]
    pub fn with_underline(mut self) -> Self {
        self.underline();
        self
    }

    /// Pushes blink effect while consuming the builder.
    #[must_use]
    pub fn with_blink(mut self) -> Self {
        self.blink();
        self
    }

    /// Pushes inverse effect while consuming the builder.
    #[must_use]
    pub fn with_inverse(mut self) -> Self {
        self.inverse();
        self
    }

    /// Pushes hidden effect while consuming the builder.
    #[must_use]
    pub fn with_hidden(mut self) -> Self {
        self.hidden();
        self
    }

    /// Pushes strikethrough effect while consuming the builder.
    #[must_use]
    pub fn with_strikethrough(mut self) -> Self {
        self.strikethrough();
        self
    }

    /// Pops the current style from the style stack.
    /// It also emits the characters necessary to transition from the current style to the new style
    /// into the buffer.
    ///
    /// If the stack is empty, the method has no effect
    ///
    /// # Example
    /// ```rust
    /// use termlayout::ext::{Effect, Style, TextBuilder};
    ///
    /// let mut builder = TextBuilder::new();
    ///
    /// builder.bold().italic().pop_style();
    /// assert_eq!(builder.as_ref(), "\x1b[1m\x1b[3m\x1b[23m");
    /// assert_eq!(builder.current_style(),
    ///     Style::default()
    ///         .with_effect(Effect::Bold));
    /// ```
    ///
    /// # Panics
    /// Panics if writing the style transition to the internal buffer fails.
    pub fn pop_style(&mut self) -> &mut Self {
        let style = self.style_stack.pop().unwrap_or_default();
        let transition = Transition::new(style, self.current_style());
        if !transition.is_empty() {
            transition.render(&mut self.buffer).unwrap();
        }
        self
    }

    /// Pops the current style from the style stack with consuming the builder.
    /// It also emits the characters necessary to transition from the current style to the new style
    /// into the buffer.
    ///
    /// If the stack is empty, the method has no effect
    ///
    /// This is the consuming version of [`pop_style`](TextBuilder::pop_style), which allows
    /// chaining of method calls.
    ///
    /// # Returns
    /// The new [`TextBuilder`]
    ///
    /// # Example
    /// ```rust
    /// use termlayout::ext::{Effect, Style, TextBuilder};
    ///
    /// let builder = TextBuilder::new()
    ///     .with_bold()
    ///     .with_italic();
    ///
    /// let builder = builder.without_style();
    /// assert_eq!(builder.as_ref(), "\x1b[1m\x1b[3m\x1b[23m");
    /// assert_eq!(builder.current_style(),
    ///     Style::default()
    ///         .with_effect(Effect::Bold));
    /// ```
    ///
    /// # Panics
    /// Panics if writing the style transition to the internal buffer fails.
    #[must_use]
    pub fn without_style(mut self) -> Self {
        self.pop_style();
        self
    }

    /// Pops the current style from the style stack while consuming the builder.
    ///
    /// Deprecated alias of [`without_style`](TextBuilder::without_style).
    #[must_use]
    #[deprecated(since = "0.1.0", note = "Use `without_style` instead")]
    pub fn without_last_style(self) -> Self {
        self.without_style()
    }

    /// Appends `text` to this.
    /// Note that it does not change the style stack, even if the text contains style changes.
    ///
    /// # Parameters
    /// - `text`: The text to append.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::ext::{Style, TextBuilder};
    ///
    /// let mut builder = TextBuilder::new();
    /// builder.append("\x1b[1mHello");
    ///
    /// assert_eq!(builder.as_ref(), "\x1b[1mHello");
    /// assert_eq!(builder.current_style(), Style::default());
    /// ```
    pub fn append<T>(&mut self, text: T) -> &mut Self
    where
        T: AsRef<str>,
    {
        self.buffer.push_str(text.as_ref());
        self
    }

    /// Appends `text` to the builder with consuming the builder.
    /// Note that it does not change the style stack, even if the text contains style changes.
    ///
    /// This is the consuming version of [`append`](TextBuilder::append), which allows
    /// chaining of method calls.
    ///
    /// # Parameters
    /// - `text`: The text to append.
    ///
    /// # Returns
    /// The new [`TextBuilder`]
    ///
    /// # Example
    /// ```rust
    /// use termlayout::ext::{Style, TextBuilder};
    ///
    /// let builder = TextBuilder::new()
    ///  .with_text("\x1b[1mHello");
    ///
    /// assert_eq!(builder.as_ref(), "\x1b[1mHello");
    /// assert_eq!(builder.current_style(), Style::default());
    /// ```
    #[must_use]
    pub fn with_text<T>(mut self, text: T) -> Self
    where
        T: AsRef<str>,
    {
        self.append(text);
        self
    }

    /// Resets this instance by clearing the buffer and style stack.
    pub fn reset(&mut self) {
        self.style_stack.clear();
        self.buffer.clear();
    }

    /// Resets this instance by clearing the buffer and style stack.
    /// In opposite to `reset`, this method returns the buffer as a `String`.
    ///
    /// # Returns
    /// The buffer as a `String`.
    ///
    /// # Example
    /// ```rust
    ///
    /// use termlayout::ext::{Style, TextBuilder};
    ///
    /// let mut builder = TextBuilder::new();
    ///
    /// builder.bold().append("Hallo");
    ///
    /// let result = builder.flush();
    /// assert_eq!(result, "\x1b[1mHallo");
    /// assert_eq!(builder.current_style(), Style::default());
    /// assert_eq!(builder.as_ref(), "");
    /// ```
    pub fn flush(&mut self) -> String {
        self.style_stack.clear();
        mem::take(&mut self.buffer)
    }

    /// Partially flushes the builder by returning and flushing the text but keeping the style stack.
    ///
    /// # Returns
    /// The buffer as a `String`.
    ///
    /// # Example
    /// ```rust
    ///
    /// use termlayout::ext::{Effect, Style, TextBuilder};
    ///
    /// let mut builder = TextBuilder::new();
    ///
    /// builder.bold().append("Hallo");
    ///
    /// let result = builder.partial_flush();
    /// assert_eq!(result, "\x1b[1mHallo");
    /// assert_eq!(builder.current_style(), Style::default().with_effect(Effect::Bold));
    /// assert_eq!(builder.as_ref(), "\x1b[1m");
    /// ```
    ///
    /// # Panics
    /// Panics if writing the current style to the internal buffer fails.
    pub fn partial_flush(&mut self) -> String {
        let result = mem::take(&mut self.buffer);
        let current_style = self.current_style();
        current_style.render(&mut self.buffer).unwrap();
        result
    }

    /// Returns true if the buffer is semantically empty.
    /// It is empty if it contains at least one display character (so no control sequences).
    ///
    /// # Returns
    /// `true` if the buffer is empty, `false` otherwise.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.buffer.display_len() == 0
    }
}

impl<T> AddAssign<T> for TextBuilder
where
    T: AsRef<str>,
{
    fn add_assign(&mut self, other: T) {
        self.append(other);
    }
}

impl From<TextBuilder> for String {
    fn from(value: TextBuilder) -> Self {
        value.buffer
    }
}

impl From<&TextBuilder> for String {
    fn from(value: &TextBuilder) -> Self {
        value.buffer.clone()
    }
}

impl AsRef<str> for TextBuilder {
    fn as_ref(&self) -> &str {
        self.buffer.as_str()
    }
}
