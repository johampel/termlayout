use crate::{Dimension, Measurements, Rect};

pub struct  LayoutContext<'a>  {
    pub options: LayoutOptions,
    pub measurements: &'a Measurements,
}

impl<'a> LayoutContext<'a> {
    pub fn new(options: LayoutOptions, measurements: &'a Measurements) -> Self {
        Self {
            options,
            measurements,
        }
    }

    pub fn derive(measurements: &'a Measurements, x: usize, y: usize, options: &LayoutOptions, adjust_fill_rows: bool) -> Self
    {
        Self::new(options.intersect(Rect::new(x, y, measurements.dim), adjust_fill_rows), measurements)
    }

}

impl<'a> Into<LayoutOptions> for LayoutContext<'a> {
    fn into(self) -> LayoutOptions {
        self.options
    }
}

/// Configuration settings that control how a layout is rendered.
///
/// `LayoutOptions` defines the target dimensions, row-filling behavior, text wrapping mode,
/// and optional clipping rectangle for a layout.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct LayoutOptions {
    /// Specifies the overall dimensioning of the layout. It is the basis for the layout
    /// calculation and - unless `clip` is set - is also the dimension of the output one can
    /// expect.
    pub dim: Dimension,

    /// Determines whether the layout fills the row completly. A `true` value indicates that
    /// the layout should stretch to fill the row, while a `false` value may allow the layout
    /// to adjust based on its content. In doubt, "stretching" means to fill the row with spaces.
    pub fill_rows: bool,

    /// Configures the behavior for wrapping content within the layout. The `WrapMode` type
    /// is expected to define different strategies for handling wrap situations (e.g., overflow).
    pub wrap_mode: WrapMode,

    /// Specifies an optional clipping rect. If such a clipping rect is present, the layout is
    /// calculated for the `dimension`, but only the portion within the clipping rect comes to
    /// output.
    pub clip: Option<Rect>,
}

impl LayoutOptions {
    /// Creates a new instance.
    /// Exhaustive description of the parameters can be found in the struct's documentation.
    ///
    /// # Parameters
    /// - `dim`: The [`Dimension`] for the layout.
    /// - `fill_rows`: Determines whether the layout fills the row completely.
    /// - `wrap_mode`: Configures the behavior for wrapping content within the layout.
    /// - `clip`: Specifies an optional clipping rect
    ///
    /// # Returns
    /// A new instance
    #[must_use]
    pub fn new(dim: Dimension, fill_rows: bool, wrap_mode: WrapMode, clip: Option<Rect>) -> Self {
        Self {
            dim,
            fill_rows,
            wrap_mode,
            clip,
        }
    }

    /// Creates a new instance based on this one having the given dimension
    ///
    /// # Parameters
    /// - `dimension`: The new [`Dimension`]
    ///
    /// # Returns
    /// A new instance with the new dimension
    ///
    /// # Example
    /// ```rust
    /// use termlayout::*;
    ///
    /// let options = LayoutOptions::default()
    ///     .with_dim(Dimension::new(1, 2));
    /// assert_eq!(options.dim.width, 1);
    /// assert_eq!(options.dim.height, 2);
    /// ```
    #[must_use]
    pub fn with_dim(&self, dim: Dimension) -> Self {
        Self { dim, ..*self }
    }

    /// Creates a new instance based on this one having the given `clip`
    ///
    /// # Parameters
    /// - `clip`: The new clipping rect or `None`.
    ///
    /// # Returns
    /// A new instance with the new clipping rect
    ///
    /// # Example
    /// ```rust
    /// use termlayout::*;
    ///
    /// let options = LayoutOptions::default()
    ///     .with_clip(Some(Rect::from(Dimension::new(5, 6))));
    /// assert_eq!(options.clip,Some(Rect::from(Dimension::new(5, 6))));
    /// ```
    #[must_use]
    pub fn with_clip(&self, clip: Option<Rect>) -> Self {
        Self { clip, ..*self }
    }

    /// Creates a new instance based on this one having the given `wrap_mode`
    ///
    /// # Parameters
    /// - `wrap_mode`: The new [`WrapMode`]
    ///
    /// # Returns
    /// A new instance with the new wrap mode
    ///
    /// # Example
    /// ```rust
    /// use termlayout::*;
    ///
    /// let options = LayoutOptions::default()
    ///     .with_wrap_mode(WrapMode::Truncate("..."));
    /// assert_eq!(options.wrap_mode, WrapMode::Truncate("..."));
    /// ```
    #[must_use]
    pub fn with_wrap_mode(&self, wrap_mode: WrapMode) -> Self {
        Self { wrap_mode, ..*self }
    }

