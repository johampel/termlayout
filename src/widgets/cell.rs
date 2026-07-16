mod formatted;
pub(crate) mod metrics;

use crate::{rc_layout, BoxedFormattedLayout, Dimension, Layout, LayoutOptions, MeasureMode, RcLayout, Rect, WrapMode, Measurements, LayoutContext};
use std::any::Any;
use std::cmp::min;

/// A container widget with pre-sized content and positioning control.
///
/// A `Cell` consists of content and configuration for how to lay out and place this content
/// within the cell's area.
///
/// Besides the `content` itself (which is a [`Layout`]), the `Cell` also contains fields for the
/// content's dimension, wrapping mode, and clipping. This allows the sizing of the content to be
/// done independently of the layout of the surrounding `Cell`. The content's dimension is defined
/// using the [`CellDimension`] enum, which can be either a fixed [`Dimension`] or a [`CellWidth`]
/// that describes how the width is computed based on available space.
///
/// Since the dimension of the content is not necessarily related to the [`Cell`]'s dimension,
/// there is a need to configure the placement of the content within the cell. This is done using
/// the [`CellAnchor`] enum: depending on the anchor, the content is placed in the according cells
/// region or the content is trimmed to fit the cell.
///
/// This makes the `Cell` a container for a [`Layout`] that can be placed in larger contexts, such
/// as a [`Table`](crate::widgets::Table) or a [`Horizontal`](crate::widgets::Horizontal).
///
/// # Layout process
/// When laying out a `Cell`, the following steps are performed:
///
/// In a first step, the concrete dimension of the content is determined. Depending on the `dim`
/// of the `Cell`, this might be either a fixed [`Dimension`] or a [`CellWidth`]. In case of a
/// `CellWidth` some context-dependent calculation might take place to figure out the exact
/// dimensions; for details see the documentation of [`CellDimension`].
///
/// The resulting dimension might be larger or smaller than the dimension of the `Cell`. Depending
/// on that relation and the `anchor` field of the `Cell`, the content is either aligned or trimmed
/// to match the constraints of the anchor.
///
/// All these calculations also respect the `clip` field (which contains the optional clipping of
/// the content) and all the other settings of the [`LayoutOptions`] that is active during the layout
/// process.
///
/// # Example
/// ```rust
/// use termlayout::{Dimension, LayoutOptions, RcLayout, Layout};
/// use termlayout::widgets::{Cell, CellAnchor, CellWidth, Paragraph};
///
/// // The following has the dimension 12x8:
/// let content = Paragraph::left("This is the content with a width of 12.");
///
/// // Now we embed that in a cell...
/// let cell =  Cell::of(content)
///                     .with_width(CellWidth::Fixed(12))
///                     .with_anchor(CellAnchor::SouthEast);
///
/// // ... and we lay it out in a 16x6 area, so the content is laid out in the south east of the
/// // cell.
/// let options = LayoutOptions::default().with_dim(Dimension::new(16, 6));
///
/// let result = format!("{}", cell.layout_strict(options));
/// assert_eq!(result, concat!(
///     "\n",
///     "\n",
///     "    This is the\n",
///     "    content with\n",
///     "    a width of\n",
///     "    12.\n"
/// ));
/// ```
#[derive(Clone)]
pub struct Cell {
    /// The content of the cell. This is a [`RcLayout`] that can be any other kind of `Layout`.
    pub content: RcLayout,

    /// A [`CellDimension`] that defines the dimension of the cell. This might be either a
    /// fixed [`Dimension`] or a [`CellWidth`]; see the documentation of [`CellDimension`] for more
    /// information.
    pub dim: CellDimension,

    /// The [`CellAnchor`] that defines the placement of the content within in cell
    pub anchor: CellAnchor,

    /// The [`WrapMode`] that defines how the content is wrapped, optional (if omitted,
    /// the wrap mode of the cell is used).
    pub wrap_mode: Option<WrapMode>,

    /// An optional [`Rect`] that defines the clipping of the content.
    pub clip: Option<Rect>,
}

impl Cell {
    /// Creates a new cell with the given settings
    ///
    /// # Parameters
    /// - `content`: The content of the cell. This is a [`RcLayout`] that can be any other kind of
    ///   content.
    /// - `dim`: A [`CellDimension`] that defines the dimension of the cell. This might be either a
    ///   fixed [`Dimension`] or a [`CellWidth`]; see the documentation of [`CellDimension`] for more
    ///   information.
    /// - `wrap_mode`: The [`WrapMode`] that defines how the content is wrapped, optional (if omitted,
    ///   the wrap mode of the cell is used).
    /// - `clip`: An optional [`Rect`] that defines the clipping of the content.
    /// - `anchor`: The [`CellAnchor`] that defines the placement of the content within in cell
    ///
    /// # Returns
    /// A new [`Cell`] with the given settings
    #[must_use]
    pub fn new<T>(
        content: T,
        dim: CellDimension,
        wrap_mode: Option<WrapMode>,
        clip: Option<Rect>,
        anchor: CellAnchor,
    ) -> Self
    where
        T: Into<RcLayout>,
    {
        Self {
            content: content.into(),
            dim,
            anchor,
            wrap_mode,
            clip,
        }
    }

