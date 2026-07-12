use crate::ext::{Effect, Style};
use crate::widgets::lines::LinesTrimming;
use crate::widgets::markdown::buffer::LayoutBuffer;
use crate::widgets::markdown::config::LinkConfig;
use crate::widgets::{
    Cell, CellAnchor, CellDimension, CellWidth, Filler, Horizontal, Lines, ListItemMarker,
    Paragraph, Table, TableColumn,
};
use crate::{MarkdownConfig, RcLayout, WrapMode};
use pulldown_cmark::{Alignment, Event, HeadingLevel, Parser, Tag, TagEnd};

/// Internal handler for processing markdown events.
///
/// This handler maintains a stack of buffers to handle nested markdown structures
/// like lists, block quotes, and tables. It processes events from the pulldown-cmark
/// parser and converts them into layout structures.
pub(crate) struct Handler<'a> {
    /// The markdown configuration to use for styling
    config: &'a MarkdownConfig<'a>,
    /// The markdown parser providing events
    parser: &'a mut Parser<'a>,
    /// Stack of buffers for handling nested structures
    buffer_stack: Vec<LayoutBuffer>,
}

impl<'a> Handler<'a> {
    /// Creates a new markdown handler.
    ///
    /// # Parameters
    /// - `config`: The markdown configuration to use
    /// - `parser`: The markdown parser to process
    ///
    /// # Returns
    /// A new handler instance
    pub(crate) fn new(config: &'a MarkdownConfig<'a>, parser: &'a mut Parser<'a>) -> Self {
        Self {
            config,
            parser,
            buffer_stack: vec![LayoutBuffer::default()],
        }
    }

    /// Pops the current buffer from the stack.
    ///
    /// # Returns
    /// The popped buffer
    ///
    /// # Panics
    /// Panics if the buffer stack is empty
    fn pop_buffer(&mut self) -> LayoutBuffer {
        self.buffer_stack
            .pop()
            .expect("buffer stack should not be empty")
    }

    /// Pushes a new empty buffer onto the stack.
    fn push_buffer(&mut self, style: Style) {
        self.buffer_stack
            .push(LayoutBuffer::with_default_style(style));
    }

    /// Returns a mutable reference to the current buffer.
    ///
    /// # Returns
    /// A mutable reference to the top buffer on the stack
    ///
    /// # Panics
    /// Panics if the buffer stack is empty
    fn buffer(&mut self) -> &mut LayoutBuffer {
        self.buffer_stack
            .last_mut()
            .expect("buffer stack should not be empty")
    }

    /// Processes all markdown events and returns the resulting layout.
    ///
    /// # Returns
    /// The complete layout for the markdown content
    #[allow(clippy::while_let_on_iterator)]
    pub(crate) fn handle(&mut self) -> RcLayout {
        while let Some(event) = self.parser.next() {
            self.default_event_handler(event);
        }
        let mut buffer = self.pop_buffer();
        buffer.flush_as_vertical(|s| Paragraph::left(s).into())
    }

    /// Handles a markdown list (ordered or unordered).
    ///
    /// # Parameters
    /// - `initial_index`: The starting index for ordered lists, or `None` for unordered lists
    ///
    /// # Returns
    /// The layout for the list
    fn handle_list(&mut self, initial_index: Option<u64>) -> RcLayout {
        let layout = self.buffer().flush_as_paragraph();
        self.buffer().append_layouts(&layout);

        self.push_buffer(Style::default());
        while let Some(event) = self.parser.next() {
            match event {
                Event::End(TagEnd::List(_)) => break,
                Event::Start(Tag::Item) => {
                    self.buffer().flush_text_as_paragraph(false);
                    self.push_buffer(Style::default());
                }
                Event::End(TagEnd::Item) => {
                    let layout = self
                        .pop_buffer()
                        .flush_as_vertical(|s| Paragraph::left(s).into());
                    self.buffer().append_layout(layout);
                }
                _ => self.default_event_handler(event),
            }
        }
        let mut buffer = self.pop_buffer();
        let item_marker = match initial_index {
            Some(index) => ListItemMarker::default_numbered()
                .with_first_index(index.saturating_sub(1).try_into().unwrap()),
            None => ListItemMarker::default_fixed(),
        };
        buffer.flush_as_list(item_marker)
    }