    /// Creates a new instance based on this one having the given `fill_rows` flag
    ///
    /// # Parameters
    /// - `fill_rows`: The new fill-row flag
    ///
    /// # Returns
    /// A new instance with the new fill row flag
    ///
    /// # Example
    /// ```rust
    /// use termlayout::*;
    ///
    /// let options = LayoutOptions::default()
    ///     .with_fill_rows(true);
    /// assert_eq!(options.fill_rows, true);
    /// ```
    #[must_use]
    pub fn with_fill_rows(&self, fill_rows: bool) -> Self {
        Self { fill_rows, ..*self }
    }

    /// Calculates the effectively visible rect.
    /// When no clipping is active, it is simply the rect of the options dimension, otherwise
    /// the intersection of the dimension and the clipping rect
    ///
    /// # Returns
    /// [`Rect`] of the visible area
    ///
    /// # Examples
    /// ```rust
    /// use termlayout::*;
    ///
    /// let options = LayoutOptions::default()
    ///     .with_dim(Dimension::new(10, 5));
    ///
    /// assert_eq!(options.visible_rect(), Rect::from(Dimension::new(10, 5)));
    ///
    /// let options = options.with_clip(Some(Rect::new(1, 2, Dimension::new(5, 10))));
    /// assert_eq!(options.visible_rect(), Rect::new(1, 2, Dimension::new(5, 3)));
    /// ```
    #[must_use]
    pub fn visible_rect(&self) -> Rect {
        let rect = self.dim.into();
        self.clip.map_or(rect, |c| c.intersect(rect))
    }

    /// Computes the intersection of this instance with the given `rect`.
    /// The resulting options will have the same dimensions as `rect`, and the clipping area
    /// is adapted according to the position and dimension of `rect`. If `adjust_fill_rows` is
    /// `true`, it also sets the `fill_rows` flag in case the rect has a smaller right side bound
    /// than the original options.
    ///
    /// # Parameters
    /// - `rect`: The [`Rect`] to intersect with.
    /// - `adjust_fill_rows`: If `true`, the fill rows flag will be adapted.
    ///
    /// # Returns
    /// A new [`LayoutOptions`] instance representing the intersection.
    ///
    /// # Examples
    /// ```rust
    /// use termlayout::*;
    ///
    /// let options = LayoutOptions::default()
    ///     .with_dim(Dimension::new(11, 9))
    ///     .with_clip(Some(Rect::new(2, 0, Dimension::new(8, 6))));
    ///
    /// let rect = Rect::new(1, 1, Dimension::new(5, 5));
    /// let result = options.intersect(rect, true);
    /// assert_eq!(result, LayoutOptions::new(
    ///     rect.dim,
    ///     true,
    ///     options.wrap_mode,
    ///     Some(Rect::new(1, 0, Dimension::new(4, 5)))
    /// ));
    /// ```
    #[must_use]
    pub fn intersect(&self, rect: Rect, adjust_fill_rows: bool) -> Self {
        let clip = rect.intersect_relative(self.visible_rect());
        let fill_rows = if adjust_fill_rows && !self.fill_rows {
            rect.x_range().end < self.dim.width
        } else {
            self.fill_rows
        };

        Self::new(rect.dim, fill_rows, self.wrap_mode, Some(clip))
    }

    /// Creates a new [`LayoutOptions`] instance with a normalized horizontal clip.
    /// This method adjusts the horizontal clipping of the layout based on the visible rectangle's
    /// dimensions while maintaining the original vertical dimensions. It ensures proper rendering
    /// by normalizing the horizontal range and preserving other layout options like row fill behavior
    /// and wrap mode.
    ///
    /// # Returns
    /// A new `LayoutOptions` instance with the same vertical dimensions as the current instance,
    /// but with the horizontal dimensions normalized to match the visible rectangle's width.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::*;
    ///
    /// let options = LayoutOptions::default()
    ///     .with_dim(Dimension::new(11, 9))
    ///     .with_clip(Some(Rect::new(2, 2, Dimension::new(8, 6))));
    ///
    /// let result = options.with_normalized_horizontal_clip();
    /// assert_eq!(result, LayoutOptions::new(
    ///     Dimension::new(8, 9),
    ///     false,
    ///     WrapMode::default(),
    ///     Some(Rect::new(0, 2, Dimension::new(8, 6)))
    /// ));
    /// ```
    #[must_use]
    pub fn with_normalized_horizontal_clip(&self) -> Self {
        let visible = self.visible_rect();
        LayoutOptions::new(
            Dimension::new(visible.dim.width, self.dim.height),
            self.fill_rows,
            self.wrap_mode,
            Some(Rect::new(0, visible.y, visible.dim)),
        )
    }