    /// Creates a new [`Cell`] with the given content and default settings.
    ///
    /// # Parameters
    /// - `content`: The content of the cell.
    ///
    /// # Returns
    /// A new [`Cell`] with the given content and default settings.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::widgets::{Cell, CellAnchor, CellDimension, CellWidth, Paragraph};
    ///
    /// let content = Paragraph::left("This is the content.");
    /// let cell = Cell::of(content);
    ///
    /// assert_eq!(cell.dim, CellDimension::Declarative(CellWidth::Fill));
    /// assert_eq!(cell.clip, None);
    /// assert_eq!(cell.wrap_mode, None);
    /// assert_eq!(cell.anchor, CellAnchor::NorthWest);
    /// ```
    #[must_use]
    pub fn of<T>(content: T) -> Self
    where
        T: Into<RcLayout>,
    {
        Self {
            content: content.into(),
            dim: CellDimension::Declarative(CellWidth::Fill),
            anchor: CellAnchor::NorthWest,
            wrap_mode: None,
            clip: None,
        }
    }

    /// Creates a new [`Cell`] with the given content and minimal width.
    ///
    /// # Parameters
    /// - `content`: The content of the cell.
    ///
    /// # Returns
    /// A new [`Cell`] with the given content and minimal width.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::widgets::{Cell, CellAnchor, CellDimension, CellWidth, Paragraph};
    ///
    /// let content = Paragraph::left("This is the content.");
    /// let cell = Cell::minimal(content);
    ///
    /// assert_eq!(cell.dim, CellDimension::Declarative(CellWidth::Minimal));
    /// assert_eq!(cell.clip, None);
    /// assert_eq!(cell.wrap_mode, None);
    /// assert_eq!(cell.anchor, CellAnchor::NorthWest);
    /// ```
    #[must_use]
    pub fn minimal<T>(content: T) -> Self
    where
        T: Into<RcLayout>,
    {
        Self {
            content: content.into(),
            dim: CellDimension::Declarative(CellWidth::Minimal),
            anchor: CellAnchor::NorthWest,
            wrap_mode: None,
            clip: None,
        }
    }
    /// Creates a new [`Cell`] with the given content as filler.
    ///
    /// # Parameters
    /// - `content`: The content of the cell.
    ///
    /// # Returns
    /// A new [`Cell`] with the given content and configured as filler.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::widgets::{Cell, CellAnchor, CellDimension, CellWidth, Paragraph};
    ///
    /// let content = Paragraph::left("This is the content.");
    /// let cell = Cell::fill(content);
    ///
    /// assert_eq!(cell.dim, CellDimension::Declarative(CellWidth::Minimal));
    /// assert_eq!(cell.clip, None);
    /// assert_eq!(cell.wrap_mode, None);
    /// assert_eq!(cell.anchor, CellAnchor::Fill);
    /// ```
    #[must_use]
    pub fn fill<T>(content: T) -> Self
    where
        T: Into<RcLayout>,
    {
        Self {
            content: content.into(),
            dim: CellDimension::Declarative(CellWidth::Minimal),
            anchor: CellAnchor::Fill,
            wrap_mode: None,
            clip: None,
        }
    }

    /// Sets the `dim` field of the [`Cell`] to the given [`CellWidth`].
    ///
    /// # Parameters
    /// - `width`: The `CellWidth`
    ///
    /// # Returns
    /// A new [`Cell`] with the `dim` field set to the given `width`
    ///
    ///
    /// # Example
    /// ```rust
    /// use termlayout::widgets::{Cell, CellDimension, CellWidth, Paragraph};
    ///
    /// let content = Paragraph::left("This is the content.");
    /// let cell = Cell::of(content)
    ///             .with_width(CellWidth::Proportional(0.5));
    ///
    /// assert_eq!(cell.dim, CellDimension::Declarative(CellWidth::Proportional(0.5)));
    /// ```
    #[must_use]
    pub fn with_width(self, width: CellWidth) -> Self {
        Self {
            dim: CellDimension::Declarative(width),
            ..self
        }
    }

    /// Sets the `dim` field of the [`Cell`] to the given [`Dimension`].
    /// Both, the `cell` and the `content` dimension are set to the given `dim`.
    ///
    /// # Parameters
    /// - `dim`: The `Dimension`
    ///
    /// # Returns
    /// A new [`Cell`] with the `dim` field set to the given `dim`
    ///
    /// # Example
    /// ```rust
    /// use termlayout::Dimension;
    /// use termlayout::widgets::{Cell, CellDimension, CellWidth, Paragraph};
    ///
    /// let content = Paragraph::left("This is the content.");
    /// let cell = Cell::of(content)
    ///             .with_dim(Dimension::new(20, 10));
    ///
    /// assert_eq!(cell.dim, CellDimension::Fixed{
    ///     cell: Dimension::new(20, 10),
    ///     content: Dimension::new(20, 10)
    /// });
    /// ```
    #[must_use]
    pub fn with_dim(self, dim: Dimension) -> Self {
        self.with_dims(dim, dim)
    }

    /// Sets the `dim` field of the [`Cell`] to the given pair of [`Dimension`]s.
    ///
    /// # Parameters
    /// - `cell`: The `Dimension` for the cell
    /// - `content`: The `Dimension` for the content
    ///
    /// # Returns
    /// A new [`Cell`] with the `dim` field set to the given dimensions.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::Dimension;
    /// use termlayout::widgets::{Cell, CellDimension, CellWidth, Paragraph};
    ///
    /// let content = Paragraph::left("This is the content.");
    /// let cell = Cell::of(content)
    ///             .with_dims(Dimension::new(20, 10), Dimension::new(16, 8));
    ///
    /// assert_eq!(cell.dim, CellDimension::Fixed{
    ///     cell: Dimension::new(20, 10),
    ///     content: Dimension::new(16, 8)
    /// });
    /// ```
    #[must_use]
    pub fn with_dims(self, cell: Dimension, content: Dimension) -> Self {
        Self {
            dim: CellDimension::Fixed { cell, content },
            ..self
        }
    }

