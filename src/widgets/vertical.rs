use crate::core::measurements::MeasurementSpecifics;
use crate::ext::{
    BaseLayoutWriter, BoxedLayoutWriter, FormattedLayout, LayoutWriter, SizedLayoutResult,
};
use crate::ext::{BoxedFormattedLayout, box_formatted_layout, rc_layout};
use crate::{
    Dimension, Layout, LayoutContext, LayoutOptions, MeasureMode, Measurements, RcLayout,
    WrapMode,
};
use std::any::Any;
use std::fmt::Write;

/// A widget that arranges multiple layouts vertically.
///
/// This composing widget stacks its children vertically, where each child is an
/// independent layout that can be any other widget type.
///
/// # Example
/// ```rust
/// use termlayout::*;
/// use termlayout::widgets::Lines;
/// use termlayout::widgets::Vertical;
///
/// let vertical = Vertical::new(vec![
///     Lines::left("This is a line."),
///     Lines::center("This is another line."),
///     Lines::right("This is a third and fourth\n line.")]);
///
/// assert_eq!(format!("{}", vertical.layout(30)), concat!(
///     "This is a line.\n",
///     "  This is another line.\n",
///     "This is a third and fourth\n",
///     "                     line.\n"));
/// ```
pub struct Vertical {
    /// The vector of independent layouts to be displayed vertically.
    pub content: Vec<RcLayout>,
}

impl Vertical {
    /// Creates a new vertical layout with the given content.
    ///
    /// # Parameters
    /// - `content`: The content to be laid out vertically.
    ///
    /// # Returns
    /// The new instance
    #[must_use]
    pub fn new<T>(content: Vec<T>) -> Self
    where
        T: Into<RcLayout>,
    {
        Self {
            content: content.into_iter().map(Into::into).collect(),
        }
    }

    fn measure_exact(&self, dimension: Dimension, wrap_mode: WrapMode) -> Measurements {
        let mut height = dimension.height;
        let mut children = Vec::with_capacity(self.content.len());
        for (index, item) in self.content.iter().enumerate() {
            if height == 0 {
                break;
            }
            let measurement = if index + 1 < self.content.len() {
                let measurement = item.measure(MeasureMode::fixed_width(dimension.width, wrap_mode));
                if measurement.dim.height > height {
                    item.measure(MeasureMode::exact(
                        Dimension::new(dimension.width, height),
                        wrap_mode,
                    ))
                } else {
                    measurement
                }
            } else {
                item.measure(MeasureMode::exact(
                    Dimension::new(dimension.width, height),
                    wrap_mode,
                ))
            };
            height = height.saturating_sub(measurement.dim.height);
            children.push(measurement);
        }
        Measurements::new(dimension, MeasurementSpecifics::Children(children))
    }
}

impl<const N: usize> From<[RcLayout; N]> for Vertical {
    fn from(content: [RcLayout; N]) -> Self {
        Self::new(content.to_vec())
    }
}

impl Default for Vertical {
    fn default() -> Self {
        Self::new(Vec::<RcLayout>::new())
    }
}

impl Layout for Vertical {
    fn measure(&self, mode: MeasureMode) -> Measurements {
        match mode {
            MeasureMode::Exact {
                dimension,
                wrap_mode,
            } => self.measure_exact(dimension, wrap_mode),
            _ => Measurements::fold_vertically(self.content.iter(), mode)
        }
    }

