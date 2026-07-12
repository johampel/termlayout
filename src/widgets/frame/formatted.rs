use crate::ext::{
    BaseLayoutWriter, BoxedLayoutWriter, DisplayStr, FormattedLayout, LayoutWriter,
    SizedLayoutResult, Style, VoidLayoutResult,
};
use crate::widgets::FrameDecoration;
use crate::widgets::frame::decoration::FrameDecorationKey;
use crate::{BoxedFormattedLayout, LayoutOptions, WrapMode, box_formatted_layout};
use std::cmp::min;
use std::fmt::Write;

pub(crate) struct FormattedFrame<'fmt> {
    content: BoxedFormattedLayout<'fmt>,
    title: Option<&'fmt str>,
    decoration: &'fmt FrameDecoration,
    options: LayoutOptions,
}

impl<'fmt> FormattedFrame<'fmt> {
    pub(crate) fn new(
        content: BoxedFormattedLayout<'fmt>,
        title: Option<&'fmt str>,
        decoration: &'fmt FrameDecoration,
        options: LayoutOptions,
    ) -> Self {
        Self {
            content,
            title,
            decoration,
            options,
        }
    }
}

impl FormattedLayout for FormattedFrame<'_> {
    fn options(&self) -> &LayoutOptions {
        &self.options
    }

    fn new_writer(&'_ self) -> BoxedLayoutWriter<'_> {
        Box::new(FrameWriter::new(
            self.content.new_writer(),
            self.title,
            self.decoration,
            &self.options,
        ))
    }
}

box_formatted_layout!(FormattedFrame);

struct FrameWriter<'wrt> {
    base: BaseLayoutWriter<'wrt>,
    content: BoxedLayoutWriter<'wrt>,
    title: Option<&'wrt str>,
    decoration: &'wrt FrameDecoration,
    has_north_frame: bool,
    has_south_frame: bool,
    left_margin: usize,
    right_margin: usize,
    content_width: usize,
}

impl<'wrt> FrameWriter<'wrt> {
    fn new(
        content: BoxedLayoutWriter<'wrt>,
        title: Option<&'wrt str>,
        decoration: &'wrt FrameDecoration,
        options: &'wrt LayoutOptions,
    ) -> Self {
        let max_width = options.dim.width;
        let content_width = content.options().visible_rect().dim.width;
        let left_margin = min(
            max_width.saturating_sub(content_width),
            decoration.get_left_margin(),
        );
        let right_margin = min(
            max_width.saturating_sub(content_width + left_margin),
            decoration.get_right_margin(),
        );
        let has_north_frame = decoration.has_frame(FrameDecorationKey::is_north);
        let has_south_frame = decoration.has_frame(FrameDecorationKey::is_south);

        Self {
            base: BaseLayoutWriter::new(options),
            title,
            content,
            decoration,
            has_north_frame,
            has_south_frame,
            left_margin,
            right_margin,
            content_width,
        }
    }

    fn is_top_frame_row(&self) -> bool {
        self.has_north_frame && self.base.row() == 0
    }

    fn is_bottom_frame_row(&self) -> bool {
        self.has_south_frame && self.base.row() + 1 == self.base.max_height()
    }

    fn is_title_row(&self) -> bool {
        if self.title.is_none() {
            return false;
        }
        let row = self.base.row();
        match (
            self.decoration.title_placement.bottom,
            self.decoration.title_placement.inside,
        ) {
            // At the top, in frame
            (false, false) => row == 0,
            // At top, inside ->
            (false, true) => row == usize::from(self.has_north_frame),
            // At bottom, inside
            (true, true) => row + 1 + usize::from(self.has_south_frame) == self.base.max_height(),
            // At bottom, in frame ->
            (true, false) => row + usize::from(self.has_south_frame) == self.base.max_height(),
        }
    }