    /// Sets the `wrap_mode` field of the [`Cell`] to the given [`WrapMode`].
    ///
    /// # Parameters
    /// - `wrap_mode`: The `WrapMode`
    ///
    /// # Returns
    /// A new [`Cell`] with the `wrap_mode` field set to the given `wrap_mode`
    ///
    /// # Example
    /// ```rust
    /// use termlayout::{Dimension, WrapMode};
    /// use termlayout::widgets::{Cell, CellDimension, CellWidth, Paragraph};
    ///
    /// let content = Paragraph::left("This is the content.");
    /// let cell = Cell::of(content)
    ///             .with_wrap_mode(Some(WrapMode::Wrap));
    ///
    /// assert_eq!(cell.wrap_mode, Some(WrapMode::Wrap));
    /// ```
    #[must_use]
    pub fn with_wrap_mode(self, wrap_mode: Option<WrapMode>) -> Self {
        Self { wrap_mode, ..self }
    }

    /// Sets the `anchor` field of the [`Cell`] to the given [`CellAnchor`].
    ///
    /// # Parameters
    /// - `anchor`: The `CellAnchor`
    ///
    /// # Returns
    /// A new [`Cell`] with the `anchor` field set to the given `CellAnchor`
    ///
    /// # Example
    /// ```rust
    /// use termlayout::Dimension;
    /// use termlayout::widgets::{Cell, CellDimension, CellWidth, CellAnchor, Paragraph};
    ///
    /// let content = Paragraph::left("This is the content.");
    /// let cell = Cell::of(content)
    ///             .with_anchor(CellAnchor::South);
    ///
    /// assert_eq!(cell.anchor, CellAnchor::South);
    /// ```
    #[must_use]
    pub fn with_anchor(self, anchor: CellAnchor) -> Self {
        Self { anchor, ..self }
    }

    /// Sets the `clip` field of the [`Cell`] to the given [`Rect`].
    ///
    /// # Parameters
    /// - `clip`: The clipping `Rect`
    ///
    /// # Returns
    /// A new [`Cell`] with the `clip` field set to the given `Rect`
    ///
    /// # Example
    /// ```rust
    /// use termlayout::{Dimension, Rect};
    /// use termlayout::widgets::{Cell, CellDimension, CellWidth, CellAnchor, Paragraph};
    ///
    /// let content = Paragraph::left("This is the content.");
    /// let cell = Cell::of(content)
    ///             .with_clip(Some(Rect::new(0, 0, Dimension::new(10, 10))));
    ///
    /// assert_eq!(cell.clip, Some(Rect::new(0, 0, Dimension::new(10, 10))));
    /// ```
    #[must_use]
    pub fn with_clip(self, clip: Option<Rect>) -> Self {
        Self { clip, ..self }
    }

    /// Determines the effective wrap mode by using the current wrap mode or a provided default.
    ///
    /// This method checks if the `wrap_mode` of the current instance is set.
    /// If it is set (`Some`), it returns that value. If it is not set (`None`),
    /// it falls back to the provided `wrap_mode` parameter as the default.
    ///
    /// # Parameters
    /// - `wrap_mode`: A `WrapMode` value to use as the default if the current `wrap_mode` is `None`.
    ///
    /// # Returns
    /// - A `WrapMode` value that is either the current `wrap_mode` or the provided default `wrap_mode`.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::widgets::{Cell, Paragraph};
    /// use termlayout::WrapMode;
    ///
    /// let mut cell = Cell::of(Paragraph::left("This is the content."));
    ///
    /// cell.wrap_mode = Some(WrapMode::Wrap);
    /// assert_eq!(cell.effective_wrap_mode(WrapMode::default_truncate()), WrapMode::Wrap);
    ///
    /// cell.wrap_mode = None;
    /// assert_eq!(cell.effective_wrap_mode(WrapMode::default_truncate()), WrapMode::default_truncate());
    /// ```
    #[must_use]
    pub fn effective_wrap_mode(&self, wrap_mode: WrapMode) -> WrapMode {
        self.wrap_mode.unwrap_or(wrap_mode)
    }

    /// Calculates the cell and content dimension based on the provided maximum width and wrap mode.
    ///
    /// # Parameters
    /// - `max_width`: The maximum width in terms of columns; if set to `None`, the dimension is
    ///   calculated based on the minimum size of the content.
    /// - `wrap_mode`: The default [`WrapMode`] that is used in case the cell's wrap mode is not set.
    ///
    /// # Returns
    /// The calculated dimension of the cell and content based on the provided parameters.
    /// The first is the cell's dimension and the second is the content's dimension.
    #[must_use]
    pub fn calculate_dims(
        &self,
        max_width: Option<usize>,
        wrap_mode: WrapMode,
    ) -> (Dimension, Dimension) {
        let wrap_mode = self.effective_wrap_mode(wrap_mode);
        self.dim.calculate_dims(&self.content, max_width, wrap_mode)
    }

    /// Returns the visible content rectangle of the cell.
    /// This is the intersection of the content's dimension and the cell's clip rectangle, if set.
    /// If the cell has no clip rectangle, it returns the content's dimension. If the content
    /// dimension is not known, it panics.
    ///
    /// # Returns
    /// The visible content rectangle of the cell.
    ///
    /// # Panics
    /// Panics if the concrete content dimension is not known for calculation.
    #[must_use]
    pub fn visible_content(&self) -> Rect {
        let content_rect: Rect = self
            .dim
            .dims()
            .expect("cell dimension must be known for calculation")
            .0
            .into();
        self.clip.unwrap_or(content_rect).intersect(content_rect)
    }

