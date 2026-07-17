use crate::{Dimension, LayoutContext, LayoutOptions, MeasureMode, Measurements, WrapMode};
use std::any::Any;
use std::fmt::{Display, Error, Formatter, Write};
use std::rc::Rc;

/// Central trait that describes how to lay out textual content.
///
/// The `Layout` trait provides two groups of methods:
/// - Methods that calculate the [`Dimension`] of a `Layout`. These are typically called during the
///   layout process to determine the measurements of the content.
/// - Methods that actually lay out the content. The most central one is
///   [`layout_strict`](Layout::layout_strict); all other `layout*` methods are convenience
///   wrappers that delegate to this one.
///
/// # Layout Process
/// The `Layout` trait just defines the content and the intention, how to lay it out. Typically,
/// the `Layout` itself has no information about the dimensions of the output area. When it comes
/// to the actual layout process, the `layout_strict` is called, which expects a [`LayoutOptions`]
/// that describes the output area and other aspects of the layout.
///
/// The `layout_strict` returns a [`FormattedLayout`] (more exactly a [`BoxedFormattedLayout`],
/// which is just a boxed `FormattedLayout`) that contains intermediate data from the `Layout` and
/// the `LayoutOptions`. In opposite to a `Layout`, the `FormattedLayout` is strictly bound to
/// concrete `LayoutOptions`.
///
/// The `BoxedFormattedLayout` implements the [`Display`] trait, which means that it can be used to
/// actually display the layout. As described in the [`FormattedLayout`] trait, the `FormattedLayout`
/// creates a [`LayoutWriter`] that is used to display the content.
///
/// So all in all the involved types are related as follows:
///
/// ```text
///                         +--------+
///                         | Layout |
///                         +--------+
///                             |               +---------------+
///                         layout_strict() <-- | LayoutOptions |
///                             |               +---------------+
///                            \/
///                    +-----------------+
///                    | FormattedLayout |
///                    +-----------------+
///                             |
///                   format/display produces
///                             |
///                            \/
///                     +--------------+
///                     | LayoutWriter |
///                     +--------------+
///
/// ```
///
/// # Examples
/// The following example shows a typical usage:
/// ```rust
/// use termlayout::*;
/// use termlayout::widgets::Paragraph;
///
/// let layout = Paragraph::left("This is a sample paragraph we want to lay out.");
/// assert_eq!(format!("{}", layout.layout(22)),
///     concat!(
///         "This is a sample\n",
///         "paragraph we want to\n",
///         "lay out.\n",
///     ));
/// ```
pub trait Layout {
    /// Calculates the preferred [`Dimension`] of this layout.
    /// The resulting `Dimension` must not exceed `max_width`; it represents the `Dimension`
    /// if this layout was laid out with the given `max_width` and `wrap_mode`.
    ///
    /// # Parameters
    /// - `max_width`: The maximum width the result `Dimension` has. The concrete width
    ///   might be less or equal; but never larger
    /// - `wrap_mode`: The [`WrapMode`] to apply when lay out.
    ///
    /// # Returns
    /// The preferred `Dimension`
    ///
    /// # Examples
    /// ```rust
    /// use termlayout::*;
    /// use termlayout::widgets::Paragraph;
    ///
    /// let layout = Paragraph::left("This is some sample text");
    ///
    /// let dimension = layout.pref_dim(100000, WrapMode::Wrap);
    /// assert_eq!(dimension, Dimension::new(24, 1));
    ///
    /// let dimension = layout.pref_dim(13, WrapMode::Wrap);
    /// assert_eq!(dimension, Dimension::new(12, 2));
    ///
    /// let dimension = layout.pref_dim(13, WrapMode::default_truncate());
    /// assert_eq!(dimension, Dimension::new(12, 2));
    ///
    /// let dimension = layout.pref_dim(4, WrapMode::Wrap);
    /// assert_eq!(dimension, Dimension::new(4, 6));
    ///
    /// let dimension = layout.pref_dim(4, WrapMode::default_truncate());
    /// assert_eq!(dimension, Dimension::new(4, 5));
    /// ```
    #[deprecated(since = "0.1.1", note = "Use `measure()` instead")]
    fn pref_dim(&self, max_width: usize, wrap_mode: WrapMode) -> Dimension {
        self.measure(MeasureMode::pref(max_width, wrap_mode)).dim
    }

