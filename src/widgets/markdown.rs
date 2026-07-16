mod buffer;
pub(crate) mod config;
mod handler;

use crate::widgets::markdown::handler::Handler;
use crate::{rc_layout, BoxedFormattedLayout, Dimension, Layout, LayoutOptions, RcLayout, WrapMode, MeasureMode, Measurements, LayoutContext};
pub use config::FrameConfig;
pub use config::MarkdownConfig;
use pulldown_cmark::{Options, Parser};
use std::any::Any;

/// A widget that renders Markdown content using the `termlayout` widget system.
///
/// This widget parses a Markdown string and converts it into a hierarchy of native widgets
/// like [`Paragraph`](crate::widgets::Paragraph), [`List`](crate::widgets::List), and
/// [`Table`](crate::widgets::Table). It supports `CommonMark` and GFM features
/// like tables, strikethrough, task lists, and GitHub-flavored alerts.
///
/// # Features
/// - **Headings**: H1-H6 with customizable styling
/// - **Text formatting**: Bold, italic, strikethrough, inline code
/// - **Lists**: Ordered and unordered lists with nesting
/// - **Tables**: Full table support with headers and alignment
/// - **Code blocks**: Fenced code blocks with language hints
/// - **Block quotes**: Including GFM alerts (NOTE, TIP, IMPORTANT, WARNING, CAUTION)
/// - **Math**: Inline and display math expressions
/// - **Links and images**: With customizable styling
///
/// # Example
/// ```rust
/// use termlayout::widgets::Markdown;
/// use termlayout::Layout;
///
/// let md = Markdown::new("# Hello\nThis is **bold** and *italic*.");
/// let output = md.layout(80);
/// println!("{}", output);
/// ```
///
/// # Custom Styling
/// ```rust
/// use termlayout::widgets::{Markdown, MarkdownConfig};
/// use termlayout::ext::{Style, Effect, Color};
/// use termlayout::Layout;
///
/// let mut config = MarkdownConfig::default();
/// config.inline_strong = &|style| style
///     .with_foreground(Color::Red)
///     .with_effect(Effect::Bold);
///
/// let md = Markdown::with_config("**Custom bold**", &config);
/// ```
pub struct Markdown {
    inner: RcLayout,
}

impl Markdown {
    /// Creates a new `Markdown` widget from the provided source string with default styling.
    ///
    /// # Parameters
    /// - `source`: The Markdown source text.
    ///
    /// # Returns
    /// A new `Markdown` instance.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::widgets::Markdown;
    /// use termlayout::Layout;
    ///
    /// let md = Markdown::new("# Title\n\nParagraph with **bold** text.");
    /// println!("{}", md.layout(80));
    /// ```
    pub fn new<T>(source: T) -> Self
    where
        T: AsRef<str>,
    {
        Self::with_config(source, &MarkdownConfig::default())
    }

    /// Creates a new `Markdown` widget with custom configuration.
    ///
    /// # Parameters
    /// - `source`: The Markdown source text.
    /// - `config`: The `MarkdownConfig` to use for styling.
    ///
    /// # Returns
    /// A new `Markdown` instance with custom styling.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::widgets::{Markdown, MarkdownConfig};
    /// use termlayout::ext::{Style, Effect, Color};
    /// use termlayout::Layout;
    ///
    /// let mut config = MarkdownConfig::default();
    /// config.inline_strong = &|style| style
    ///     .with_foreground(Color::Blue)
    ///     .with_effect(Effect::Bold);
    ///
    /// let md = Markdown::with_config("# Custom Header", &config);
    /// ```
    pub fn with_config<T>(source: T, config: &MarkdownConfig) -> Self
    where
        T: AsRef<str>,
    {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_MATH);
        options.insert(Options::ENABLE_GFM);
        let inner = Handler::new(config, &mut Parser::new_ext(source.as_ref(), options)).handle();
        Markdown { inner }
    }
}

impl Layout for Markdown {
    fn pref_dim(&self, max_width: usize, wrap_mode: WrapMode) -> Dimension {
        self.inner.pref_dim(max_width, wrap_mode)
    }

    fn min_dim(&self) -> Dimension {
        self.inner.min_dim()
    }

    fn measure(&self, mode: MeasureMode) -> Measurements {
        todo!()
    }

    fn layout_strict(&'_ self, options: LayoutOptions) -> BoxedFormattedLayout<'_> {
        self.inner.layout_strict(options)
    }

    fn layout_with_context(&'_ self, context: LayoutContext) -> BoxedFormattedLayout<'_> {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

rc_layout!(Markdown);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Layout;

    #[test]
    fn markdown_smoke_test() {
        let md = Markdown::new(
            "# Header\n\nThis is a paragraph with **bold** text.\n\n- Item 1\n- Item 2",
        );
        let output = format!("{}", md.layout(40));
        assert!(output.contains("Header"));
        assert!(output.contains("Item 1"));
        assert!(output.contains("Item 2"));
    }
}