    /// Truncates this cell horizontally at the given width.
    /// The truncation influences just the clipping area of the cell, not its content dimension.
    ///
    /// # Parameters
    /// - `width`: The width to truncate the cell at.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::{Dimension, Rect};
    /// use termlayout::widgets::{Cell, CellDimension, Lines};
    ///
    /// let mut cell = Cell::of(Lines::left("This is the content."))
    ///             .with_dim(Dimension::new(10, 10));
    /// assert_eq!(cell.dim, CellDimension::Fixed{
    ///     cell: Dimension::new(10, 10),
    ///     content: Dimension::new(10, 10)
    /// });
    /// assert_eq!(cell.visible_content(), Rect::new(0, 0, Dimension::new(10, 10)));
    ///
    /// cell.truncate_horizontal(5);
    /// assert_eq!(cell.dim, CellDimension::Fixed{
    ///     cell: Dimension::new(10, 10),
    ///     content: Dimension::new(10, 10)
    /// });
    /// assert_eq!(cell.visible_content(), Rect::new(0, 0, Dimension::new(5, 10)));
    /// ```
    pub fn truncate_horizontal(&mut self, width: usize) {
        let mut clip = self.clip.unwrap_or_else(|| Dimension::MAX.into());
        clip.dim.width = min(width, clip.dim.width);
        self.clip = Some(clip);
    }

    /// Splits this cell horizontally at the given `width` into two portions.
    /// The splitting influences just the clipping area of the cell, not its content dimension.
    ///
    /// # Parameters
    /// - `width`: The width to split the cell at.
    ///
    /// # Returns
    /// `(left, right)`: A tuple containing two cells, one with the left portion and the other
    /// with the right portion.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::{Dimension, Rect};
    /// use termlayout::widgets::{Cell, CellDimension, Lines};
    ///
    /// let cell = Cell::of(Lines::left("This is the content."))
    ///             .with_dim(Dimension::new(10, 10));
    /// assert_eq!(cell.dim, CellDimension::Fixed{
    ///     cell: Dimension::new(10, 10),
    ///     content: Dimension::new(10, 10)
    /// });
    /// assert_eq!(cell.visible_content(), Rect::new(0, 0, Dimension::new(10, 10)));
    ///
    /// let (left, right) = cell.split_horizontal(5);
    /// assert_eq!(cell.dim, CellDimension::Fixed{
    ///     cell: Dimension::new(10, 10),
    ///     content: Dimension::new(10, 10)
    /// });
    /// assert_eq!(left.visible_content(), Rect::new(0, 0, Dimension::new(5, 10)));
    /// assert_eq!(cell.dim, CellDimension::Fixed{
    ///     cell: Dimension::new(10, 10),
    ///     content: Dimension::new(10, 10)
    /// });
    /// assert_eq!(right.visible_content(), Rect::new(5, 0, Dimension::new(5, 10)));
    ///```
    #[must_use]
    pub fn split_horizontal(&self, width: usize) -> (Self, Self) {
        let clip = self.clip.unwrap_or_else(|| Dimension::MAX.into());
        let (left_clip, right_clip) = clip.split_horizontal(width);
        let left = self.clone().with_clip(Some(left_clip));
        let right = self.clone().with_clip(Some(right_clip));
        (left, right)
    }

    /// Truncates this cell vertically at the given height.
    /// The truncation influences just the clipping area of the cell, not its content dimension.
    ///
    /// # Parameters
    /// - `height`: The height to truncate the cell at.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::{Dimension, Rect};
    /// use termlayout::widgets::{Cell, CellDimension, Lines};
    ///
    /// let mut cell = Cell::of(Lines::left("This is the content."))
    ///             .with_dim(Dimension::new(10, 10));
    /// assert_eq!(cell.dim, CellDimension::Fixed{
    ///     cell: Dimension::new(10, 10),
    ///     content: Dimension::new(10, 10)
    /// });
    /// assert_eq!(cell.visible_content(), Rect::new(0, 0, Dimension::new(10, 10)));
    ///
    /// cell.truncate_vertical(5);
    /// assert_eq!(cell.dim, CellDimension::Fixed{
    ///     cell: Dimension::new(10, 10),
    ///     content: Dimension::new(10, 10)
    /// });
    /// assert_eq!(cell.visible_content(), Rect::new(0, 0, Dimension::new(10, 5)));
    /// ```
    pub fn truncate_vertical(&mut self, height: usize) {
        let mut clip = self.clip.unwrap_or_else(|| Dimension::MAX.into());
        clip.dim.height = min(height, clip.dim.height);
        self.clip = Some(clip);
    }