    /// Calculates the [`Dimension`] of this layout based on the given `max_width`.
    /// The result is similar to [`pref_dim()`](Layout::pref_dim), but returns a dimension with
    /// exactly the given `max_width`.
    ///
    /// # Parameters
    /// - `width` - The maximum width the result `Dimension` has. The concrete width
    ///   might be less or equal; but never larger
    /// - `wrap_mode` - The [`WrapMode`] to apply when lay out.
    ///
    /// # Returns
    /// The preferred `Dimension`
    ///
    /// # Examples
    /// ```rust
    /// use termlayout::*;
    /// use termlayout::widgets::Paragraph;
    ///
    /// let layout = Paragraph::left("This is some sample text");
    ///
    /// let dimension = layout.pref_dim_fixed_width(100000, WrapMode::Wrap);
    /// assert_eq!(dimension, Dimension::new(100000, 1));
    ///
    /// let dimension = layout.pref_dim_fixed_width(13, WrapMode::Wrap);
    /// assert_eq!(dimension, Dimension::new(13, 2));
    /// ```
    #[deprecated(since = "0.1.1", note = "Use `measure()` instead")]
    fn pref_dim_fixed_width(&self, width: usize, wrap_mode: WrapMode) -> Dimension {
        self.measure(MeasureMode::FixedWidth {width, wrap_mode}).dim
    }

    /// Returns the [`Dimension`] with the minimal width that can be used to display this
    /// `Layout` without wrapping, truncation, and loss of information.
    ///
    /// For example, for a paragraph this would be a dimension with a width of the longest word
    /// and an according height.
    ///
    /// # Returns
    /// The minimal `Dimension` in terms of the width.
    ///
    /// # Examples
    /// ```rust
    /// use termlayout::*;
    /// use termlayout::widgets::Paragraph;
    ///
    /// let layout = Paragraph::left("This is a small test");
    ///
    /// let dimension = layout.min_dim();
    ///
    /// assert_eq!(dimension, Dimension::new(5, 4));
    ///```
    #[deprecated(since = "0.1.1", note = "Use `measure()` instead")]
    fn min_dim(&self) -> Dimension {
        self.measure(MeasureMode::Min).dim
    }

    fn measure(&self, mode: MeasureMode) -> Measurements;
    
    /// Generates a [`FormattedLayout`] so that the content does not exceed `max_width` columns
    /// and is wrapped if required.
    ///
    /// # Parameters
    /// * `max_width`: The maximum output width
    ///
    /// # Returns
    /// The matching [`FormattedLayout`]
    ///
    /// # Examples
    /// ```rust
    /// use termlayout::*;
    /// use termlayout::widgets::Paragraph;
    ///
    /// let layout = Paragraph::left("This is a sample paragraph we want to lay out.");
    /// assert_eq!(format!("{}", layout.layout(20)),
    ///     concat!(
    ///         "This is a sample\n",
    ///         "paragraph we want to\n",
    ///         "lay out.\n",
    ///     ));
    /// ```
    fn layout(&'_ self, max_width: usize) -> BoxedFormattedLayout<'_> {
        self.layout_with_wrap_mode(max_width, WrapMode::default())
    }

    /// Generates a [`FormattedLayout`] so that the content does not exceed `max_width` columns
    /// and uses the given `wrap_mode`.
    ///
    /// # Parameters
    /// * `max_width`: The maximum output width
    /// * `wrap_mode`: The [`WrapMode`] to use
    ///
    /// # Returns
    /// The matching [`FormattedLayout`]
    ///
    /// # Examples
    /// ```rust
    /// use termlayout::*;
    /// use termlayout::widgets::Paragraph;
    ///
    /// let layout = Paragraph::left("This is a sample paragraph we want to lay out.");
    /// assert_eq!(format!("{}", layout.layout_with_wrap_mode(8, WrapMode::default_truncate())),
    ///     concat!(
    ///         "This is\n",
    ///         "a sample\n",
    ///         "paragra…\n",
    ///         "we want\n",
    ///         "to lay\n",
    ///         "out.\n"
    ///     ));
    /// ```
    fn layout_with_wrap_mode(
        &'_ self,
        max_width: usize,
        wrap_mode: WrapMode,
    ) -> BoxedFormattedLayout<'_> {
        let measurements = self.measure(MeasureMode::pref(max_width, wrap_mode));
        let options = LayoutOptions::new(
            measurements.dim,
            false,
            wrap_mode,
            None
        );
        self.layout_with_context(LayoutContext::new(options, measurements))
    }