    /// Handles a single table row.
    ///
    /// # Parameters
    /// - `end_tag`: The tag that marks the end of the row
    ///
    /// # Returns
    /// A vector of layouts, one for each cell in the row
    fn handle_table_row(&mut self, end_tag: TagEnd) -> Vec<RcLayout> {
        self.push_buffer(Style::default());

        while let Some(event) = self.parser.next() {
            match event {
                Event::Start(Tag::TableCell) => {
                    self.buffer().flush_text_as_paragraph(false);
                    self.push_buffer(Style::default());
                }
                Event::End(TagEnd::TableCell) => {
                    let layout = self.pop_buffer().flush_as_paragraph();
                    self.buffer().append_layouts(&layout);
                }
                Event::End(tag) if tag == end_tag => break,
                _ => self.default_event_handler(event),
            }
        }

        self.pop_buffer().flush_as_paragraph()
    }

    /// Handles a markdown table.
    ///
    /// # Parameters
    /// - `alignments`: The column alignments for the table
    ///
    /// # Returns
    /// The layout for the table
    fn handle_table(&mut self, alignments: &[Alignment]) -> RcLayout {
        let mut cells = vec![];
        let mut columns = vec![];

        while let Some(event) = self.parser.next() {
            match event {
                Event::End(TagEnd::Table) => break,
                Event::Start(Tag::TableHead) => {
                    let row = self.handle_table_row(TagEnd::TableHead);
                    row.iter()
                        .zip(alignments.iter())
                        .for_each(|(cell, alignment)| {
                            let (width, anchor) = match alignment {
                                Alignment::None => (CellWidth::Fill, CellAnchor::NorthWest),
                                Alignment::Left => (CellWidth::Minimal, CellAnchor::NorthWest),
                                Alignment::Center => (CellWidth::Fill, CellAnchor::North),
                                Alignment::Right => (CellWidth::Fill, CellAnchor::NorthEast),
                            };
                            columns.push(TableColumn::new(
                                Some(cell.clone()),
                                width,
                                anchor,
                                WrapMode::default(),
                            ));
                        });
                }
                Event::Start(Tag::TableRow) => {
                    let row = self.handle_table_row(TagEnd::TableRow);
                    cells.push(row);
                }
                _ => self.default_event_handler(event),
            }
        }

        Table::new(self.config.table.clone(), columns, cells).into()
    }

    /// Handles a markdown heading.
    ///
    /// # Returns
    /// The layout for the heading
    fn handle_heading(&mut self) -> Vec<RcLayout> {
        self.buffer().flush_text_as_paragraph(false);
        self.push_buffer(Style::default());
        while let Some(event) = self.parser.next() {
            match event {
                Event::End(TagEnd::Heading(level)) => {
                    let config = match level {
                        HeadingLevel::H1 => &self.config.header_1,
                        HeadingLevel::H2 => &self.config.header_2,
                        HeadingLevel::H3 => &self.config.header_3,
                        HeadingLevel::H4 => &self.config.header_4,
                        HeadingLevel::H5 => &self.config.header_5,
                        HeadingLevel::H6 => &self.config.header_6,
                    };
                    let style = (config.style)(Style::default());
                    let mut prefix: Lines = Lines::left_with_style(style, config.prefix);
                    prefix.trimming = LinesTrimming::None;
                    let layout = self
                        .pop_buffer()
                        .flush_as_vertical(|s| Paragraph::left_with_style(style, s).into());
                    let horizontal = Horizontal::new(
                        vec![
                            Cell::new(
                                prefix,
                                CellDimension::Declarative(CellWidth::Minimal),
                                None,
                                None,
                                CellAnchor::NorthWest,
                            ),
                            Cell::new(
                                layout,
                                CellDimension::Declarative(CellWidth::Fill),
                                None,
                                None,
                                CellAnchor::West,
                            ),
                        ],
                        None,
                    );
                    self.buffer().append_newline_unless_empty();
                    self.buffer().append_layout(horizontal.into());
                    return self.buffer().flush_as_paragraph();
                }
                _ => self.default_event_handler(event),
            }
        }
        self.buffer().flush_as_paragraph()
    }