    /// Splits this cell vertically at the given `height` into two portions.
    /// The splitting influences just the clipping area of the cell, not its content dimension.
    ///
    /// # Parameters
    /// - `height`: The height to split the cell at.
    ///
    /// # Returns
    /// `(top, bottom)`: A tuple containing two cells, one with the top portion and the other
    /// with the bottom portion.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::{Dimension, Rect};
    /// use termlayout::widgets::{Cell, CellDimension, Lines};
    ///
    /// let cell = Cell::of(Lines::left("This is the content."))
    ///             .with_dim(Dimension::new(10, 10));
    /// assert_eq!(cell.dim, CellDimension::Fixed{
    ///     cell: Dimension::new(10, 10),
    ///     content: Dimension::new(10, 10)
    /// });
    /// assert_eq!(cell.visible_content(), Rect::new(0, 0, Dimension::new(10, 10)));
    ///
    /// let (top, bottom) = cell.split_vertical(5);
    /// assert_eq!(cell.dim, CellDimension::Fixed{
    ///     cell: Dimension::new(10, 10),
    ///     content: Dimension::new(10, 10)
    /// });
    /// assert_eq!(top.visible_content(), Rect::new(0, 0, Dimension::new(10, 5)));
    /// assert_eq!(cell.dim, CellDimension::Fixed{
    ///     cell: Dimension::new(10, 10),
    ///     content: Dimension::new(10, 10)
    /// });
    /// assert_eq!(bottom.visible_content(), Rect::new(0, 5, Dimension::new(10, 5)));
    ///```
    #[must_use]
    pub fn split_vertical(&self, height: usize) -> (Self, Self) {
        let clip = self.clip.unwrap_or_else(|| Dimension::MAX.into());
        let (top_clip, bottom_clip) = clip.split_vertical(height);
        let top = self.clone().with_clip(Some(top_clip));
        let bottom = self.clone().with_clip(Some(bottom_clip));
        (top, bottom)
    }
}

impl From<RcLayout> for Cell {
    fn from(layout: RcLayout) -> Self {
        Self {
            content: layout,
            dim: CellDimension::Declarative(CellWidth::Fill),
            anchor: CellAnchor::NorthWest,
            wrap_mode: None,
            clip: None,
        }
    }
}

impl Layout for Cell {
    fn pref_dim(&self, max_width: usize, wrap_mode: WrapMode) -> Dimension {
        let (mut dim, _) = self.calculate_dims(Some(max_width), wrap_mode);
        dim.width = min(dim.width, max_width);
        dim
    }

    fn min_dim(&self) -> Dimension {
        self.calculate_dims(None, WrapMode::Wrap).0
    }

    fn measure(&self, mode: MeasureMode) -> Measurements {
        todo!()
    }