    /// Creates a new [`LayoutOptions`] instance with a normalized vertical clip.
    /// This method adjusts the vertical clipping of the layout based on the visible rectangle's
    /// dimensions while maintaining the original horizontal dimensions. It ensures proper rendering
    /// by normalizing the vertical range and preserving other layout options like row fill behavior
    /// and wrap mode.
    ///
    /// # Returns
    /// A new `LayoutOptions` instance with the same horizontal dimensions as the current instance,
    /// but with the vertical dimensions normalized to match the visible rectangle's width.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::*;
    ///
    /// let options = LayoutOptions::default()
    ///     .with_dim(Dimension::new(11, 9))
    ///     .with_clip(Some(Rect::new(2, 2, Dimension::new(8, 6))));
    ///
    /// let result = options.with_normalized_vertical_clip();
    /// assert_eq!(result, LayoutOptions::new(
    ///     Dimension::new(8, 9),
    ///     false,
    ///     WrapMode::default(),
    ///     Some(Rect::new(2, 0, Dimension::new(8, 6)))
    /// ));
    /// ```
    #[must_use]
    pub fn with_normalized_vertical_clip(&self) -> Self {
        let visible = self.visible_rect();
        LayoutOptions::new(
            Dimension::new(visible.dim.width, self.dim.height),
            self.fill_rows,
            self.wrap_mode,
            Some(Rect::new(visible.x, 0, visible.dim)),
        )
    }

    /// Creates a new [`LayoutOptions`] instance with a normalized clip.
    /// This method adjusts the clipping of the layout based on the visible rectangle's
    /// dimensions while maintaining the original dimensions. It ensures proper rendering
    /// by normalizing the range and preserving other layout options like row fill behavior
    /// and wrap mode.
    ///
    /// # Returns
    /// A new `LayoutOptions` instance with the same dimensions as the current instance,
    /// but with the dimensions normalized to match the visible rectangle's dimension.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::*;
    ///
    /// let options = LayoutOptions::default()
    ///     .with_dim(Dimension::new(11, 9))
    ///     .with_clip(Some(Rect::new(2, 2, Dimension::new(8, 6))));
    ///
    /// let result = options.with_normalized_clip();
    /// assert_eq!(result, LayoutOptions::new(
    ///     Dimension::new(8, 9),
    ///     false,
    ///     WrapMode::default(),
    ///     Some(Rect::new(0, 0, Dimension::new(8, 6)))
    /// ));
    /// ```
    #[must_use]
    pub fn with_normalized_clip(&self) -> Self {
        let visible = self.visible_rect();
        LayoutOptions::new(
            Dimension::new(visible.dim.width, self.dim.height),
            self.fill_rows,
            self.wrap_mode,
            Some(Rect::new(0, 0, visible.dim)),
        )
    }
}

impl Default for LayoutOptions {
    fn default() -> Self {
        Self {
            dim: Dimension::new(80, usize::MAX),
            fill_rows: false,
            wrap_mode: WrapMode::default(),
            clip: None,
        }
    }
}

/// The `WrapMode` enum specifies the behavior for handling text or content that exceeds a given
/// boundary. It is used in the [`LayoutOptions`] and also influences the calculation of the
/// [`crate::Layout`] dimensions.
#[derive(Copy, Clone, Debug, PartialEq, Default, Eq)]
pub enum WrapMode {
    /// This variant specifies that any content exceeding the boundary should be truncated.
    /// A suffix, given as a static string, can be added to the truncated text to indicate the truncation.
    /// For example, a common suffix could be `...`.
    /// - Parameter:
    ///   * `str` - The suffix to append to the truncated content.
    Truncate(&'static str),

    /// This variant specifies that content exceeding the boundary should wrap to the next line,
    /// preserving the entire content without truncation.
    #[default]
    Wrap,
}

impl WrapMode {
    /// Creates a default instance of the `WrapMode::Truncate` to append an ellipsis ("…") when the
    /// content exceeds a certain length.
    #[must_use]
    pub const fn default_truncate() -> Self {
        WrapMode::Truncate("…")
    }

    /// Creates an empty instance of the `WrapMode::Truncate` to append nothing when the
    /// content exceeds a certain length.
    #[must_use]
    pub const fn empty_truncate() -> Self {
        WrapMode::Truncate("")
    }

