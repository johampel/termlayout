use crate::ext::DisplayStr;
use crate::widgets::FrameDecoration;
use crate::widgets::frame::formatted::FormattedFrame;
use crate::{
    BoxedFormattedLayout, Dimension, Layout, LayoutOptions, RcLayout, Rect, WrapMode, rc_layout,
};
use std::any::Any;
use std::cmp::max;

pub(crate) mod decoration;
mod formatted;

/// A widget that decorates content with a border frame and optional title.
///
/// Besides the frame around the content, it can display a title that can be placed at the
/// top or bottom of the frame, either inside or integrated into the border.
///
/// # Example
/// ```rust
/// use termlayout::Layout;
/// use termlayout::widgets::{Frame, FrameDecoration, Paragraph};
///
/// let content = Paragraph::left("This is the content.");
/// let decoration = FrameDecoration::boxed();
///
/// let frame = Frame::new(decoration, Some("Title".into()), content);
///
/// assert_eq!(format!("{}", frame.layout(15)), concat!(
///     "┌Title──────┐\n",
///     "│This is the│\n",
///     "│content.   │\n",
///     "└───────────┘\n"));
/// ```
pub struct Frame {
    /// The decoration of the frame, represented as a [`FrameDecoration`].
    /// This contains the frame itself plus the information about the title placement
    pub decoration: FrameDecoration,

    /// The title of the frame or `None` if no title is present.
    pub title: Option<String>,

    /// The content of the frame, represented as a [`RcLayout`].
    pub content: RcLayout,
}

impl Frame {
    /// Creates a new instance.
    ///
    /// # Parameters
    /// - `decoration`: The [`FrameDecoration`]
    /// - `title`: The title, if any
    /// - `content`: The content of the frame, represented as a [`RcLayout`]
    ///
    /// # Returns
    /// A new instance of [`Frame`]
    pub fn new<T, C>(decoration: FrameDecoration, title: T, content: C) -> Self
    where
        T: Into<Option<String>>,
        C: Into<RcLayout>,
    {
        Self {
            decoration,
            title: title.into(),
            content: content.into(),
        }
    }
}

impl Layout for Frame {
    fn pref_dim(&self, max_width: usize, wrap_mode: WrapMode) -> Dimension {
        if max_width == 0 {
            return Dimension::empty();
        }
        let hmargin = self.decoration.get_left_margin() + self.decoration.get_right_margin();
        let content_width = max(1, max_width.saturating_sub(hmargin));
        let dim = self.content.pref_dim(content_width, wrap_mode);
        self.decoration.frame_dim(dim, self.title.is_some())
    }

    fn min_dim(&self) -> Dimension {
        let mut dim = self.content.min_dim();
        if let Some(title) = &self.title {
            dim.width = max(title.display_len(), dim.width);
        }
        self.decoration.frame_dim(dim, self.title.is_some())
    }