    /// Handles an HTML block.
    ///
    /// # Returns
    /// The layout for the HTML block
    fn handle_html(&mut self) -> Vec<RcLayout> {
        self.buffer().flush_text_as_paragraph(false);
        while let Some(event) = self.parser.next() {
            match event {
                Event::End(TagEnd::HtmlBlock) => break,
                Event::Html(html) => self.buffer().append_text(html.as_ref()),
                _ => self.default_event_handler(event),
            }
        }
        self.buffer().flush_as_paragraph()
    }

    /// Handles a horizontal rule.
    fn handle_rule(&mut self) {
        let config = self.config.rule;
        self.buffer()
            .append_layout(Filler::horizontal(config).into());
    }

    fn handle_link(&mut self, config: &LinkConfig, url: &str, end_tag: TagEnd) {
        let mut empty_label = true;
        if let Some(style) = config.label_style {
            self.buffer().push_style_change(style);
        }
        while let Some(event) = self.parser.next() {
            if event == Event::End(end_tag) {
                if config.label_style.is_some() {
                    self.buffer().pop_style();
                }
                if let Some(style) = config.link_style
                    && !url.is_empty()
                {
                    if !empty_label {
                        self.buffer().append_text(" (➔");
                    }
                    self.buffer().append_styled_text(style, url);
                    if !empty_label {
                        self.buffer().append_text(")");
                    }
                }
                break;
            } else if config.link_style.is_some() {
                self.default_event_handler(event);
                empty_label = false;
            }
        }
    }