    fn layout_strict(&'_ self, options: LayoutOptions) -> BoxedFormattedLayout<'_> {
        let (_, content_dim) = self.calculate_dims(Some(options.dim.width), options.wrap_mode);
        let metrics = metrics::CellMetrics::new(&options, content_dim, self.clip, self.anchor);
        let content_options = metrics.content_options(
            options.fill_rows,
            self.effective_wrap_mode(options.wrap_mode),
        );
        let cell_options = metrics.cell_options(options.fill_rows, options.wrap_mode);

        let formatted_content = self.content.layout_strict(content_options);
        Box::new(formatted::FormattedCell::new(
            formatted_content,
            metrics.padding,
            cell_options,
        ))
    }

    fn layout_with_context(&'_ self, context: LayoutContext) -> BoxedFormattedLayout<'_> {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

rc_layout!(Cell);

/// Describes the anchor point of the content with a [`Cell`].
/// In case that the content is smaller than the cell, the cell contains according padding to
/// align the content correctly; in case it is larger, the content is trimmed/clipped accodingly
///
/// A special variant is `Fill` which actually means that the content is stretched to fill the cell.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum CellAnchor {
    /// You see the visualization of the anchor in case the content has size 2x2 and the cell is 6x4:
    ///   ```text
    ///   +------+
    ///   |  CC  |
    ///   |  CC  |
    ///   |      |
    ///   |      |
    ///   +------+
    ///   ```
    North,

    /// You see the visualization of the anchor in case the content has size 2x2 and the cell is 6x4:
    ///   ```text
    ///   +------+
    ///   |    CC|
    ///   |    CC|
    ///   |      |
    ///   |      |
    ///   +------+
    ///   ```
    NorthEast,

    /// You see the visualization of the anchor in case the content has size 2x2 and the cell is 6x4:
    ///   ```text
    ///   +------+
    ///   |      |
    ///   |    CC|
    ///   |    CC|
    ///   |      |
    ///   +------+
    ///   ```
    East,

    /// You see the visualization of the anchor in case the content has size 2x2 and the cell is 6x4:
    ///   ```text
    ///   +------+
    ///   |      |
    ///   |      |
    ///   |    CC|
    ///   |    CC|
    ///   +------+
    ///   ```
    SouthEast,

    /// You see the visualization of the anchor in case the content has size 2x2 and the cell is 6x4:
    ///   ```text
    ///   +------+
    ///   |      |
    ///   |      |
    ///   |  CC  |
    ///   |  CC  |
    ///   +------+
    ///   ```
    South,

    /// You see the visualization of the anchor in case the content has size 2x2 and the cell is 6x4:
    ///   ```text
    ///   +------+
    ///   |      |
    ///   |      |
    ///   |CC    |
    ///   |CC    |
    ///   +------+
    ///   ```
    SouthWest,

    /// You see the visualization of the anchor in case the content has size 2x2 and the cell is 6x4:
    ///   ```text
    ///   +------+
    ///   |      |
    ///   |CC    |
    ///   |CC    |
    ///   |      |
    ///   +------+
    ///   ```
    West,

    /// You see the visualization of the anchor in case the content has size 2x2 and the cell is 6x4:
    ///   ```text
    ///   +------+
    ///   |CC    |
    ///   |CC    |
    ///   |      |
    ///   |      |
    ///   +------+
    ///   ```
    #[default]
    NorthWest,

    /// You see the visualization of the anchor in case the content has size 2x2 and the cell is 6x4:
    ///   ```text
    ///   +------+
    ///   |      |
    ///   |  CC  |
    ///   |  CC  |
    ///   |      |
    ///   +------+
    ///   ```
    Center,

    /// You see the visualization of the anchor in case the content has size 2x2 and the cell is 6x4:
    ///   ```text
    ///   +------+
    ///   |CCCCCC|
    ///   |CCCCCC|
    ///   |CCCCCC|
    ///   |CCCCCC|
    ///   |CCCCCC|
    ///   |CCCCCC|
    ///   +------+
    ///   ```
    Fill,
}

impl CellAnchor {
    /// Returns the padding or clipping factors for the anchor.
    /// The behavior is best explained by an example: consider an area of 10x10 units where you want
    /// to place a content with the measurements of 6x6 units. If the anchor was `East`, you
    /// need to place the content at coordinate (4,2) relative to the area.
    /// The factors returned by this function allow calculating this coordinate like shown in the
    /// example below
    ///
    /// # Returns
    /// The factor tuple
    ///
    /// # Example
    /// ```rust
    /// use termlayout::Dimension;
    /// use termlayout::widgets::CellAnchor;
    ///
    /// let area = Dimension::new(10, 10);
    /// let content = Dimension::new(6, 6);
    ///
    /// let factors = CellAnchor::East.factors();
    ///
    /// assert_eq!((area.width-content.width)*factors.0/2, 4);
    /// assert_eq!((area.height-content.height)*factors.1/2, 2);
    /// ```
    #[must_use]
    pub fn factors(&self) -> (usize, usize) {
        match self {
            CellAnchor::North => (1, 0),
            CellAnchor::NorthEast => (2, 0),
            CellAnchor::East => (2, 1),
            CellAnchor::SouthEast => (2, 2),
            CellAnchor::South => (1, 2),
            CellAnchor::SouthWest => (0, 2),
            CellAnchor::West => (0, 1),
            CellAnchor::Center => (1, 1),
            _ => (0, 0),
        }
    }
}

/// Defines the dimension of the content of a [`Cell`].
/// The dimension might be a plain fixed [`Dimension`], which provides concrete values for
/// the width and height, or it might be a [`CellWidth`] that describes how the width is computed.
///
/// Instances of this enum can be created using the `From` trait implementation,
/// where the source is either a [`Dimension`] or a [`CellWidth`].
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CellDimension {
    /// The dimension of the cell and its content is fixed and defined by the given [`Dimension`].
    Fixed {
        /// The [`Dimension`] of the cell.
        cell: Dimension,
        /// The [`Dimension`] of the content.
        content: Dimension,
    },

    /// The dimension of the cell is computed based on the given [`CellWidth`]. The exact sizing
    /// behavior depends on the usage.
    Declarative(CellWidth),
}

impl CellDimension {
    /// Returns the [`Dimension`]s of the cell and content, if known.
    /// For the `Fixed` variant it returns the `Dimension`s, otherwise `None`
    ///
    /// # Returns
    /// The `Dimension`s (cell, content) or `None`
    #[must_use]
    pub fn dims(&self) -> Option<(Dimension, Dimension)> {
        match self {
            CellDimension::Fixed { cell, content } => Some((*cell, *content)),
            CellDimension::Declarative(_) => None,
        }
    }

    /// Returns the declarative [`CellWidth`] of the dimension.
    /// If the dimension is fixed, returns `CellWidth::Fixed` with the fixed dimension's width
    ///
    /// # Returns
    /// The [`CellWidth`]
    ///
    /// # Example
    /// ```rust
    ///
    /// use termlayout::Dimension;
    /// use termlayout::widgets::{CellDimension, CellWidth};
    ///
    /// let dim = CellDimension::Declarative(CellWidth::Minimal);
    /// assert_eq!(dim.cell_width(), CellWidth::Minimal);
    ///
    /// let dim = CellDimension::Fixed{cell: Dimension::new(17, 13), content: Dimension::new(11, 9)};
    /// assert_eq!(dim.cell_width(), CellWidth::Fixed(17));
    /// ```
    #[must_use]
    pub fn cell_width(&self) -> CellWidth {
        match self {
            CellDimension::Fixed { cell, content } => {
                if cell.width >= content.width {
                    CellWidth::Fixed(cell.width)
                } else {
                    CellWidth::Preferred(cell.width)
                }
            }
            CellDimension::Declarative(width) => *width,
        }
    }

    /// Checks, whether this instance is a fill dimension
    ///
    /// # Returns
    /// `true`, if this is a `CellDimension::Declarative(CellWidth::Fill)`
    #[must_use]
    pub fn is_fill(&self) -> bool {
        matches!(self, CellDimension::Declarative(CellWidth::Fill))
    }

    /// Calculates the cell and content [`Dimension`] for the given `content`, `max_width`, and
    /// `wrap_mode`.
    ///
    /// # Parameters
    /// - `content`: The [`RcLayout`] of the cell content being laid out with this
    ///   [`CellWidth`].
    /// - `max_width`: The maximum width in terms of columns; if set to `None`, the dimension is
    ///   calculated based on the minimum size of the content.
    /// - `wrap_mode`: The default [`WrapMode`] that is used in case the cell's wrap mode is not set.
    ///
    /// # Returns
    /// The calculated [`Dimension`] for the cell and content.
    #[must_use]
    pub fn calculate_dims(
        &self,
        content: &RcLayout,
        max_width: Option<usize>,
        wrap_mode: WrapMode,
    ) -> (Dimension, Dimension) {
        match self {
            CellDimension::Fixed { cell, content } => (*cell, *content),
            CellDimension::Declarative(width) => {
                width.calculate_dims(content, max_width, wrap_mode)
            }
        }
    }
}

impl From<Dimension> for CellDimension {
    fn from(dim: Dimension) -> Self {
        CellDimension::Fixed {
            cell: dim,
            content: dim,
        }
    }
}

impl From<CellWidth> for CellDimension {
    fn from(width: CellWidth) -> Self {
        CellDimension::Declarative(width)
    }
}

/// Defines the width of the content of a [`Cell`].
/// Using this enum, it is possible to define the width of the cell's content as a fixed value or a
/// dynamic one that depends on the available space and usage.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum CellWidth {
    /// The width of the cell content is the same as the minimum width of its content, as
    /// returned by [`min_dim()`](Layout::min_dim)
    #[default]
    Minimal,

    /// The width of the cell content is exactly the width specified with `value`.
    Fixed(usize),

    /// The width of the cell content is the same as the preferred width of the content,
    /// as returned by [`pref_dim()`](Layout::pref_dim) for the given `value`. Note that this is a
    /// subtile way different from `Fixed`, since the actual width might be smaller than `value`.
    Preferred(usize),

    /// The width of the cell content is proportional to the width of the
    /// entire available width. It depends on the usage what the entire available width is.
    /// The actual width is `value` times the width of the available width, so if - for example - the
    /// available width has a width of 100, and the cell is set to `Proportional(0.5)`, the cell
    /// will have a width of 50.
    Proportional(f32),

    /// The width of the cell content is calculated so that it fills the available width.
    /// Depending on the usage - for example in a table - the available width is the the width of
    /// a table row without the widths of the other cells in the row, devided by the number of cells
    /// with the `Fill` width.
    Fill,
}

impl CellWidth {
    /// Calculates the cell and content [`Dimension`] for the given `content`, `max_width`, and
    /// `wrap_mode`.
    ///
    /// # Parameters
    /// - `content`: The [`RcLayout`] of the cell content being laid out with this
    ///   [`CellWidth`].
    /// - `max_width`: The maximum width in terms of columns; if set to `None`, the dimension is
    ///   calculated based on the minimum size of the content.
    /// - `wrap_mode`: The default [`WrapMode`] that is used in case the cell's wrap mode is not set.
    ///
    /// # Returns
    /// The calculated [`Dimension`] for the cell and content.
    #[allow(clippy::missing_panics_doc)]
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_precision_loss)]
    #[must_use]
    pub fn calculate_dims(
        &self,
        content: &RcLayout,
        max_width: Option<usize>,
        wrap_mode: WrapMode,
    ) -> (Dimension, Dimension) {
        match self {
            CellWidth::Fixed(w) => {
                let dim = content.pref_dim_fixed_width(*w, wrap_mode);
                (dim, dim)
            }
            CellWidth::Proportional(p) if max_width.is_some() => {
                let w = (max_width.unwrap() as f32 * p) as usize;
                let content = content.pref_dim(w, wrap_mode);
                let cell = Dimension::new(w, content.height);
                (cell, content)
            }
            CellWidth::Fill if max_width.is_some() => {
                let w = max_width.unwrap();
                let content = content.pref_dim(w, wrap_mode);
                let cell = Dimension::new(w, content.height);
                (cell, content)
            }
            CellWidth::Preferred(w) => {
                let content = content.pref_dim(*w, wrap_mode);
                let cell = Dimension::new(*w, content.height);
                (cell, content)
            }
            _ => {
                let dim = content.min_dim();
                (dim, dim)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::Lines;

    #[test]
    fn cell_pref_dim() {
        let content: RcLayout = Lines::left("abcde\nfghij").into();

        let cell = Cell::new(
            content.clone(),
            CellDimension::Declarative(CellWidth::Fixed(4)),
            None,
            None,
            CellAnchor::Center,
        );
        assert_eq!(cell.pref_dim(4, WrapMode::Wrap), Dimension::new(4, 4));

        let cell = Cell::new(
            content.clone(),
            CellDimension::Declarative(CellWidth::Fixed(10)),
            None,
            None,
            CellAnchor::NorthWest,
        );
        assert_eq!(cell.pref_dim(20, WrapMode::Wrap), Dimension::new(10, 2));
    }

    #[test]
    fn cell_min_dim() {
        let content: RcLayout = Lines::left("abcde\nfghij").into();

        let cell = Cell::new(
            content.clone(),
            CellDimension::Declarative(CellWidth::Minimal),
            None,
            None,
            CellAnchor::Center,
        );
        assert_eq!(cell.min_dim(), Dimension::new(5, 2));

        let cell = Cell::new(
            content.clone(),
            CellDimension::Declarative(CellWidth::Fixed(10)),
            None,
            None,
            CellAnchor::NorthWest,
        );
        assert_eq!(cell.min_dim(), Dimension::new(10, 2));
    }

    #[test]
    fn cell_layout_content_fit_no_inner_clip_no_outer_clip() {
        // Arrange
        let content = Lines::left(concat!(
            "abcdefg\n", //
            "hijklmn\n", //
            "opqrstu\n", //
            "vwxyz01\n", //
            "2345678"
        ));
        let cell = Cell::new(
            content,
            CellDimension::Declarative(CellWidth::Fixed(7)),
            None,
            None,
            CellAnchor::Center,
        );

        // No fill
        let options = LayoutOptions::default().with_dim(Dimension::new(11, 7));
        let result = format!("{}", cell.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "\n",          //
                "  abcdefg\n", //
                "  hijklmn\n", //
                "  opqrstu\n", //
                "  vwxyz01\n", //
                "  2345678\n", //
                "\n"           //
            )
        );

        // With fill
        let options = LayoutOptions::default()
            .with_fill_rows(true)
            .with_dim(Dimension::new(11, 7));
        let result = format!("{}", cell.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "           \n", //
                "  abcdefg  \n", //
                "  hijklmn  \n", //
                "  opqrstu  \n", //
                "  vwxyz01  \n", //
                "  2345678  \n", //
                "           \n"  //
            )
        );
    }

    #[test]
    fn cell_layout_content_fit_with_inner_clip_no_outer_clip() {
        // Arrange
        let content = Lines::left(concat!(
            "abcdefg\n", //
            "hijklmn\n", //
            "opqrstu\n", //
            "vwxyz01\n", //
            "2345678"
        ));
        let cell = Cell::new(
            content,
            CellDimension::Declarative(CellWidth::Fixed(7)),
            None,
            Some(Rect::new(1, 1, Dimension::new(5, 3))),
            CellAnchor::Center,
        );

        let options = LayoutOptions::default()
            .with_fill_rows(true)
            .with_dim(Dimension::new(11, 7));
        let result = format!("{}", cell.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "           \n", //
                "           \n", //
                "   ijklm   \n", //
                "   pqrst   \n", //
                "   wxyz0   \n", //
                "           \n", //
                "           \n"  //
            )
        );
    }

