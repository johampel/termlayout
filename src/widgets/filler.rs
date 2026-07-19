use crate::ext::{
    BaseLayoutWriter, BoxedFormattedLayout, BoxedLayoutWriter, DisplayStr, FormattedLayout,
    LayoutWriter, SizedLayoutResult,
};
use crate::ext::{box_formatted_layout, rc_layout};
use crate::{Dimension, Layout, LayoutContext, LayoutOptions, MeasureMode, Measurements};
use std::any::Any;
use std::cmp::min;
use std::fmt::Write;

/// A widget that fills space with a repeating pattern.
///
/// Depending on the [`FillMode`], the fill pattern can be repeated horizontally and/or vertically.
/// The `pattern` must not contain newlines.
///
/// # Examples
/// ```rust
/// use termlayout::*;
/// use termlayout::widgets::Filler;
///
/// let layout = Filler::once("abc");
/// assert_eq!(format!("{}", layout.layout_strict(LayoutOptions::default()
///         .with_dim(Dimension::new(5,3)))),
/// concat!(
///     "abc\n",
///     "\n",
///     "\n",
/// ));
///
/// let layout = Filler::horizontal("abc");
/// assert_eq!(format!("{}", layout.layout_strict(LayoutOptions::default()
///         .with_dim(Dimension::new(5,3)))),
/// concat!(
///     "abcab\n",
///     "\n",
///     "\n",
/// ));
///
/// let layout = Filler::vertical("abc");
/// assert_eq!(format!("{}", layout.layout_strict(LayoutOptions::default()
///         .with_dim(Dimension::new(5,3)))),
/// concat!(
///     "abc\n",
///     "abc\n",
///     "abc\n",
/// ));
///
/// let layout = Filler::both("abc");
/// assert_eq!(format!("{}", layout.layout_strict(LayoutOptions::default()
///         .with_dim(Dimension::new(5,3)))),
/// concat!(
///     "abcab\n",
///     "cabca\n",
///     "bcabc\n",
/// ));
/// ```
pub struct Filler {
    /// The fill pattern
    pub pattern: String,

    /// The [`FillMode`] describing whether and how to repeat the pattern
    pub mode: FillMode,
}

impl Filler {
    /// Creates a [`Filler`] with the given parameters
    ///
    /// # Parameters
    /// - `pattern`: The filler pattern, must not contain newlines
    /// - `mode`: The [`FillMode`]
    ///
    /// # Returns
    /// A new [`Filler`]
    pub fn new<T: Into<String>>(pattern: T, mode: FillMode) -> Self {
        Self {
            pattern: pattern.into(),
            mode,
        }
    }

    /// Creates a [`Filler`] layout that fills the space by printing the given pattern once
    ///
    /// # Parameters
    /// - `pattern`: The filler pattern, must not contain newlines
    ///
    /// # Returns
    /// A new [`Filler`]
    pub fn once<T: Into<String>>(pattern: T) -> Self {
        Self::new(pattern, FillMode::Once)
    }

    /// Creates a [`Filler`] layout that fills the space in vertical direction by repeating the
    /// given `pattern`
    ///
    /// # Parameters
    /// - `pattern`: The filler pattern, must not contain newlines
    ///
    /// # Returns
    /// A new [`Filler`]
    pub fn vertical<T: Into<String>>(pattern: T) -> Self {
        Self::new(pattern, FillMode::Vertical)
    }

    /// Creates a [`Filler`] layout that fills the space in horizontal direction by repeating the
    /// given `pattern`
    ///
    /// # Parameters
    /// - `pattern`: The filler pattern, must not contain newlines
    ///
    /// # Returns
    /// A new [`Filler`]
    pub fn horizontal<T: Into<String>>(pattern: T) -> Self {
        Self::new(pattern, FillMode::Horizontal)
    }

    /// Creates a [`Filler`] layout that fills the space in vertical and horizontal direction
    /// by repeating the given `pattern`
    ///
    /// # Parameters
    /// - `pattern`: The filler pattern, must not contain newlines
    ///
    /// # Returns
    /// A new [`Filler`]
    pub fn both<T: Into<String>>(pattern: T) -> Self {
        Self::new(pattern, FillMode::Both)
    }
}

impl Layout for Filler {
    fn measure(&self, mode: MeasureMode) -> Measurements {
        if mode.is_empty() {
            return Measurements::empty();
        }
        match mode {
            MeasureMode::Min => Dimension::new(self.pattern.display_len(), 1).into(),
            MeasureMode::PrefWidth { max_width,.. } => Dimension::new(min(self.pattern.display_len(), max_width), 1).into(),
            MeasureMode::FixedWidth { width, .. } => Dimension::new(width, 1).into(),
            MeasureMode::Exact { dimension,.. } => dimension.into(),
        }
    }