    /// Handles a single markdown event.
    ///
    /// # Parameters
    /// - `event`: The markdown event to process
    #[allow(clippy::match_same_arms, clippy::too_many_lines)]
    fn default_event_handler(&mut self, event: Event) {
        match event {
            // Plain Text
            Event::Text(text) => self.buffer().append_text(text),
            Event::SoftBreak => self.buffer().append_text("\n"),
            // Inline formatting
            Event::Start(Tag::Emphasis) => {
                let config = self.config.inline_emphasis;
                self.buffer().push_style_change(config);
            }
            Event::End(TagEnd::Emphasis) => self.buffer().pop_style(),
            Event::Start(Tag::Strong) => {
                let config = self.config.inline_strong;
                self.buffer().push_style_change(config);
            }
            Event::End(TagEnd::Strong) => self.buffer().pop_style(),
            Event::Start(Tag::Strikethrough) => {
                let config = self.config.inline_strikethrough;
                self.buffer().push_style_change(config);
            }
            Event::End(TagEnd::Strikethrough) => self.buffer().pop_style(),
            Event::InlineHtml(html) => self.handle_html_tag(html.as_ref()),
            Event::Code(code) => {
                let config = self.config.inline_code;
                self.buffer().append_styled_text(config, code);
            }
            Event::InlineMath(math) => {
                let config = self.config.inline_math;
                self.buffer().append_styled_text(config, math);
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                let config = &self.config.link;
                self.handle_link(config, dest_url.as_ref(), TagEnd::Link);
            }
            Event::Start(Tag::Image { dest_url, .. }) => {
                let config = &self.config.image;
                self.handle_link(config, dest_url.as_ref(), TagEnd::Image);
            }
            // Unsupported inline formatting
            Event::Start(Tag::Subscript) => {}
            Event::End(TagEnd::Subscript) => {}
            Event::Start(Tag::Superscript) => {}
            Event::End(TagEnd::Superscript) => {}
            // Paragraph
            Event::Start(Tag::Paragraph) => {
                self.buffer().flush_text_as_paragraph(false);
            }
            Event::End(TagEnd::Paragraph) => {
                self.buffer().flush_text_as_paragraph(false);
            }
            // Header
            Event::Start(Tag::Heading { .. }) => {
                let layout = self.handle_heading();
                self.buffer().append_layouts(&layout);
                self.emit_newline_unless_compact();
            }
            // CodeBlock
            Event::Start(Tag::CodeBlock(_)) => {
                let config = &self.config.code_block;
                self.buffer().flush_text_as_paragraph(true);
                self.push_buffer(config.initial_style);
            }
            Event::End(TagEnd::CodeBlock) => {
                let config = &self.config.code_block;
                let layout = self.pop_buffer().flush_as_frame_with_lines(config);
                self.emit_newline_unless_compact();
                self.buffer().append_layout(layout);
                self.emit_newline_unless_compact();
            }
            // BlockQuote
            Event::Start(Tag::BlockQuote(kind)) => {
                let config = self.config.get_frame_config_for_block_quote(kind);
                self.buffer().flush_text_as_paragraph(true);
                self.push_buffer(config.initial_style);
            }
            Event::End(TagEnd::BlockQuote(kind)) => {
                let config = self.config.get_frame_config_for_block_quote(kind);
                let layout = self.pop_buffer().flush_as_frame_with_paragraph(config);
                self.emit_newline_unless_compact();
                self.buffer().append_layout(layout);
                self.emit_newline_unless_compact();
            }
            // DisplayMath
            Event::DisplayMath(math) => {
                let config = &self.config.display_math;
                self.buffer().flush_text_as_paragraph(true);
                self.push_buffer(config.initial_style);
                self.buffer().append_text(math.as_ref());
                let layout = self.pop_buffer().flush_as_frame_with_lines(config);
                self.emit_newline_unless_compact();
                self.buffer().append_layout(layout);
                self.emit_newline_unless_compact();
            }
            // List
            Event::Start(Tag::List(initial_index)) => {
                let layout = self.handle_list(initial_index);
                self.buffer().append_layout(layout);
            }
            // Table
            Event::Start(Tag::Table(alignments)) => {
                let layout = self.handle_table(alignments.as_ref());
                self.buffer().append_layout(layout);
            }
            // HtmlBlock
            Event::Start(Tag::HtmlBlock) => {
                let layout = self.handle_html();
                self.buffer().append_layouts(&layout);
            }
            // hardBreak
            Event::HardBreak => {
                let layout = self.buffer().flush_as_paragraph();
                self.buffer().append_layouts(&layout);
            }
            Event::Rule => self.handle_rule(),
            // Event::FootnoteReference(_) => {}
            // Event::TaskListMarker(_) => {}
            _ => {}
        }
    }

    /// Handles an inline HTML tag.
    ///
    /// # Parameters
    /// - `html`: The HTML tag content
    fn handle_html_tag(&mut self, html: &str) {
        if !html.ends_with('>') || !html.starts_with('<') {
            // Not a valid HTML tag
            self.buffer().append_text(html);
            return;
        }
        if html.starts_with("</") {
            self.handle_html_end_tag(html[2..html.len() - 1].trim().as_ref());
        } else {
            self.handle_html_start_tag(html[1..html.len() - 1].trim().as_ref());
        }
    }

    /// Handles an HTML opening tag.
    ///
    /// # Parameters
    /// - `tag`: The tag name (without angle brackets)
    fn handle_html_start_tag(&mut self, tag: &str) {
        if tag.eq_ignore_ascii_case("i") {
            self.buffer()
                .push_style_change(|s| s.with_effect(Effect::Italic));
        } else if tag.eq_ignore_ascii_case("b") {
            self.buffer()
                .push_style_change(|s| s.with_effect(Effect::Bold));
        } else if tag.eq_ignore_ascii_case("u") {
            self.buffer()
                .push_style_change(|s| s.with_effect(Effect::Underline));
        } else {
            self.buffer().append_text("<");
            self.buffer().append_text(tag);
            self.buffer().append_text(">");
        }
    }