    fn write_content_row(&mut self, w: &mut dyn Write) -> VoidLayoutResult {
        self.write_frame_decoration(FrameDecorationKey::West, self.left_margin, w)?;
        self.base.write_row(self.content.as_mut(), w)?;
        self.write_frame_decoration(FrameDecorationKey::East, self.right_margin, w)?;
        self.base.write_style(Style::default(), w)
    }

    fn write_title_row(&mut self, w: &mut dyn Write) -> VoidLayoutResult {
        let title = self.title.unwrap();
        let title_len = min(title.display_len(), self.content_width);
        let title_left_margin = self
            .decoration
            .title_placement
            .alignment
            .indent(self.content_width.saturating_sub(title_len));
        let title_right_margin = self
            .content_width
            .saturating_sub(title_len + title_left_margin);

        let (left, filler, right) = if self.is_top_frame_row() {
            (
                FrameDecorationKey::NorthWest,
                self.first_utf8_char(FrameDecorationKey::North),
                FrameDecorationKey::NorthEast,
            )
        } else if self.is_bottom_frame_row() {
            (
                FrameDecorationKey::SouthWest,
                self.first_utf8_char(FrameDecorationKey::South),
                FrameDecorationKey::SouthEast,
            )
        } else {
            (FrameDecorationKey::West, ' ', FrameDecorationKey::East)
        };

        self.write_frame_decoration(left, self.left_margin, w)?;
        self.base.write_repeated(filler, title_left_margin, w)?;

        self.base
            .write_str_with_len(title, title_len, WrapMode::default_truncate(), w)?;

        self.base.write_repeated(filler, title_right_margin, w)?;
        self.write_frame_decoration(right, self.right_margin, w)?;
        self.base.write_style(Style::default(), w)
    }

    fn write_top_frame_row(&mut self, w: &mut dyn Write) -> VoidLayoutResult {
        self.write_frame_decoration(FrameDecorationKey::NorthWest, self.left_margin, w)?;
        self.base.write_repeated(
            self.first_utf8_char(FrameDecorationKey::North),
            self.content.options().visible_rect().dim.width,
            w,
        )?;
        self.write_frame_decoration(FrameDecorationKey::NorthEast, self.right_margin, w)?;
        self.base.write_style(Style::default(), w)
    }

    fn write_bottom_frame_row(&mut self, w: &mut dyn Write) -> VoidLayoutResult {
        self.write_frame_decoration(FrameDecorationKey::SouthWest, self.left_margin, w)?;
        self.base.write_repeated(
            self.first_utf8_char(FrameDecorationKey::South),
            self.content.options().visible_rect().dim.width,
            w,
        )?;
        self.write_frame_decoration(FrameDecorationKey::SouthEast, self.right_margin, w)?;
        self.base.write_style(Style::default(), w)
    }

    fn write_frame_decoration(
        &mut self,
        key: FrameDecorationKey,
        width: usize,
        w: &mut dyn Write,
    ) -> VoidLayoutResult {
        if let Some(decoration) = self.decoration.get_frame(key) {
            let width = min(width, decoration.display_len());
            self.base.write_str(
                decoration.display_slice(..width),
                WrapMode::empty_truncate(),
                w,
            )?;
        }
        Ok(())
    }

    fn first_utf8_char(&self, key: FrameDecorationKey) -> char {
        if let Some(decoration) = self.decoration.get_frame(key) {
            return decoration.chars().next().unwrap_or(' ');
        }
        ' '
    }
}

impl<'wrt> LayoutWriter<'wrt> for FrameWriter<'wrt> {
    fn options(&self) -> &'wrt LayoutOptions {
        self.base.options()
    }

    fn write_row(&mut self, w: &mut dyn Write) -> SizedLayoutResult {
        if self.is_title_row() {
            self.write_title_row(w)?;
        } else if self.is_top_frame_row() {
            self.write_top_frame_row(w)?;
        } else if self.is_bottom_frame_row() {
            self.write_bottom_frame_row(w)?;
        } else {
            self.write_content_row(w)?;
        }
        self.base.end_row(w)
    }
}