    fn layout_with_context(&'_ self, context: LayoutContext) -> BoxedFormattedLayout<'_> {
        FormattedFiller::new(&self.pattern, self.mode, context.into()).into()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

rc_layout!(Filler);

/// Defines the different modes the [`Filler`] might apply for filling the space.
///
/// # Variants
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum FillMode {
    /// The pattern is printed only once at the top-left corner.
    Once,

    /// The pattern is repeated vertically to fill the height.
    Vertical,

    /// The pattern is repeated horizontally to fill the width.
    Horizontal,

    /// The pattern is repeated both horizontally and vertically to fill the entire area.
    Both,
}

impl FillMode {
    fn is_vertical(self) -> bool {
        self == FillMode::Both || self == FillMode::Vertical
    }
    fn is_horizontal(self) -> bool {
        self == FillMode::Both || self == FillMode::Horizontal
    }
}

struct FormattedFiller<'fmt> {
    pattern: &'fmt str,
    mode: FillMode,
    options: LayoutOptions,
}

impl<'fmt> FormattedFiller<'fmt> {
    fn new(pattern: &'fmt str, mode: FillMode, options: LayoutOptions) -> Self {
        Self {
            pattern,
            mode,
            options,
        }
    }
}

impl FormattedLayout for FormattedFiller<'_> {
    fn options(&self) -> &LayoutOptions {
        &self.options
    }

    fn new_writer(&'_ self) -> BoxedLayoutWriter<'_> {
        Box::new(FillerWriter::new(self.pattern, self.mode, &self.options))
    }
}

box_formatted_layout!(FormattedFiller);

struct FillerWriter<'wrt> {
    base: BaseLayoutWriter<'wrt>,
    mode: FillMode,
    pattern: &'wrt str,
    next: Option<&'wrt str>,
}

impl<'wrt> FillerWriter<'wrt> {
    fn new(pattern: &'wrt str, mode: FillMode, options: &'wrt LayoutOptions) -> Self {
        Self {
            base: BaseLayoutWriter::new(options),
            mode,
            pattern,
            next: None,
        }
    }

    fn write_part(&mut self, w: &mut dyn Write) -> std::fmt::Result {
        let next = self.next.take().unwrap_or(self.pattern);
        let rest = self.base.write_str(next, self.base.wrap_mode(), w)?;
        if !rest.is_empty() {
            self.next = Some(rest);
        }
        Ok(())
    }