    /// Creates a [`FormattedLayout`] that strictly follows the provided [`LayoutOptions`].
    /// This means that it will never exceed the provided dimensions and will wrap text as specified.
    ///
    /// The returned `FormattedLayout` is bound to the lifetime of this instance and typically
    /// contains intermediate/precomputed data based on this instance and the supplied options.
    ///
    /// # Parameters
    /// - `options`: The `LayoutOptions` to use for formatting.
    ///
    /// # Returns
    /// A `BoxedFormattedLayout` that represents the formatted layout.
    ///
    /// # Examples
    /// ```rust
    /// use termlayout::*;
    /// use termlayout::widgets::Paragraph;
    ///
    /// let layout = Paragraph::left("This is a sample paragraph we want to lay out.");
    /// let options = LayoutOptions::new(
    ///     Dimension::new(22, 4),
    ///     true,
    ///     WrapMode::empty_truncate(),
    ///     None);
    /// assert_eq!(format!("{}", layout.layout_strict(options)),
    ///     concat!(
    ///         "This is a sample      \n",
    ///         "paragraph we want to  \n",
    ///         "lay out.              \n",
    ///         "                      \n"
    ///     ));
    /// ```
    fn layout_strict(&'_ self, options: LayoutOptions) -> BoxedFormattedLayout<'_> {
        let measurements = self.measure(MeasureMode::exact(options.dim));
        let context = LayoutContext::new(options, measurements);
        self.layout_with_context(context)
    }
    
    fn layout_with_context(&'_ self, context: LayoutContext) -> BoxedFormattedLayout<'_>;

    /// Returns `self` as a `&dyn Any`, allowing for runtime type introspection.
    ///
    /// This method enables safe downcasting of `Layout` trait objects to their concrete types.
    ///
    /// # Examples
    /// ```rust
    /// use std::any::Any;
    /// use termlayout::RcLayout;
    /// use termlayout::widgets::Lines;
    /// use termlayout::Layout;
    ///
    /// let layout: RcLayout = Lines::left("Hello").into();
    ///
    /// // Attempt to downcast the RcLayout to a concrete Lines type
    /// if let Some(lines_layout) = layout.as_any().downcast_ref::<Lines>() {
    ///     println!("Successfully downcasted to Lines: {}", lines_layout.content);
    /// } else {
    ///     println!("Could not downcast to Lines.");
    /// }
    /// ```
    fn as_any(&self) -> &dyn Any;
}

/// A layout living in a [`Rc`]
pub type RcLayout = Rc<dyn Layout>;

/// Intermediate data from a [`Layout`] with associated [`LayoutOptions`].
/// A [`FormattedLayout`] is created by calling [`layout_strict()`](Layout::layout_strict) on a
/// `Layout` and it knows both - the data from the `Layout` and the `LayoutOptions`. Since the
/// [`BoxedFormattedLayout`] (which is just a boxed `FormattedLayout`) implements the [`Display`]
/// trait, the `FormattedLayout` builds the bridge between the [`Formatter`] system and the `Layout`
/// system.
/// While a `Layout` describes the content and its intention in a more declarative
/// way, the `FormattedLayout` represents the concrete representation of the content for the given
/// `LayoutOptions`.
///
/// The most important method of this trait is the [`new_writer()`](FormattedLayout::new_writer)
/// method which creates a [`LayoutWriter`] that displays this instance row by row.
///
pub trait FormattedLayout {
    /// Returns the [`LayoutOptions`] of this instance.
    /// These are the effective options that are used to render the layout.
    ///
    /// # Returns
    /// The effective [`LayoutOptions`] used for rendering.
    fn options(&self) -> &LayoutOptions;

    /// Creates a new [`LayoutWriter`] for displaying the formatted content of this instance.
    /// The lifetime of the writer is bound to the lifetime of this instance
    ///
    /// # Returns
    /// A new [`LayoutWriter`] wrapped in a `Box` to ensure dynamic dispatch.
    fn new_writer(&'_ self) -> BoxedLayoutWriter<'_>;
}

impl Display for dyn FormattedLayout + '_ {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut writer = self.new_writer();
        let height = self.options().dim.height;
        let rect = self.options().visible_rect();
        for row in 0..height {
            writer.write_row(f)?;
            if rect.y_range().contains(&row) {
                f.write_str("\n")?;
            }
        }
        Ok(())
    }
}

/// A [`FormattedLayout`] living in a [`Box`].
pub type BoxedFormattedLayout<'fmt> = Box<dyn FormattedLayout + 'fmt>;

/// Generates the output of a [`FormattedLayout`]. Objects of this type are created by the
/// [`new_writer()`](FormattedLayout::new_writer) method and are used when displaying the
/// `FormattedLayout`. A `LayoutWriter` typically has a lifetime bound to the `FormattedLayout`
/// it was created for, so it lives only during the display process.
///
/// The most important method is [`write_row`](LayoutWriter::write_row), which is called during the
/// display process to write each row to the [`Write`]. The usage of the [`crate::ext::BaseLayoutWriter`]
/// struct might simplify the implementation of this trait.
///
/// # Examples
/// The following example demonstrates the basic usage; be aware that the implementation is
/// simplified to demonstrate the surroundings.
///
///```rust
///
///
/// // This is just the FormattedLayout that drives the LayoutWriter
/// use termlayout::*;
/// use termlayout::ext::*;
/// use std::fmt::Write;
///
/// struct RowNumbers {
///    options: LayoutOptions
/// }
///
/// impl FormattedLayout for RowNumbers {
///
///    fn options(&self) -> &LayoutOptions {
///         &self.options
///    }
///    fn new_writer(&self) -> BoxedLayoutWriter<'_> {
///         Box::new(RowNumbersWriter{ options: &self.options, row: 0})
///    }
/// }
///
/// struct RowNumbersWriter<'wrt> {
///    options: &'wrt LayoutOptions,
///    row: usize
/// }
///
/// impl<'wrt> LayoutWriter<'wrt> for RowNumbersWriter<'wrt> {
///    fn options(&self) -> &'wrt LayoutOptions {
///        self.options
///    }
///
///    fn write_row(&mut self, w: &mut dyn Write) -> SizedLayoutResult {
///
///        self.row += 1;
///        // This is extremely simplified; real implementations have to ensure that we respect
///        // options, especially the fill_rows and dimension.width settings. Normally you have
///        // to check whether the output fits into the area and may add padding spaces to the
///        // output
///        let value = format!("{}", self.row);
///        w.write_str(value.as_str())?;
///        Ok(value.len())
///    }
/// }
///
/// let row_numbers: BoxedFormattedLayout = Box::new(RowNumbers{
///    options: LayoutOptions::default().with_dim(Dimension::new(4,5))
/// });
///
/// let formatted = format!("{}", row_numbers);
/// assert_eq!(formatted, "1\n2\n3\n4\n5\n");
///```
pub trait LayoutWriter<'wrt> {
    /// Gets the associated [`LayoutOptions`]
    ///
    /// # Returns
    /// The `LayoutOptions`
    fn options(&self) -> &'wrt LayoutOptions;