    /// Handles an HTML closing tag.
    ///
    /// # Parameters
    /// - `tag`: The tag name (without angle brackets and slash)
    fn handle_html_end_tag(&mut self, tag: &str) {
        if tag.eq_ignore_ascii_case("i") {
            self.buffer()
                .push_style_change(|s| s.without_effect(Effect::Italic));
        } else if tag.eq_ignore_ascii_case("b") {
            self.buffer()
                .push_style_change(|s| s.without_effect(Effect::Bold));
        } else if tag.eq_ignore_ascii_case("u") {
            self.buffer()
                .push_style_change(|s| s.without_effect(Effect::Underline));
        } else {
            self.buffer().append_text("</");
            self.buffer().append_text(tag);
            self.buffer().append_text(">");
        }
    }

    fn emit_newline_unless_compact(&mut self) {
        if !self.config.compact {
            self.buffer().append_newline_unless_empty();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::widgets::markdown::handler::Handler;
    use crate::{MarkdownConfig, RcLayout};
    use pulldown_cmark::{Options, Parser};

    fn parse(source: &str) -> RcLayout {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_MATH);
        options.insert(Options::ENABLE_GFM);

        let config = MarkdownConfig::default();
        let mut parser = Parser::new_ext(source, options);
        let mut handler: Handler<'_> = Handler::new(&config, &mut parser);
        handler.handle()
    }

    #[test]
    fn paragraph_plain() {
        let source = "This is an example text";
        let layout = parse(source);
        let result = format!("{}", layout.layout(50));

        assert_eq!(result, "This is an example text\n");
    }

    #[test]
    fn paragraph_emphasis() {
        let source = "This _is_ an *example* text";
        let layout = parse(source);
        let result = format!("{}", layout.layout(50));

        assert_eq!(
            result,
            "This \x1b[3mis\x1b[0m an \x1b[3mexample\x1b[0m text\n"
        );
    }

    #[test]
    fn paragraph_strong() {
        let source = "This **is an example** text";
        let layout = parse(source);
        let result = format!("{}", layout.layout(50));

        assert_eq!(result, "This \x1b[1mis an example\x1b[0m text\n");
    }

    #[test]
    fn paragraph_strikethrough() {
        let source = "This ~~is an example~~ text";
        let layout = parse(source);
        let result = format!("{}", layout.layout(50));

        assert_eq!(result, "This \x1b[9mis an example\x1b[0m text\n");
    }

    #[test]
    fn paragraph_inline_code() {
        let source = "This `is an example` text";
        let layout = parse(source);
        let result = format!("{}", layout.layout(50));

        assert_eq!(result, "This \x1b[48;5;235mis an example\x1b[0m text\n");
    }

    #[test]
    fn paragraph_inline_math() {
        let source = "This $is an example$ text";
        let layout = parse(source);
        let result = format!("{}", layout.layout(50));

        assert_eq!(result, "This \x1b[2;3mis an example\x1b[0m text\n");
    }

    #[test]
    fn paragraph_display_math() {
        let source = "This **is $$a display math$$ example** text";
        let layout = parse(source);
        let result = format!("{}", layout.layout(50));

        assert_eq!(
            result,
            concat!(
                "This \x1b[1mis\x1b[0m\n",
                "\n",
                "  \x1b[2;3ma display math\x1b[0m\n",
                "\n",
                "\x1b[1mexample\x1b[0m text\n"
            )
        );
    }

    #[test]
    fn code_block() {
        let source = concat!(
            "This **is \n",
            "```text\n",
            "code block\n",
            "with two lines\n",
            "```\n",
            "example**"
        );
        let layout = parse(source);
        let result = format!("{}", layout.layout(50));

        assert_eq!(
            result,
            concat!(
                "This **is\n",
                "\n",
                "  \x1b[48;5;235mcode block    \x1b[0m \n",
                "  \x1b[48;5;235mwith two lines\x1b[0m \n",
                "\n",
                "example**\n"
            )
        );
    }
    #[test]
    fn block_quote() {
        let source = concat!(
            "This is a simple block quote example:\n",
            "> Simple BlockQuote\n",
            "\n",
            "This is a tip block quote example:\n",
            "> [!TIP]\n",
            "> Tip BlockQuote\n",
            "\n",
            "This is an important block quote example:\n",
            "> [!IMPORTANT]\n",
            "> Important BlockQuote\n",
            "\n",
            "This is a warning block quote example:\n",
            "> [!WARNING]\n",
            "> Warning BlockQuote\n",
            "\n",
            "This is a tip block quote example:\n",
            "> [!CAUTION]\n",
            "> Caution BlockQuote\n",
            "\n",
            "This is a tip block quote example:\n",
            "> [!NOTE]\n",
            "> Note BlockQuote\n",
        );
        let layout = parse(source);
        let result = format!("{}", layout.layout(50));

        assert_eq!(
            result,
            concat!(
                "This is a simple block quote example:\n",
                "\n",
                "│ \x1b[3mSimple BlockQuote\x1b[0m\n",
                "\n",
                "This is a tip block quote example:\n",
                "\n",
                "  \x1b[48;5;235m💡                                    \x1b[0m\n",
                "\x1b[0m  \x1b[32;48;5;235m▌ \x1b[0m\x1b[48;5;235mTip BlockQuote                      \x1b[0m \n",
                "\n",
                "This is an important block quote example:\n",
                "\n",
                "  \x1b[48;5;235mℹ\u{fe0f}                                    \x1b[0m\n",
                "\x1b[0m  \x1b[36;48;5;235m▌ \x1b[0m\x1b[48;5;235mImportant BlockQuote                \x1b[0m \n",
                "\n",
                "This is a warning block quote example:\n\n  \x1b[48;5;235m⚠\u{fe0f}                                    \x1b[0m\n",
                "\x1b[0m  \x1b[35;48;5;235m▌ \x1b[0m\x1b[48;5;235mWarning BlockQuote                  \x1b[0m \n",
                "\n",
                "This is a tip block quote example:\n\n  \x1b[48;5;235m🛑                                    \x1b[0m\n",
                "\x1b[0m  \x1b[31;48;5;235m▌ \x1b[0m\x1b[48;5;235mCaution BlockQuote                  \x1b[0m \n",
                "\n",
                "This is a tip block quote example:\n\n  \x1b[48;5;235m✨                                    \x1b[0m\n",
                "\x1b[0m  \x1b[32;48;5;235m▌ \x1b[0m\x1b[48;5;235mNote BlockQuote                     \x1b[0m \n",
                "\n"
            )
        );
    }

    #[test]
    fn header() {
        let source = concat!(
            "# Header `one`\n",
            "## Header two\n",
            "### Header 3\n",
            "#### Header 4\n",
            "##### Header 5\n",
            "###### Header 6\n",
        );
        let layout = parse(source);
        let result = format!("{}", layout.layout(50));

        assert_eq!(
            result,
            concat!(
                "\x1b[1;4;38;5;33mHeader \x1b[48;5;235mone\x1b[0m\n",
                "\n",
                "\x1b[1;38;5;33m## \x1b[0m\x1b[1;38;5;33mHeader two\x1b[0m\n",
                "\n",
                "\x1b[1;38;5;33m### \x1b[0m\x1b[1;38;5;33mHeader 3\x1b[0m\n",
                "\n",
                "\x1b[1;38;5;33m#### \x1b[0m\x1b[1;38;5;33mHeader 4\x1b[0m\n",
                "\n",
                "\x1b[1;38;5;33m##### \x1b[0m\x1b[1;38;5;33mHeader 5\x1b[0m\n",
                "\n",
                "\x1b[1;38;5;33m###### \x1b[0m\x1b[1;38;5;33mHeader 6\x1b[0m\n",
                "\n"
            )
        );
    }

    #[test]
    fn list() {
        let source = concat!(
            "5. abc **def**\n",
            "1. ghi\n",
            "   - jkl\n",
            "   - mno\n",
            "\n",
            "10. abc **def**\n",
            "10. ghi\n",
        );
        let layout = parse(source);
        let result = format!("{}", layout.layout(50));

        assert_eq!(
            result,
            concat!(
                "5. abc \x1b[1mdef\x1b[0m\n", //
                "6. ghi\n",                   //
                "   • jkl\n",                 //
                "   • mno\n",                 //
                "7. abc \x1b[1mdef\x1b[0m\n", //
                "8. ghi\n"                    //
            )
        );
    }
    #[test]
    fn table() {
        let source = concat!(
            "| Column a |  Column b   |  Column c   |  Column d   |\n",
            "|-----|:----|:---:|----:|\n",
            "|abc  |def  |ghi  |jkl  |\n",
        );
        let layout = parse(source);
        let result = format!("{}", layout.layout(50));

        assert_eq!(
            result,
            concat!(
                "┌─────────────┬──────┬─────────────┬─────────────┐\n",
                "│Column a     │Column│  Column c   │     Column d│\n",
                "│             │b     │             │             │\n",
                "├─────────────┼──────┼─────────────┼─────────────┤\n",
                "│abc          │def   │     ghi     │          jkl│\n",
                "└─────────────┴──────┴─────────────┴─────────────┘\n" //
            )
        );
    }

    #[test]
    fn inline_html() {
        let source = concat!(
            "Inline <p>foo</p>\n",
            "Inline <b>foo</b>\n",
            "Inline <i>foo</i>\n",
            "Inline <u>foo</u>\n",
        );
        let layout = parse(source);
        let result = format!("{}", layout.layout(50));

        assert_eq!(
            result,
            concat!(
                "Inline <p>foo</p> ",
                "Inline \x1b[1mfoo\x1b[0m ",
                "Inline \x1b[3mfoo\x1b[0m ",
                "Inline \x1b[4mfoo\x1b[0m\n" //
            )
        );
    }

    #[test]
    fn hard_break() {
        let source = concat!("A hard\\\n", "break\n",);
        let layout = parse(source);
        let result = format!("{}", layout.layout(50));

        assert_eq!(
            result,
            concat!(
                "A hard\n", //
                "break\n"
            )
        );
    }

    #[test]
    fn rule() {
        let source = concat!(
            "A hard\n", //
            "\n",       //
            "***\n",    //
            "\n",       //
            "break\n",
        );
        let layout = parse(source);
        let result = format!("{}", layout.layout(50));

        assert_eq!(
            result,
            concat!(
                "A hard\n", //
                "──────\n", //
                "break\n"
            )
        );
    }

    #[test]
    fn link() {
        let source = "A [link](https://hipphampel.de) to homepage\n";
        let layout = parse(source);
        let result = format!("{}", layout.layout(50));

        assert_eq!(
            result,
            "A \x1b[36mlink\x1b[0m (➔\x1b[4;34mhttps://hipphampel.de\x1b[0m) to homepage\n"
        );
    }

    #[test]
    fn image() {
        let source = "An ![image](https://hipphampel.de) to homepage\n";
        let layout = parse(source);
        let result = format!("{}", layout.layout(50));

        assert_eq!(
            result,
            "An \x1b[36mimage\x1b[0m (➔\x1b[4;34mhttps://hipphampel.de\x1b[0m) to homepage\n"
        );
    }

    #[test]
    fn show_events() {
        let source = include_str!("../../../examples/doc/EXAMPLE.md");
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_MATH);
        options.insert(Options::ENABLE_GFM);

        let parser = Parser::new_ext(source, options);
        for event in parser {
            println!("{event:?}");
        }
    }
}
