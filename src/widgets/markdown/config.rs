use crate::ext::{Color, Effect, Style, TextBuilder};
use crate::widgets::TableDecoration;
use pulldown_cmark::BlockQuoteKind;
use std::borrow::Cow;

/// Styling and framing configuration for the [`Markdown`](crate::widgets::Markdown) widget.
///
/// Each field controls how a specific Markdown construct is rendered. Inline styles are applied
/// via functions that transform a base [`Style`]; block-level constructs use [`FrameConfig`]
/// to define a frame decoration and initial text style.
pub struct MarkdownConfig<'a> {
    /// Whether to use compact mode. If `true` it formats with less newlines for spacing vertically
    pub compact: bool,
    /// Style applied to `**strong**` text.
    pub inline_strong: &'a dyn Fn(Style) -> Style,
    /// Style applied to `*emphasis*` text.
    pub inline_emphasis: &'a dyn Fn(Style) -> Style,
    /// Style applied to `~~strikethrough~~` text.
    pub inline_strikethrough: &'a dyn Fn(Style) -> Style,
    /// Style applied to `` `inline code` `` text.
    pub inline_code: &'a dyn Fn(Style) -> Style,
    /// Style applied to inline math expressions.
    pub inline_math: &'a dyn Fn(Style) -> Style,
    /// Frame and style for display (block) math.
    pub display_math: FrameConfig<'a>,
    /// Frame and style for fenced code blocks.
    pub code_block: FrameConfig<'a>,
    /// Default frame and style for block quotes without a GFM alert type.
    pub block_quote: FrameConfig<'a>,
    /// Frame and style for GFM `> [!NOTE]` block quotes.
    pub block_quote_note: Option<FrameConfig<'a>>,
    /// Frame and style for GFM `> [!TIP]` block quotes.
    pub block_quote_tip: Option<FrameConfig<'a>>,
    /// Frame and style for GFM `> [!IMPORTANT]` block quotes.
    pub block_quote_important: Option<FrameConfig<'a>>,
    /// Frame and style for GFM `> [!WARNING]` block quotes.
    pub block_quote_warning: Option<FrameConfig<'a>>,
    /// Frame and style for GFM `> [!CAUTION]` block quotes.
    pub block_quote_caution: Option<FrameConfig<'a>>,
    /// Frame and style for level one headers
    pub header_1: HeaderConfig<'a>,
    /// Frame and style for level two headers
    pub header_2: HeaderConfig<'a>,
    /// Frame and style for level three headers
    pub header_3: HeaderConfig<'a>,
    /// Frame and style for level four headers
    pub header_4: HeaderConfig<'a>,
    /// Frame and style for level five headers
    pub header_5: HeaderConfig<'a>,
    /// Frame and style for level six headers
    pub header_6: HeaderConfig<'a>,
    /// Styled string for rule
    pub rule: &'a str,
    /// Style for links
    pub link: LinkConfig<'a>,
    /// Style for images
    pub image: LinkConfig<'a>,
    /// Decoration spec for tables
    pub table: TableDecoration,
}

impl<'a> MarkdownConfig<'a> {
    /// Creates a new `MarkdownConfig` with dark theme styling.
    ///
    /// # Returns
    /// A `MarkdownConfig` instance configured for dark backgrounds
    #[must_use]
    pub fn dark() -> Self {
        Self::new(&Styles::DARK)
    }

    /// Creates a new `MarkdownConfig` with light theme styling.
    ///
    /// # Returns
    /// A `MarkdownConfig` instance configured for light backgrounds
    #[must_use]
    pub fn light() -> Self {
        Self::new(&Styles::LIGHT)
    }

