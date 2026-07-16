use crate::ext::{Color, DisplayStr, Effect, Style, Transition};
use std::mem;
use std::ops::AddAssign;

/// A builder for building text with styles.
/// A `TextBuilder` allows the creation of styled text by [`appending`](TextBuilder::append) text
/// fragments and [pushing](TextBuilder::push_style) and [popping](TextBuilder::pop_last_style) [`Style`]
/// information to style the text.
///
/// # Example
/// ```rust
/// use termlayout::ext::{Color, Effect, Style, TextBuilder};
///
/// let mut builder = TextBuilder::new();
///
/// builder.append("A ");
/// builder.push_style_change(|s| s.with_foreground(Color::Blue));
/// builder.append("styled ");
/// builder.push_style_change(|s| s.with_effect(Effect::Bold));
/// builder.append("text");
/// builder.pop_last_style();
/// builder.pop_last_style();
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
    /// builder.push_style(Style::default().with_foreground(Color::Red));
    /// assert_eq!(builder.current_style(), Style::default().with_foreground(Color::Red));
    ///
    /// builder.pop_last_style();
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
    /// The counterpart is [`pop_style`](TextBuilder::pop_last_style), which pops from the style stack.
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
    pub fn push_style(&mut self, new_style: Style) {
        let current_style = self.current_style();
        let transition = Transition::new(current_style, new_style);
        transition.render(&mut self.buffer).unwrap();
        self.style_stack.push(new_style);
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
    /// builder.push_style(Style::default().with_effect(Effect::Bold));
    ///
    /// builder.push_style_change(|s| s.with_effect(Effect::Italic));
    /// assert_eq!(builder.as_ref(), "\x1b[1m\x1b[3m");
    /// assert_eq!(builder.current_style(),
    ///     Style::default()
    ///         .with_effect(Effect::Italic)
    ///         .with_effect(Effect::Bold));
    /// ```
    pub fn push_style_change<T>(&mut self, change: T)
    where
        T: FnOnce(Style) -> Style,
    {
        let current_style = self.current_style();
        let new_style = change(current_style);
        self.push_style(new_style);
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
    ///     .with_style(Style::default().with_effect(Effect::Bold));
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

    /// Shortcut for calling `with_style_change` to set the given effect
    ///
    /// # Parameters
    /// - `effect`: The effect to set
    ///
    /// # Returns
    /// The modified `TextBuilder` instance
    #[must_use]
    pub fn with_effect(mut self, effect: Effect) -> Self {
        self.with_style_change(|s| s.with_effect(effect))
    }

    /// Shortcut for calling `with_style_change` to set the bold effect
    ///
    /// # Returns
    /// The modified `TextBuilder` instance
    #[must_use]
    pub fn with_bold(mut self) -> Self {
        self.with_effect(Effect::Bold)
    }

    /// Shortcut for calling `with_style_change` to set the italic effect
    ///
    /// # Returns
    /// The modified `TextBuilder` instance
    #[must_use]
    pub fn with_italic(mut self) -> Self {
        self.with_effect(Effect::Italic)
    }

    /// Shortcut for calling `with_style_change` to set the underline effect
    ///
    /// # Returns
    /// The modified `TextBuilder` instance
    #[must_use]
    pub fn with_underline(mut self) -> Self {
        self.with_effect(Effect::Underline)
    }

    /// Shortcut for calling `with_style_change` to set foreground color
    ///
    /// # Parameters
    /// - `color`: The foreground color to set
    ///
    /// # Returns
    /// The modified `TextBuilder` instance
    #[must_use]
    pub fn with_color(mut self, color: Color) -> Self {
        self.with_style_change(|s| s.with_foreground(color))
    }

    /// Shortcut for calling `with_style_change` to set foreground color to red
    ///
    /// # Returns
    /// The modified `TextBuilder` instance
    #[must_use]
    pub fn with_red(mut self) -> Self {
        self.with_color(Color::Red)
    }

    /// Shortcut for calling `with_style_change` to set foreground color to green
    ///
    /// # Returns
    /// The modified `TextBuilder` instance
    #[must_use]
    pub fn with_green(mut self) -> Self {
        self.with_color(Color::Green)
    }

    /// Shortcut for calling `with_style_change` to set foreground color to yellow
    ///
    /// # Returns
    /// The modified `TextBuilder` instance
    #[must_use]
    pub fn with_yellow(mut self) -> Self {
        self.with_color(Color::Yellow)
    }

    /// Shortcut for calling `with_style_change` to set foreground color to blue
    ///
    /// # Returns
    /// The modified `TextBuilder` instance
    #[must_use]
    pub fn with_blue(mut self) -> Self {
        self.with_color(Color::Blue)
    }

    /// Shortcut for calling `with_style_change` to set foreground color to magenta
    ///
    /// # Returns
    /// The modified `TextBuilder` instance
    #[must_use]
    pub fn with_magenta(mut self) -> Self {
        self.with_color(Color::Magenta)
    }

    /// Shortcut for calling `with_style_change` to set foreground color to cyan
    ///
    /// # Returns
    /// The modified `TextBuilder` instance
    #[must_use]
    pub fn with_cyan(mut self) -> Self {
        self.with_color(Color::Cyan)
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
    /// builder.push_style(Style::default().with_effect(Effect::Bold));
    /// builder.push_style_change(|s| s.with_effect(Effect::Italic));
    ///
    /// builder.pop_last_style();
    /// assert_eq!(builder.as_ref(), "\x1b[1m\x1b[3m\x1b[23m");
    /// assert_eq!(builder.current_style(),
    ///     Style::default()
    ///         .with_effect(Effect::Bold));
    /// ```
    ///
    /// # Panics
    /// Panics if writing the style transition to the internal buffer fails.
    pub fn pop_last_style(&mut self) {
        let style = self.style_stack.pop().unwrap_or_default();
        let transition = Transition::new(style, self.current_style());
        if !transition.is_empty() {
            transition.render(&mut self.buffer).unwrap();
        }
    }

    /// Pops the current style from the style stack with consuming the builder.
    /// It also emits the characters necessary to transition from the current style to the new style
    /// into the buffer.
    ///
    /// If the stack is empty, the method has no effect
    ///
    /// This is the consuming version of [`pop_last_style`](TextBuilder::pop_last_style), which allows
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
    ///     .with_style(Style::default().with_effect(Effect::Bold))
    ///     .with_style_change(|s| s.with_effect(Effect::Italic));
    ///
    /// let builder = builder.without_last_style();
    /// assert_eq!(builder.as_ref(), "\x1b[1m\x1b[3m\x1b[23m");
    /// assert_eq!(builder.current_style(),
    ///     Style::default()
    ///         .with_effect(Effect::Bold));
    /// ```
    ///
    /// # Panics
    /// Panics if writing the style transition to the internal buffer fails.
    #[must_use]
    pub fn without_last_style(mut self) -> Self {
        self.pop_last_style();
        self
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
    ///  builder.append("\x1b[1mHello");
    ///
    /// assert_eq!(builder.as_ref(), "\x1b[1mHello");
    /// assert_eq!(builder.current_style(), Style::default());
    /// ```
    pub fn append<T>(&mut self, text: T)
    where
        T: AsRef<str>,
    {
        self.buffer.push_str(text.as_ref());
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
    /// use termlayout::ext::{Effect, Style, TextBuilder};
    ///
    /// let mut builder = TextBuilder::new();
    ///
    /// builder.push_style_change(|s| s.with_effect(Effect::Bold));
    /// builder.append("Hallo");
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
    /// builder.push_style_change(|s| s.with_effect(Effect::Bold));
    /// builder.append("Hallo");
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