    fn layout_with_context(&'_ self, context: LayoutContext) -> BoxedFormattedLayout<'_> {
        // Validate, whether options fit
        if context.measurements.specifics.children().is_none() {
            return self.layout_strict(context.options);
        }

        // Go on with our stuff
        let child_measurements = context.measurements.specifics.children().unwrap();
        let mut y = 0;
        let children = self.content.iter().zip(child_measurements.iter())
            .map(|(l, m)| {
                let ctxt = LayoutContext::derive(m, 0,y, &context.options, false);
                y += ctxt.options.dim.height;
                l.layout_with_context(ctxt)
            })
            .collect();
        FormattedVertical::new(children, context.options).into()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

rc_layout!(Vertical);

/// [`FormattedLayout`] implementation for the [`Vertical`] layout.
pub struct FormattedVertical<'fmt> {
    content: Vec<BoxedFormattedLayout<'fmt>>,
    options: LayoutOptions,
}

impl<'fmt> FormattedVertical<'fmt> {
    /// Creates a new [`FormattedVertical`].
    /// The caller has to ensure that the `content` and `options` fit together
    ///
    /// # Parameters
    /// - `content`: The subordinated layouts
    /// - `options`: The options for this instance
    ///
    /// # Returns
    /// A new instance of [`FormattedVertical`]
    pub fn new(content: Vec<BoxedFormattedLayout<'fmt>>, options: LayoutOptions) -> Self {
        Self {
            content,
            options: options.with_normalized_horizontal_clip(),
        }
    }
}

impl FormattedLayout for FormattedVertical<'_> {
    fn options(&self) -> &LayoutOptions {
        &self.options
    }

    fn new_writer(&'_ self) -> BoxedLayoutWriter<'_> {
        let writers = self
            .content
            .iter()
            .map(|layout| layout.new_writer())
            .collect::<Vec<_>>();
        Box::new(VerticalWriter::new(writers, &self.options))
    }
}

box_formatted_layout!(FormattedVertical);

struct VerticalWriter<'wrt> {
    base: BaseLayoutWriter<'wrt>,
    content: Vec<BoxedLayoutWriter<'wrt>>,
    content_index: usize,
    start_row: usize,
}

impl<'wrt> VerticalWriter<'wrt> {
    fn new(content: Vec<BoxedLayoutWriter<'wrt>>, options: &'wrt LayoutOptions) -> Self {
        VerticalWriter {
            base: BaseLayoutWriter::new(options),
            content,
            content_index: 0,
            start_row: 0,
        }
    }

    fn update_writer_index(&mut self) {
        while self.content_index < self.content.len()
            && self.base.row() - self.start_row
                >= self.content[self.content_index].options().dim.height
        {
            self.content_index += 1;
            self.start_row = self.base.row();
        }
    }
}

impl<'wrt> LayoutWriter<'wrt> for VerticalWriter<'wrt> {
    fn options(&self) -> &'wrt LayoutOptions {
        self.base.options()
    }

    fn write_row(&mut self, w: &mut dyn Write) -> SizedLayoutResult {
        self.update_writer_index();
        self.content
            .get_mut(self.content_index)
            .map_or(Ok(0), |writer| self.base.write_row(writer.as_mut(), w))?;
        self.base.end_row(w)
    }
}

#[cfg(test)]
mod tests {
    use crate::Rect;
    use super::*;
    use crate::widgets::Lines;

    #[test]
    fn vertical_measure_pref_width() {
        // Arrange
        let lines1 = Lines::left(concat!("The quick brown fox jumps over\n", "the lazy dog."));
        let lines2 = Lines::center(concat!("To be or not to be,\n", "that is the question."));
        let lines3 = Lines::right(concat!("Life\n", "is\n", "life."));
        let vertical = Vertical::new(vec![lines1, lines2, lines3]);

        // Wrap case
        assert_eq!(vertical.measure(MeasureMode::pref_width(30, WrapMode::Wrap)).dim, Dimension::new(30, 7));
        assert_eq!(vertical.measure(MeasureMode::pref_width(25, WrapMode::Wrap)).dim, Dimension::new(25, 8));
        assert_eq!(
            vertical.measure(MeasureMode::pref_width(15, WrapMode::Wrap)).dim,
            Dimension::new(15, 10)
        );
        assert_eq!(vertical.measure(MeasureMode::pref_width(5, WrapMode::Wrap)).dim, Dimension::new(5, 21));
        assert_eq!(vertical.measure(MeasureMode::pref_width(1, WrapMode::Wrap)).dim, Dimension::new(1, 94));
        assert_eq!(vertical.measure(MeasureMode::pref_width(0, WrapMode::Wrap)).dim, Dimension::empty());

        // Truncate case
        assert_eq!(
            vertical.measure(MeasureMode::pref_width(30, WrapMode::Truncate("..."))).dim,
            Dimension::new(30, 7)
        );
        assert_eq!(
            vertical.measure(MeasureMode::pref_width(25, WrapMode::Truncate("..."))).dim,
            Dimension::new(25, 7)
        );
        assert_eq!(
            vertical.measure(MeasureMode::pref_width(15, WrapMode::Truncate("..."))).dim,
            Dimension::new(15, 7)
        );
        assert_eq!(
            vertical.measure(MeasureMode::pref_width(5, WrapMode::Truncate("..."))).dim,
            Dimension::new(5, 7)
        );
        assert_eq!(
            vertical.measure(MeasureMode::pref_width(1, WrapMode::Truncate("..."))).dim,
            Dimension::new(1, 7)
        );
        assert_eq!(
            vertical.measure(MeasureMode::pref_width(0, WrapMode::Truncate("..."))).dim,
            Dimension::empty()
        );
    }