    #[allow(clippy::too_many_lines)]
    fn new(styles: &'a Styles) -> Self {
        Self {
            compact: false,
            inline_strong: &|style| style.with_effect(Effect::Bold),
            inline_emphasis: &|style| style.with_effect(Effect::Italic),
            inline_strikethrough: &|style| style.with_effect(Effect::Strikethrough),
            inline_code: &styles.code_style,
            inline_math: &|style| style.with_effect(Effect::Dim).with_effect(Effect::Italic),
            display_math: FrameConfig {
                frame_spec: Cow::Borrowed("  C"),
                initial_style: Style::default()
                    .with_effect(Effect::Dim)
                    .with_effect(Effect::Italic),
            },
            code_block: FrameConfig {
                frame_spec: Cow::Borrowed("  C "),
                initial_style: (styles.code_style)(Style::default()),
            },
            block_quote: FrameConfig {
                frame_spec: Cow::Borrowed("│ C"),
                initial_style: Style::default().with_effect(Effect::Italic),
            },
            block_quote_tip: Some(FrameConfig {
                frame_spec: Cow::Owned(
                    TextBuilder::new()
                        .with_text("  ")
                        .with_style((styles.quote_with_marker_background_style)(Style::default()))
                        .with_text("💡 \n")
                        .without_last_style()
                        .with_text("  ")
                        .with_style((styles.quote_with_marker_background_style)(
                            Style::default().with_foreground(Color::Green),
                        ))
                        .with_text("▌ ")
                        .without_last_style()
                        .with_text("C ")
                        .into(),
                ),
                initial_style: (styles.quote_with_marker_background_style)(Style::default()),
            }),
            block_quote_note: Some(FrameConfig {
                frame_spec: Cow::Owned(
                    TextBuilder::new()
                        .with_text("  ")
                        .with_style((styles.quote_with_marker_background_style)(Style::default()))
                        .with_text("✨ \n")
                        .without_last_style()
                        .with_text("  ")
                        .with_style((styles.quote_with_marker_background_style)(
                            Style::default().with_foreground(Color::Green),
                        ))
                        .with_text("▌ ")
                        .without_last_style()
                        .with_text("C ")
                        .into(),
                ),
                initial_style: (styles.quote_with_marker_background_style)(Style::default()),
            }),
            block_quote_important: Some(FrameConfig {
                frame_spec: Cow::Owned(
                    TextBuilder::new()
                        .with_text("  ")
                        .with_style((styles.quote_with_marker_background_style)(Style::default()))
                        .with_text("ℹ️ \n")
                        .without_last_style()
                        .with_text("  ")
                        .with_style((styles.quote_with_marker_background_style)(
                            Style::default().with_foreground(Color::Cyan),
                        ))
                        .with_text("▌ ")
                        .without_last_style()
                        .with_text("C ")
                        .into(),
                ),
                initial_style: (styles.quote_with_marker_background_style)(Style::default()),
            }),
            block_quote_warning: Some(FrameConfig {
                frame_spec: Cow::Owned(
                    TextBuilder::new()
                        .with_text("  ")
                        .with_style((styles.quote_with_marker_background_style)(Style::default()))
                        .with_text("⚠️ \n")
                        .without_last_style()
                        .with_text("  ")
                        .with_style((styles.quote_with_marker_background_style)(
                            Style::default().with_foreground(Color::Magenta),
                        ))
                        .with_text("▌ ")
                        .without_last_style()
                        .with_text("C ")
                        .into(),
                ),
                initial_style: (styles.quote_with_marker_background_style)(Style::default()),
            }),
            block_quote_caution: Some(FrameConfig {
                frame_spec: Cow::Owned(
                    TextBuilder::new()
                        .with_text("  ")
                        .with_style((styles.quote_with_marker_background_style)(Style::default()))
                        .with_text("🛑 \n")
                        .without_last_style()
                        .with_text("  ")
                        .with_style((styles.quote_with_marker_background_style)(
                            Style::default().with_foreground(Color::Red),
                        ))
                        .with_text("▌ ")
                        .without_last_style()
                        .with_text("C ")
                        .into(),
                ),
                initial_style: (styles.quote_with_marker_background_style)(Style::default()),
            }),
            header_1: HeaderConfig {
                style: &styles.title_style,
                prefix: "",
            },
            header_2: HeaderConfig {
                style: &styles.header_style,
                prefix: "## ",
            },
            header_3: HeaderConfig {
                style: &styles.header_style,
                prefix: "### ",
            },
            header_4: HeaderConfig {
                style: &styles.header_style,
                prefix: "#### ",
            },
            header_5: HeaderConfig {
                style: &styles.header_style,
                prefix: "##### ",
            },
            header_6: HeaderConfig {
                style: &styles.header_style,
                prefix: "###### ",
            },
            rule: "─",
            link: LinkConfig {
                label_style: Some(&|s| s.with_foreground(Color::Cyan)),
                link_style: Some(&|s| {
                    s.with_effect(Effect::Underline)
                        .with_foreground(Color::Blue)
                }),
            },
            image: LinkConfig {
                label_style: Some(&|s| s.with_foreground(Color::Cyan)),
                link_style: Some(&|s| {
                    s.with_effect(Effect::Underline)
                        .with_foreground(Color::Blue)
                }),
            },
            table: TableDecoration::boxed_grid(),
        }
    }

