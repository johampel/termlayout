use crate::RcLayout;
use crate::ext::{Style, TextBuilder};
use crate::widgets::markdown::config::FrameConfig;
use crate::widgets::{Frame, FrameDecoration, Lines, List, ListItemMarker, Paragraph, Vertical};

#[derive(Default)]
pub(crate) struct LayoutBuffer {
    default_style: Style,
    text: TextBuilder,
    items: Vec<RcLayout>,
}

impl LayoutBuffer {
    pub(crate) fn with_default_style(default_style: Style) -> Self {
        Self {
            default_style,
            ..Default::default()
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.items.is_empty() && self.text.is_empty()
    }

    pub(crate) fn append_layout(&mut self, item: RcLayout) {
        self.items.push(item);
    }

    pub(crate) fn append_layouts(&mut self, items: &[RcLayout]) {
        self.items.extend(items.iter().cloned());
    }

    pub(crate) fn append_newline_unless_empty(&mut self) {
        if !self.is_empty()
            && let Some(last) = self.items.last_mut()
        {
            if let Some(lines) = last.as_any().downcast_ref::<Lines>()
                && lines.content == "\n"
            {
                return;
            }
            self.append_layout(Lines::left("\n").into());
        }
    }

    pub(crate) fn append_styled_text<S, T>(&mut self, style_change: S, text: T)
    where
        S: FnOnce(Style) -> Style,
        T: AsRef<str>,
    {
        self.text.push_style_change(style_change);
        self.text.append(text);
        self.text.pop_last_style();
    }

    pub(crate) fn append_text<T>(&mut self, text: T)
    where
        T: AsRef<str>,
    {
        self.text.append(text);
    }

    pub(crate) fn push_style_change<T>(&mut self, change: T)
    where
        T: FnOnce(Style) -> Style,
    {
        self.text.push_style_change(change);
    }

    pub(crate) fn pop_style(&mut self) {
        self.text.pop_last_style();
    }

    pub(crate) fn flush_text<T>(&mut self, partial: bool, layout_producer: T)
    where
        T: FnOnce(&str) -> RcLayout,
    {
        let content = if partial {
            self.text.partial_flush()
        } else {
            self.text.flush()
        };
        if self.default_style != Style::default() && !partial {
            self.text.push_style(self.default_style);
        }
        if !content.is_empty() {
            self.items.push(layout_producer(content.as_str()));
        }
    }

    pub(crate) fn flush_text_as_paragraph(&mut self, partial: bool) {
        self.flush_text(partial, |s| Paragraph::left(s).into());
    }

    #[must_use]
    pub(crate) fn flush<T>(&mut self, layout_producer: T) -> Vec<RcLayout>
    where
        T: FnOnce(&str) -> RcLayout,
    {
        self.flush_text(false, layout_producer);
        std::mem::take(&mut self.items)
    }

    #[must_use]
    pub(crate) fn flush_as_vertical<T>(&mut self, layout_producer: T) -> RcLayout
    where
        T: FnOnce(&str) -> RcLayout,
    {
        let result = self.flush(layout_producer);
        if result.len() == 1 {
            result[0].clone()
        } else {
            Vertical::new(result).into()
        }
    }

    #[must_use]
    pub(crate) fn flush_as_paragraph(&mut self) -> Vec<RcLayout> {
        self.flush(|s| Paragraph::left(s).into())
    }

    #[must_use]
    pub(crate) fn flush_as_frame_with_lines(&mut self, config: &FrameConfig) -> RcLayout {
        let content =
            self.flush_as_vertical(|s| Lines::left_with_style(config.initial_style, s).into());
        Frame::new(
            FrameDecoration::from_spec(config.frame_spec.as_ref()).unwrap(),
            None,
            content,
        )
        .into()
    }

    #[must_use]
    pub(crate) fn flush_as_frame_with_paragraph(&mut self, config: &FrameConfig) -> RcLayout {
        let content =
            self.flush_as_vertical(|s| Paragraph::left_with_style(config.initial_style, s).into());
        Frame::new(
            FrameDecoration::from_spec(config.frame_spec.as_ref()).unwrap(),
            None,
            content,
        )
        .into()
    }

    #[must_use]
    pub(crate) fn flush_as_list(&mut self, item_marker: ListItemMarker) -> RcLayout {
        self.flush_text_as_paragraph(false);
        List::new(item_marker, std::mem::take(&mut self.items)).into()
    }
}