    /// Calculates the dimension of content based on the specified maximum width and content width.
    /// Depending on the wrap mode, it takes line wrapping into account to determine the dimension.
    ///
    /// # Parameters
    /// - `max_width` - The available width for the content
    /// - `width` - The actual width of the content
    ///
    /// # Returns
    /// The calculated [`Dimension`] of the content based on the wrap mode
    ///
    /// # Examples
    /// ```rust
    /// use termlayout::*;
    ///
    /// assert_eq!(WrapMode::Wrap.dimension_for(10, 0), Dimension::new(0, 0));
    /// assert_eq!(WrapMode::Wrap.dimension_for(10, 5), Dimension::new(5, 1));
    /// assert_eq!(WrapMode::Wrap.dimension_for(10, 10), Dimension::new(10, 1));
    /// assert_eq!(WrapMode::Wrap.dimension_for(10, 15), Dimension::new(10, 2));
    ///
    /// assert_eq!(WrapMode::Truncate("...").dimension_for(10, 0), Dimension::new(0, 0));
    /// assert_eq!(WrapMode::Truncate("...").dimension_for(10, 5), Dimension::new(5, 1));
    /// assert_eq!(WrapMode::Truncate("...").dimension_for(10, 10), Dimension::new(10, 1));
    /// assert_eq!(WrapMode::Truncate("...").dimension_for(10, 15), Dimension::new(10, 1));
    /// ```
    #[must_use]
    pub fn dimension_for(self, max_width: usize, width: usize) -> Dimension {
        if width > 0 && max_width > 0 {
            match self {
                WrapMode::Wrap => {
                    Dimension::new(std::cmp::min(max_width, width), 1 + (width - 1) / max_width)
                }
                WrapMode::Truncate(_) => Dimension::new(std::cmp::min(max_width, width), 1),
            }
        } else {
            Dimension::empty()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_options_with_methods() {
        let opt = LayoutOptions::default();
        let opt = opt.with_dim(Dimension::new(10, 20));
        assert_eq!(opt.dim, Dimension::new(10, 20));
        let opt = opt.with_fill_rows(true);
        assert!(opt.fill_rows);
        let opt = opt.with_wrap_mode(WrapMode::default_truncate());
        assert_eq!(opt.wrap_mode, WrapMode::default_truncate());
        let opt = opt.with_clip(Some(Rect::empty()));
        assert_eq!(opt.clip, Some(Rect::empty()));
    }

    #[test]
    fn layout_options_visible_rect() {
        let opt = LayoutOptions::new(Dimension::new(10, 10), false, WrapMode::Wrap, None);
        assert_eq!(opt.visible_rect(), Rect::new(0, 0, Dimension::new(10, 10)));

        let opt = opt.with_clip(Some(Rect::new(5, 5, Dimension::new(10, 10))));
        assert_eq!(opt.visible_rect(), Rect::new(5, 5, Dimension::new(5, 5)));
    }

    #[test]
    fn layout_options_intersect() {
        let source = LayoutOptions::new(
            Dimension::new(11, 13),
            false,
            WrapMode::default(),
            Some(Rect::new(2, 3, Dimension::new(5, 7))),
        );

        // Same dimension
        let result = source.intersect(Rect::new(0, 0, Dimension::new(11, 13)), false);
        assert_eq!(
            result,
            LayoutOptions::new(
                Dimension::new(11, 13),
                false,
                WrapMode::default(),
                Some(Rect::new(2, 3, Dimension::new(5, 7))),
            )
        );

        // Inner intersection
        let result = source.intersect(Rect::new(1, 2, Dimension::new(5, 7)), false);
        assert_eq!(
            result,
            LayoutOptions::new(
                Dimension::new(5, 7),
                false,
                WrapMode::default(),
                Some(Rect::new(1, 1, Dimension::new(4, 6))),
            )
        );
    }

    #[test]
    fn layout_options_with_normalized_horizontal_clip() {
        let opt = LayoutOptions::new(
            Dimension::new(20, 10),
            false,
            WrapMode::Wrap,
            Some(Rect::new(5, 2, Dimension::new(10, 5))),
        );
        let normalized = opt.with_normalized_horizontal_clip();
        assert_eq!(normalized.dim, Dimension::new(10, 10));
        assert_eq!(
            normalized.clip,
            Some(Rect::new(0, 2, Dimension::new(10, 5)))
        );
    }

    #[test]
    fn wrap_mode_dimension_for() {
        assert_eq!(WrapMode::Wrap.dimension_for(10, 15), Dimension::new(10, 2));
        assert_eq!(
            WrapMode::default_truncate().dimension_for(10, 15),
            Dimension::new(10, 1)
        );
        assert_eq!(WrapMode::Wrap.dimension_for(10, 0), Dimension::empty());
    }
}