    #[test]
    fn vertical_measure_min() {
        // Arrange
        let lines1 = Lines::left(concat!("The quick brown fox jumps over\n", "the lazy dog."));
        let lines2 = Lines::center(concat!("To be or not to be,\n", "that is the question."));
        let lines3 = Lines::right(concat!("Life\n", "is\n", "life."));
        let vertical = Vertical::new(vec![lines1, lines2, lines3]);

        assert_eq!(vertical.measure(MeasureMode::min()).dim, Dimension::new(30, 7));
    }

    #[test]
    fn vertical_layout_no_clip() {
        // Arrange
        let lines1 = Lines::left(concat!("The quick brown fox jumps over\n", "the lazy dog."));
        let lines2 = Lines::center(concat!("To be or not to be,\n", "that is the question."));
        let lines3 = Lines::right(concat!("Life\n", "is\n", "life."));
        let vertical = Vertical::from([lines1.into(), lines2.into(), lines3.into()]);

        // No fill rows
        let options = LayoutOptions::new(Dimension::new(30, 8), false, WrapMode::Wrap, None);
        let result = format!("{}", vertical.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "The quick brown fox jumps over\n",
                "the lazy dog.\n",
                "     To be or not to be,\n",
                "    that is the question.\n",
                "                          Life\n",
                "                            is\n",
                "                         life.\n",
                "\n",
            )
        );

        // With fill rows
        let options = LayoutOptions::new(Dimension::new(30, 8), true, WrapMode::Wrap, None);
        let result = format!("{}", vertical.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "The quick brown fox jumps over\n",
                "the lazy dog.                 \n",
                "     To be or not to be,      \n",
                "    that is the question.     \n",
                "                          Life\n",
                "                            is\n",
                "                         life.\n",
                "                              \n",
            )
        );
    }

    #[test]
    fn vertical_layout_with_clip() {
        // Arrange
        let lines1 = Lines::left(concat!("The quick brown fox jumps over\n", "the lazy dog."));
        let lines2 = Lines::center(concat!("To be or not to be,\n", "that is the question."));
        let lines3 = Lines::right(concat!("Life\n", "is\n", "life."));
        let vertical = Vertical::new(vec![lines1, lines2, lines3]);

        // No fill rows
        let options = LayoutOptions::new(
            Dimension::new(30, 8),
            false,
            WrapMode::Wrap,
            Some(Rect::new(2, 2, Dimension::new(26, 4))),
        );
        let result = format!("{}", vertical.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "   To be or not to be,\n",
                "  that is the question.\n",
                "                        Li\n",
                "                          \n",
            )
        );

        // With fill rows
        let options = LayoutOptions::new(
            Dimension::new(30, 8),
            true,
            WrapMode::Wrap,
            Some(Rect::new(2, 2, Dimension::new(26, 4))),
        );
        let result = format!("{}", vertical.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "   To be or not to be,    \n",
                "  that is the question.   \n",
                "                        Li\n",
                "                          \n",
            )
        );

        // Clip entire last row
        let options = LayoutOptions::new(
            Dimension::new(30, 8),
            true,
            WrapMode::Wrap,
            Some(Rect::new(0, 0, Dimension::new(30, 4))),
        );
        let result = format!("{}", vertical.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "The quick brown fox jumps over\n",
                "the lazy dog.                 \n",
                "     To be or not to be,      \n",
                "    that is the question.     \n",
            )
        );
    }

    #[test]
    fn vertical_layout_empty() {
        let vertical = Vertical::default();
        let options = LayoutOptions::default().with_dim(Dimension::new(10, 2));
        assert_eq!(format!("{}", vertical.layout_strict(options)), "\n\n");
    }

    #[test]
    fn vertical_layout_zero_width() {
        // Arrange
        let lines1 = Lines::left(concat!("The quick brown fox jumps over\n", "the lazy dog."));
        let lines2 = Lines::center(concat!("To be or not to be,\n", "that is the question."));
        let lines3 = Lines::right(concat!("Life\n", "is\n", "life."));
        let vertical = Vertical::new(vec![lines1, lines2, lines3]);

        assert_eq!(format!("{}", vertical.layout(0)), "");
    }
}