    #[test]
    fn cell_layout_content_fit_no_inner_clip_with_outer_clip() {
        // Arrange
        let content = Lines::left(concat!(
            "abcdefg\n", //
            "hijklmn\n", //
            "opqrstu\n", //
            "vwxyz01\n", //
            "2345678"
        ));
        let cell = Cell::new(
            content,
            CellDimension::Declarative(CellWidth::Fixed(7)),
            None,
            None,
            CellAnchor::Center,
        );

        let options = LayoutOptions::default()
            .with_fill_rows(true)
            .with_dim(Dimension::new(11, 7))
            .with_clip(Some(Rect::new(2, 1, Dimension::new(6, 3))));
        let result = format!("{}", cell.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "abcdef\n", //
                "hijklm\n", //
                "opqrst\n", //
            )
        );
    }

    #[test]
    fn cell_layout_content_fit_with_inner_clip_with_outer_clip() {
        // Arrange
        let content = Lines::left(concat!(
            "abcdefg\n", //
            "hijklmn\n", //
            "opqrstu\n", //
            "vwxyz01\n", //
            "2345678"
        ));
        let cell = Cell::new(
            content,
            CellDimension::Declarative(CellWidth::Fixed(7)),
            None,
            Some(Rect::new(1, 1, Dimension::new(5, 3))),
            CellAnchor::Center,
        );

        let options = LayoutOptions::default()
            .with_fill_rows(true)
            .with_dim(Dimension::new(11, 7))
            .with_clip(Some(Rect::new(1, 1, Dimension::new(6, 3))));
        let result = format!("{}", cell.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "      \n", //
                "  ijkl\n", //
                "  pqrs\n", //
            )
        );
    }

    #[test]
    fn cell_layout_content_no_fit_no_inner_clip_no_outer_clip() {
        // Arrange
        let content = Lines::left(concat!(
            "abcdefg\n", //
            "hijklmn\n", //
            "opqrstu\n", //
            "vwxyz01\n", //
            "2345678"
        ));
        let cell = Cell::new(
            content,
            CellDimension::Declarative(CellWidth::Fixed(7)),
            None,
            None,
            CellAnchor::Center,
        );

        let options = LayoutOptions::default().with_dim(Dimension::new(5, 3));
        let result = format!("{}", cell.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "ijklm\n", //
                "pqrst\n", //
                "wxyz0\n", //
            )
        );
    }

    #[test]
    fn cell_layout_content_no_fit_with_inner_clip_no_outer_clip() {
        // Arrange
        let content = Lines::left(concat!(
            "abcdefghijklmn\n", //
            "opqrstuvwxyz01\n", //
            "23456789ABCDEF\n", //
            "GHIJKLMNOPQRST\n", //
            "UVWXYZ01234567\n", //
            "89abcdeefghijk\n", //
            "lmnopqrstuvwxy\n", //
        ));
        let cell = Cell::new(
            content,
            CellDimension::Declarative(CellWidth::Fixed(14)),
            None,
            Some(Rect::new(1, 1, Dimension::new(11, 5))),
            CellAnchor::Center,
        );

        let options = LayoutOptions::default().with_dim(Dimension::new(5, 3));
        let result = format!("{}", cell.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "6789A\n", //
                "KLMNO\n", //
                "YZ012\n", //
            )
        );
    }

    #[test]
    fn cell_layout_content_no_fit_with_inner_clip_with_outer_clip() {
        // Arrange
        let content = Lines::left(concat!(
            "abcdefghijklmn\n", //
            "opqrstuvwxyz01\n", //
            "23456789ABCDEF\n", //
            "GHIJKLMNOPQRST\n", //
            "UVWXYZ01234567\n", //
            "89abcdeefghijk\n", //
            "lmnopqrstuvwxy\n", //
        ));
        let cell = Cell::new(
            content,
            CellDimension::Declarative(CellWidth::Fixed(14)),
            None,
            Some(Rect::new(1, 1, Dimension::new(11, 5))),
            CellAnchor::Center,
        );

        let options = LayoutOptions::default()
            .with_dim(Dimension::new(5, 3))
            .with_clip(Some(Rect::new(1, 1, Dimension::new(3, 2))));
        let result = format!("{}", cell.layout_strict(options));
        assert_eq!(
            result,
            concat!(
                "LMN\n", //
                "Z01\n", //
            )
        );
    }
}