    pub(crate) fn get_frame_config_for_block_quote(
        &self,
        kind: Option<BlockQuoteKind>,
    ) -> &FrameConfig<'a> {
        match kind {
            None => &self.block_quote,
            Some(kind) => match kind {
                BlockQuoteKind::Note => self.block_quote_note.as_ref(),
                BlockQuoteKind::Tip => self.block_quote_tip.as_ref(),
                BlockQuoteKind::Important => self.block_quote_important.as_ref(),
                BlockQuoteKind::Warning => self.block_quote_warning.as_ref(),
                BlockQuoteKind::Caution => self.block_quote_caution.as_ref(),
            }
            .unwrap_or(&self.block_quote),
        }
    }
}

impl Default for MarkdownConfig<'static> {
    fn default() -> Self {
        Self::dark()
    }
}

/// Frame decoration and initial style for a block-level Markdown element.
pub struct FrameConfig<'a> {
    /// Frame specification string passed to [`FrameDecoration::from_spec`](crate::widgets::FrameDecoration::from_spec).
    pub frame_spec: Cow<'a, str>,
    /// Initial [`Style`] applied to the framed content.
    pub initial_style: Style,
}

/// Config describing how to decorate links.
pub struct LinkConfig<'a> {
    pub label_style: Option<&'a dyn Fn(Style) -> Style>,
    pub link_style: Option<&'a dyn Fn(Style) -> Style>,
}

pub struct HeaderConfig<'a> {
    pub style: &'a dyn Fn(Style) -> Style,
    pub prefix: &'a str,
}

#[allow(clippy::struct_field_names)]
struct Styles {
    title_style: fn(Style) -> Style,
    header_style: fn(Style) -> Style,
    code_style: fn(Style) -> Style,
    quote_with_marker_background_style: fn(Style) -> Style,
}

impl Styles {
    const DARK: Styles = Styles {
        title_style: |s| {
            s.with_foreground(Color::Custom8(33))
                .with_effect(Effect::Bold)
                .with_effect(Effect::Underline)
        },
        header_style: |s| {
            s.with_foreground(Color::Custom8(33))
                .with_effect(Effect::Bold)
        },
        code_style: |s| s.with_background(Color::Custom8(235)),
        quote_with_marker_background_style: |s| s.with_background(Color::Custom8(235)),
    };

    const LIGHT: Styles = Styles {
        title_style: |s| {
            s.with_foreground(Color::Custom8(33))
                .with_effect(Effect::Bold)
                .with_effect(Effect::Underline)
        },
        header_style: |s| {
            s.with_foreground(Color::Custom8(33))
                .with_effect(Effect::Bold)
        },
        code_style: |s| s.with_background(Color::Custom8(254)),
        quote_with_marker_background_style: |s| s.with_background(Color::Custom8(254)),
    };
}