    fn layout_strict(&'_ self, options: LayoutOptions) -> BoxedFormattedLayout<'_> {
        let (left_margin, right_margin) = (
            self.decoration.get_left_margin(),
            self.decoration.get_right_margin(),
        );
        let (top_margin, bottom_margin) = (
            self.decoration.get_top_margin(self.title.is_some()),
            self.decoration.get_bottom_margin(self.title.is_some()),
        );
        let content_rect = Rect::new(
            left_margin,
            top_margin,
            Dimension::new(
                options.dim.width.saturating_sub(left_margin + right_margin),
                options
                    .dim
                    .height
                    .saturating_sub(top_margin + bottom_margin),
            ),
        );
        let mut content_options = options.intersect(content_rect, false);
        content_options.fill_rows = right_margin > 0;
        let formatted = self.content.layout_strict(content_options);
        FormattedFrame::new(formatted, self.title.as_deref(), &self.decoration, options).into()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

rc_layout!(Frame);

#[cfg(test)]
mod tests {
    use crate::widgets::frame::decoration::TitlePlacement;
    use crate::widgets::frame::*;
    use crate::widgets::{Lines, LinesAlignment};

    #[test]
    fn frame_min_dim() {
        let mut frame = Frame::new(
            FrameDecoration::boxed(),
            Some("Title".into()),
            Lines::left("abcdefghijklmnopqrstuvwxyz\n0123456789"),
        );

        assert_eq!(frame.min_dim(), Dimension::new(28, 4));

        frame.decoration.title_placement = TitlePlacement::default().with_inside(true);
        assert_eq!(frame.min_dim(), Dimension::new(28, 5));
    }

    #[test]
    fn frame_layout_fit_no_clip() {
        let mut frame = Frame::new(
            FrameDecoration::boxed(),
            Some("Title".into()),
            Lines::left("abcdefghijklmnopqrstuvwxyz\n0123456789"),
        );

        // Title top in frame
        frame.decoration.title_placement = TitlePlacement::default()
            .with_alignment(LinesAlignment::Left)
            .with_inside(false)
            .with_bottom(false);
        assert_eq!(
            format!("{}", frame.layout(30)),
            concat!(
                "┌Title─────────────────────┐\n",
                "│abcdefghijklmnopqrstuvwxyz│\n",
                "│0123456789                │\n",
                "└──────────────────────────┘\n"
            )
        );

        // Title top inside
        frame.decoration.title_placement = TitlePlacement::default()
            .with_alignment(LinesAlignment::Center)
            .with_inside(true)
            .with_bottom(false);
        assert_eq!(
            format!("{}", frame.layout(30)),
            concat!(
                "┌──────────────────────────┐\n",
                "│          Title           │\n",
                "│abcdefghijklmnopqrstuvwxyz│\n",
                "│0123456789                │\n",
                "└──────────────────────────┘\n"
            )
        );

        // Title bottom inside
        frame.decoration.title_placement = TitlePlacement::default()
            .with_alignment(LinesAlignment::Right)
            .with_inside(true)
            .with_bottom(true);
        assert_eq!(
            format!("{}", frame.layout(30)),
            concat!(
                "┌──────────────────────────┐\n",
                "│abcdefghijklmnopqrstuvwxyz│\n",
                "│0123456789                │\n",
                "│                     Title│\n",
                "└──────────────────────────┘\n"
            )
        );

        // Title bottom in frame
        frame.decoration.title_placement = TitlePlacement::default()
            .with_alignment(LinesAlignment::Center)
            .with_inside(false)
            .with_bottom(true);
        assert_eq!(
            format!("{}", frame.layout(30)),
            concat!(
                "┌──────────────────────────┐\n",
                "│abcdefghijklmnopqrstuvwxyz│\n",
                "│0123456789                │\n",
                "└──────────Title───────────┘\n"
            )
        );
    }

    #[test]
    fn frame_layout_truncate_no_clip() {
        let mut frame = Frame::new(
            FrameDecoration::boxed(),
            Some("Title".into()),
            Lines::left("abcdefghijklmnopqrstuvwxyz\n0123456789"),
        );

        // Title top in frame
        frame.decoration.title_placement = TitlePlacement::default()
            .with_alignment(LinesAlignment::Left)
            .with_inside(false)
            .with_bottom(false);
        assert_eq!(
            format!(
                "{}",
                frame.layout_with_wrap_mode(20, WrapMode::default_truncate())
            ),
            concat!(
                "┌Title─────────────┐\n",
                "│abcdefghijklmnopq…│\n",
                "│0123456789        │\n",
                "└──────────────────┘\n"
            )
        );

        // Title top inside
        frame.decoration.title_placement = TitlePlacement::default()
            .with_alignment(LinesAlignment::Center)
            .with_inside(true)
            .with_bottom(false);
        assert_eq!(
            format!(
                "{}",
                frame.layout_with_wrap_mode(20, WrapMode::default_truncate())
            ),
            concat!(
                "┌──────────────────┐\n",
                "│      Title       │\n",
                "│abcdefghijklmnopq…│\n",
                "│0123456789        │\n",
                "└──────────────────┘\n"
            )
        );

        // Title bottom inside
        frame.decoration.title_placement = TitlePlacement::default()
            .with_alignment(LinesAlignment::Right)
            .with_inside(true)
            .with_bottom(true);
        assert_eq!(
            format!(
                "{}",
                frame.layout_with_wrap_mode(20, WrapMode::default_truncate())
            ),
            concat!(
                "┌──────────────────┐\n",
                "│abcdefghijklmnopq…│\n",
                "│0123456789        │\n",
                "│             Title│\n",
                "└──────────────────┘\n"
            )
        );

        // Title bottom in frame
        frame.decoration.title_placement = TitlePlacement::default()
            .with_alignment(LinesAlignment::Center)
            .with_inside(false)
            .with_bottom(true);
        assert_eq!(
            format!(
                "{}",
                frame.layout_with_wrap_mode(20, WrapMode::default_truncate())
            ),
            concat!(
                "┌──────────────────┐\n",
                "│abcdefghijklmnopq…│\n",
                "│0123456789        │\n",
                "└──────Title───────┘\n"
            )
        );
    }

    #[test]
    fn frame_layout_wrap_no_clip() {
        let mut frame = Frame::new(
            FrameDecoration::boxed(),
            Some("Title".into()),
            Lines::left("abcdefghijklmnopqrstuvwxyz\n0123456789"),
        );

        // Title top in frame
        frame.decoration.title_placement = TitlePlacement::default()
            .with_alignment(LinesAlignment::Left)
            .with_inside(false)
            .with_bottom(false);
        assert_eq!(
            format!("{}", frame.layout(20)),
            concat!(
                "┌Title─────────────┐\n",
                "│abcdefghijklmnopqr│\n",
                "│stuvwxyz          │\n",
                "│0123456789        │\n",
                "└──────────────────┘\n"
            )
        );

        // Title top inside
        frame.decoration.title_placement = TitlePlacement::default()
            .with_alignment(LinesAlignment::Center)
            .with_inside(true)
            .with_bottom(false);
        assert_eq!(
            format!("{}", frame.layout(20)),
            concat!(
                "┌──────────────────┐\n",
                "│      Title       │\n",
                "│abcdefghijklmnopqr│\n",
                "│stuvwxyz          │\n",
                "│0123456789        │\n",
                "└──────────────────┘\n"
            )
        );

        // Title bottom inside
        frame.decoration.title_placement = TitlePlacement::default()
            .with_alignment(LinesAlignment::Right)
            .with_inside(true)
            .with_bottom(true);
        assert_eq!(
            format!("{}", frame.layout(20)),
            concat!(
                "┌──────────────────┐\n",
                "│abcdefghijklmnopqr│\n",
                "│stuvwxyz          │\n",
                "│0123456789        │\n",
                "│             Title│\n",
                "└──────────────────┘\n"
            )
        );

        // Title bottom in frame
        frame.decoration.title_placement = TitlePlacement::default()
            .with_alignment(LinesAlignment::Center)
            .with_inside(false)
            .with_bottom(true);
        assert_eq!(
            format!("{}", frame.layout(20)),
            concat!(
                "┌──────────────────┐\n",
                "│abcdefghijklmnopqr│\n",
                "│stuvwxyz          │\n",
                "│0123456789        │\n",
                "└──────Title───────┘\n"
            )
        );
    }

    #[test]
    fn frame_layout_fit_with_clip() {
        let mut frame = Frame::new(
            FrameDecoration::boxed(),
            Some("Title".into()),
            Lines::left("abcdefghijklm\nnopqrstu\nvwxyz\n0123456789"),
        );

        frame.decoration.title_placement = TitlePlacement::default()
            .with_alignment(LinesAlignment::Left)
            .with_inside(false)
            .with_bottom(false);
        let options = LayoutOptions::default()
            .with_dim(Dimension::new(15, 6))
            .with_clip(Some(Rect::new(1, 2, Dimension::new(10, 3))));
        assert_eq!(
            format!("{}", frame.layout_strict(options)),
            concat!("nopqrstu  \n", "vwxyz     \n", "0123456789\n",)
        );
    }
}