    fn write_line(&mut self, w: &mut dyn Write) -> std::fmt::Result {
        self.write_part(w)?;
        while self.base.available_width() > 0 && self.mode.is_horizontal() {
            self.write_part(w)?;
        }
        Ok(())
    }
}

impl<'wrt> LayoutWriter<'wrt> for FillerWriter<'wrt> {
    fn options(&self) -> &'wrt LayoutOptions {
        self.base.options()
    }

    fn write_row(&mut self, w: &mut dyn Write) -> SizedLayoutResult {
        if !self.pattern.is_empty() && (self.base.row() == 0 || self.mode.is_vertical()) {
            self.write_line(w)?;
        }
        self.base.end_row(w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Rect, WrapMode};

    #[test]
    fn formatted_filler_once_fit() {
        let filler = Filler::once("foo");

        // No clip
        let options = LayoutOptions::default().with_dim(Dimension::new(5, 5));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "foo\n", //
                "\n",    //
                "\n",    //
                "\n",    //
                "\n",    //
            )
        );
        let options = LayoutOptions::default()
            .with_fill_rows(true)
            .with_dim(Dimension::new(5, 5));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "foo  \n", //
                "     \n", //
                "     \n", //
                "     \n", //
                "     \n", //
            )
        );

        // With clip
        let options = LayoutOptions::default()
            .with_dim(Dimension::new(5, 5))
            .with_clip(Some(Rect::new(1, 0, Dimension::new(3, 3))));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "oo\n", //
                "\n",   //
                "\n",   //
            )
        );
        let options = LayoutOptions::default()
            .with_fill_rows(true)
            .with_dim(Dimension::new(5, 5))
            .with_clip(Some(Rect::new(1, 0, Dimension::new(3, 3))));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "oo \n", //
                "   \n", //
                "   \n", //
            )
        );
    }

    #[test]
    fn formatted_filler_once_wrap() {
        let filler = Filler::once("foo");

        // No clip
        let options = LayoutOptions::default().with_dim(Dimension::new(2, 3));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "fo\n", //
                "\n",   //
                "\n",   //
            )
        );
        let options = LayoutOptions::default()
            .with_fill_rows(true)
            .with_dim(Dimension::new(2, 3));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "fo\n", //
                "  \n", //
                "  \n", //
            )
        );

        // With clip
        let options = LayoutOptions::default()
            .with_dim(Dimension::new(2, 3))
            .with_clip(Some(Rect::new(1, 0, Dimension::new(3, 3))));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "o\n", //
                "\n",  //
                "\n",  //
            )
        );
        let options = LayoutOptions::default()
            .with_fill_rows(true)
            .with_dim(Dimension::new(2, 3))
            .with_clip(Some(Rect::new(1, 0, Dimension::new(3, 3))));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "o\n", //
                " \n", //
                " \n", //
            )
        );
    }

    #[test]
    fn formatted_filler_once_truncate() {
        let filler = Filler::once("foo");

        // No clip
        let options = LayoutOptions::default()
            .with_wrap_mode(WrapMode::default_truncate())
            .with_dim(Dimension::new(2, 3));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "f…\n", //
                "\n",   //
                "\n",   //
            )
        );
        let options = LayoutOptions::default()
            .with_wrap_mode(WrapMode::default_truncate())
            .with_fill_rows(true)
            .with_dim(Dimension::new(2, 3));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "f…\n", //
                "  \n", //
                "  \n", //
            )
        );

        // With clip
        let options = LayoutOptions::default()
            .with_wrap_mode(WrapMode::default_truncate())
            .with_dim(Dimension::new(2, 3))
            .with_clip(Some(Rect::new(1, 0, Dimension::new(3, 3))));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "…\n", //
                "\n",  //
                "\n",  //
            )
        );
        let options = LayoutOptions::default()
            .with_wrap_mode(WrapMode::default_truncate())
            .with_fill_rows(true)
            .with_dim(Dimension::new(2, 3))
            .with_clip(Some(Rect::new(1, 0, Dimension::new(3, 3))));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "…\n", //
                " \n", //
                " \n", //
            )
        );
    }

    #[test]
    fn formatted_filler_horizontal_fit() {
        let filler = Filler::horizontal("foo");

        // No clip
        let options = LayoutOptions::default().with_dim(Dimension::new(5, 5));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "foofo\n", //
                "\n",      //
                "\n",      //
                "\n",      //
                "\n",      //
            )
        );
        let options = LayoutOptions::default()
            .with_fill_rows(true)
            .with_dim(Dimension::new(5, 5));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "foofo\n", //
                "     \n", //
                "     \n", //
                "     \n", //
                "     \n", //
            )
        );

        // With clip
        let options = LayoutOptions::default()
            .with_dim(Dimension::new(5, 5))
            .with_clip(Some(Rect::new(1, 0, Dimension::new(3, 3))));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "oof\n", //
                "\n",    //
                "\n",    //
            )
        );
        let options = LayoutOptions::default()
            .with_fill_rows(true)
            .with_dim(Dimension::new(5, 5))
            .with_clip(Some(Rect::new(1, 0, Dimension::new(3, 3))));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "oof\n", //
                "   \n", //
                "   \n", //
            )
        );
    }

    #[test]
    fn formatted_filler_horizontal_wrap() {
        let filler = Filler::horizontal("foo");

        // No clip
        let options = LayoutOptions::default().with_dim(Dimension::new(2, 3));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "fo\n", //
                "\n",   //
                "\n",   //
            )
        );
        let options = LayoutOptions::default()
            .with_fill_rows(true)
            .with_dim(Dimension::new(2, 3));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "fo\n", //
                "  \n", //
                "  \n", //
            )
        );

        // With clip
        let options = LayoutOptions::default()
            .with_dim(Dimension::new(2, 3))
            .with_clip(Some(Rect::new(1, 0, Dimension::new(3, 3))));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "o\n", //
                "\n",  //
                "\n",  //
            )
        );
        let options = LayoutOptions::default()
            .with_fill_rows(true)
            .with_dim(Dimension::new(2, 3))
            .with_clip(Some(Rect::new(1, 0, Dimension::new(3, 3))));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "o\n", //
                " \n", //
                " \n", //
            )
        );
    }

    #[test]
    fn formatted_filler_horizontal_truncate() {
        let filler = Filler::horizontal("foo");

        // No clip
        let options = LayoutOptions::default()
            .with_wrap_mode(WrapMode::default_truncate())
            .with_dim(Dimension::new(2, 3));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "f…\n", //
                "\n",   //
                "\n",   //
            )
        );
        let options = LayoutOptions::default()
            .with_wrap_mode(WrapMode::default_truncate())
            .with_fill_rows(true)
            .with_dim(Dimension::new(2, 3));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "f…\n", //
                "  \n", //
                "  \n", //
            )
        );

        // With clip
        let options = LayoutOptions::default()
            .with_wrap_mode(WrapMode::default_truncate())
            .with_dim(Dimension::new(2, 3))
            .with_clip(Some(Rect::new(1, 0, Dimension::new(3, 3))));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "…\n", //
                "\n",  //
                "\n",  //
            )
        );
        let options = LayoutOptions::default()
            .with_wrap_mode(WrapMode::default_truncate())
            .with_fill_rows(true)
            .with_dim(Dimension::new(2, 3))
            .with_clip(Some(Rect::new(1, 0, Dimension::new(3, 3))));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "…\n", //
                " \n", //
                " \n", //
            )
        );
    }

    #[test]
    fn formatted_filler_vertical_fit() {
        let filler = Filler::vertical("foo");

        // No clip
        let options = LayoutOptions::default().with_dim(Dimension::new(5, 5));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "foo\n", //
                "foo\n", //
                "foo\n", //
                "foo\n", //
                "foo\n", //
            )
        );
        let options = LayoutOptions::default()
            .with_fill_rows(true)
            .with_dim(Dimension::new(5, 5));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "foo  \n", //
                "foo  \n", //
                "foo  \n", //
                "foo  \n", //
                "foo  \n", //
            )
        );

        // With clip
        let options = LayoutOptions::default()
            .with_dim(Dimension::new(5, 5))
            .with_clip(Some(Rect::new(1, 1, Dimension::new(3, 3))));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "oo\n", //
                "oo\n", //
                "oo\n", //
            )
        );
        let options = LayoutOptions::default()
            .with_fill_rows(true)
            .with_dim(Dimension::new(5, 5))
            .with_clip(Some(Rect::new(1, 1, Dimension::new(3, 3))));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "oo \n", //
                "oo \n", //
                "oo \n", //
            )
        );
    }

    #[test]
    fn formatted_filler_vertical_wrap() {
        let filler = Filler::vertical("foo");

        // No clip
        let options = LayoutOptions::default().with_dim(Dimension::new(2, 3));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "fo\n", //
                "o\n",  //
                "fo\n", //
            )
        );
        let options = LayoutOptions::default()
            .with_fill_rows(true)
            .with_dim(Dimension::new(2, 3));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "fo\n", //
                "o \n", //
                "fo\n", //
            )
        );

        // With clip
        let options = LayoutOptions::default()
            .with_dim(Dimension::new(2, 3))
            .with_clip(Some(Rect::new(1, 1, Dimension::new(3, 3))));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "\n",  //
                "o\n", //
            )
        );
        let options = LayoutOptions::default()
            .with_fill_rows(true)
            .with_dim(Dimension::new(2, 3))
            .with_clip(Some(Rect::new(1, 1, Dimension::new(3, 3))));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                " \n", //
                "o\n", //
            )
        );
    }

    #[test]
    fn formatted_filler_vertical_truncate() {
        let filler = Filler::vertical("foo");

        // No clip
        let options = LayoutOptions::default()
            .with_wrap_mode(WrapMode::default_truncate())
            .with_dim(Dimension::new(2, 3));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "f…\n", //
                "f…\n", //
                "f…\n", //
            )
        );
        let options = LayoutOptions::default()
            .with_wrap_mode(WrapMode::default_truncate())
            .with_fill_rows(true)
            .with_dim(Dimension::new(2, 3));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "f…\n", //
                "f…\n", //
                "f…\n", //
            )
        );

        // With clip
        let options = LayoutOptions::default()
            .with_wrap_mode(WrapMode::default_truncate())
            .with_dim(Dimension::new(2, 3))
            .with_clip(Some(Rect::new(1, 0, Dimension::new(3, 3))));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "…\n", //
                "…\n", //
                "…\n", //
            )
        );
        let options = LayoutOptions::default()
            .with_wrap_mode(WrapMode::default_truncate())
            .with_fill_rows(true)
            .with_dim(Dimension::new(2, 3))
            .with_clip(Some(Rect::new(1, 1, Dimension::new(3, 3))));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "…\n", //
                "…\n", //
            )
        );
    }

    #[test]
    fn formatted_filler_both_fit() {
        let filler = Filler::both("foo");

        // No clip
        let options = LayoutOptions::default().with_dim(Dimension::new(5, 5));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "foofo\n", //
                "ofoof\n", //
                "oofoo\n", //
                "foofo\n", //
                "ofoof\n", //
            )
        );
        let options = LayoutOptions::default()
            .with_fill_rows(true)
            .with_dim(Dimension::new(5, 5));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "foofo\n", //
                "ofoof\n", //
                "oofoo\n", //
                "foofo\n", //
                "ofoof\n", //
            )
        );

        // With clip
        let options = LayoutOptions::default()
            .with_dim(Dimension::new(5, 5))
            .with_clip(Some(Rect::new(1, 1, Dimension::new(3, 3))));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "foo\n", //
                "ofo\n", //
                "oof\n", //
            )
        );
        let options = LayoutOptions::default()
            .with_fill_rows(true)
            .with_dim(Dimension::new(5, 5))
            .with_clip(Some(Rect::new(1, 1, Dimension::new(3, 3))));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "foo\n", //
                "ofo\n", //
                "oof\n", //
            )
        );
    }

    #[test]
    fn formatted_filler_both_wrap() {
        let filler = Filler::both("foo");

        // No clip
        let options = LayoutOptions::default().with_dim(Dimension::new(2, 3));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "fo\n", //
                "of\n", //
                "oo\n", //
            )
        );
        let options = LayoutOptions::default()
            .with_fill_rows(true)
            .with_dim(Dimension::new(2, 3));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "fo\n", //
                "of\n", //
                "oo\n", //
            )
        );

        // With clip
        let options = LayoutOptions::default()
            .with_dim(Dimension::new(2, 3))
            .with_clip(Some(Rect::new(1, 1, Dimension::new(3, 3))));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "f\n", //
                "o\n", //
            )
        );
        let options = LayoutOptions::default()
            .with_fill_rows(true)
            .with_dim(Dimension::new(2, 3))
            .with_clip(Some(Rect::new(1, 1, Dimension::new(3, 3))));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "f\n", //
                "o\n", //
            )
        );
    }

    #[test]
    fn formatted_filler_both_truncate() {
        let filler = Filler::both("foo");

        // No clip
        let options = LayoutOptions::default()
            .with_wrap_mode(WrapMode::default_truncate())
            .with_dim(Dimension::new(2, 3));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "f…\n", //
                "f…\n", //
                "f…\n", //
            )
        );
        let options = LayoutOptions::default()
            .with_wrap_mode(WrapMode::default_truncate())
            .with_fill_rows(true)
            .with_dim(Dimension::new(2, 3));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "f…\n", //
                "f…\n", //
                "f…\n", //
            )
        );

        // With clip
        let options = LayoutOptions::default()
            .with_wrap_mode(WrapMode::default_truncate())
            .with_dim(Dimension::new(2, 3))
            .with_clip(Some(Rect::new(1, 0, Dimension::new(3, 3))));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "…\n", //
                "…\n", //
                "…\n", //
            )
        );
        let options = LayoutOptions::default()
            .with_wrap_mode(WrapMode::default_truncate())
            .with_fill_rows(true)
            .with_dim(Dimension::new(2, 3))
            .with_clip(Some(Rect::new(1, 1, Dimension::new(3, 3))));
        assert_eq!(
            format!("{}", filler.layout_strict(options)),
            concat!(
                "…\n", //
                "…\n", //
            )
        );
    }

    #[test]
    fn filler_empty_pattern() {
        let filler = Filler::both("");
        let options = LayoutOptions::default().with_dim(Dimension::new(5, 2));
        assert_eq!(format!("{}", filler.layout_strict(options)), "\n\n");
    }

    #[test]
    fn filler_empty_dimensions() {
        let filler = Filler::both("foo");
        let options = LayoutOptions::default().with_dim(Dimension::empty());
        assert_eq!(format!("{}", filler.layout_strict(options)), "");
    }

    #[test]
    fn filler_measure_min() {
        let filler = Filler::both("foobar");
        assert_eq!(filler.measure(MeasureMode::min()).dim, Dimension::new(6, 1));
    }

    #[test]
    fn filler_measure_pref() {
        let filler = Filler::both("foobar");
        assert_eq!(filler.measure(MeasureMode::pref_width(10, WrapMode::default())).dim, Dimension::new(6, 1));
    }

    #[test]
    fn filler_measure_fixed_width() {
        let filler = Filler::both("foobar");
        assert_eq!(filler.measure(MeasureMode::fixed_width(10, WrapMode::default())).dim, Dimension::new(10, 1));
    }

    #[test]
    fn filler_measure_exact() {
        let filler = Filler::both("foobar");
        assert_eq!(filler.measure(MeasureMode::exact(Dimension::new(5,3), WrapMode::default())).dim, Dimension::new(5, 3));
    }
}