    /// Writes the next row to the given [`Write`].
    /// In detail, the following things should be done:
    /// * It should write all data for the current row to the `Write`;
    ///   always *without* any new line and *never* more characters than defined in the
    ///   [width](Dimension::width) of the associated [`LayoutOptions`] dimension.
    /// * If the actual output is smaller than the width but [`fill_rows`](LayoutOptions::fill_rows)
    ///   is set to `true`, the rest of the available width must be filled with spaces.
    ///
    /// # Parameters
    /// * `w`: The [`Write`] to write to.
    ///
    /// # Returns
    /// `Ok`, if succeeded, with the number of characters written to the formatter for the entire
    /// line. This includes also the characters that were clipped, but excludes the control
    /// characters required for styling. `Err` if not successful.
    ///
    /// # Errors
    /// If writing to the provided [`Write`] fails, an error is returned.
    fn write_row(&mut self, w: &mut dyn Write) -> SizedLayoutResult;
}

/// A [`LayoutWriter`] living in a [`Box`].
pub type BoxedLayoutWriter<'wrt> = Box<dyn LayoutWriter<'wrt> + 'wrt>;

/// A result providing the number of characters written to the [`Write`]
pub type SizedLayoutResult = Result<usize, Error>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Rect;

    struct MockLayoutWriter<'a> {
        options: &'a LayoutOptions,
        row: usize,
    }

    impl<'a> LayoutWriter<'a> for MockLayoutWriter<'a> {
        fn options(&self) -> &'a LayoutOptions {
            self.options
        }

        fn write_row(&mut self, w: &mut dyn Write) -> SizedLayoutResult {
            let s = format!("row{}", self.row);
            w.write_str(&s)?;
            self.row += 1;
            Ok(s.len())
        }
    }

    struct MockFormattedLayout {
        options: LayoutOptions,
    }

    impl FormattedLayout for MockFormattedLayout {
        fn options(&self) -> &LayoutOptions {
            &self.options
        }

        fn new_writer(&self) -> BoxedLayoutWriter<'_> {
            Box::new(MockLayoutWriter {
                options: &self.options,
                row: 0,
            })
        }
    }

    #[test]
    fn formatted_layout_display() {
        let options = LayoutOptions::new(Dimension::new(10, 3), false, WrapMode::Wrap, None);
        let layout = MockFormattedLayout { options };
        let formatted = format!("{}", &layout as &dyn FormattedLayout);
        assert_eq!(formatted, "row0\nrow1\nrow2\n");
    }

    #[test]
    fn formatted_layout_display_clipped() {
        let options = LayoutOptions::new(
            Dimension::new(10, 3),
            false,
            WrapMode::Wrap,
            Some(Rect::new(0, 0, Dimension::new(10, 2))),
        );
        let layout = MockFormattedLayout { options };
        let formatted = format!("{}", &layout as &dyn FormattedLayout);
        // Display impl prints \n if rect.y_range().contains(&row)
        // row 0: contains 0 -> row0\n
        // row 1: contains 1 -> row1\n
        // row 2: NOT contains 2 -> row2
        assert_eq!(formatted, "row0\nrow1\nrow2");
    }
}
