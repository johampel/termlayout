use crate::widgets::horizontal::row::Row;
use crate::{Dimension, RcLayout, WrapMode};
use std::any::Any;
use std::cmp::min;
use std::rc::Rc;

/// Defines the different ways a [`Layout`](crate::Layout) can be [measured](crate::Layout::measure).
///
/// The result of the `measure` method is a [`Measurements`](Measurements) struct which
/// contains at least the [`Dimension`] of the `Layout` and - depending on the structure - further
/// sizing information that allows rendering the `Layout` correctly.
#[derive(Debug, Clone, Copy)]
pub enum MeasureMode {
    /// The `Min` sizing way tries to find the smallest possible size for the
    /// [`Layout`](crate::Layout) in terms of the width so that no truncation or wrapping is
    /// required. For example, for [`Lines`](crate::widgets::Lines) it will return a
    /// [`Measurements`] struct with a [`Dimension`] having the width set to
    /// the longest line and the height to the number of lines. For [`Paragraph`](crate::widgets::Paragraph)
    /// the dimension will have the width set to the longest word and the height is computed
    /// accordingly.
    Min,
    /// The `PrefWidth` sizing way tries to determine the best width for the [`Layout`](crate::Layout)
    /// so that the given `max_width` is never exceeded, but - if possible - the content is neither
    /// truncated nor wrapped. So the returned [`Measurements`] has a [`Dimension`] not bigger than
    /// the given `max_width` and a `height` depending on that width.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::{Dimension, MeasureMode, WrapMode, Layout};
    /// use termlayout::widgets::Paragraph;
    ///
    /// let layout = Paragraph::left("Hello nice World!"); // 17 display characters
    ///
    /// let mode = MeasureMode::pref_width(100, WrapMode::Wrap);
    /// assert_eq!(layout.measure(mode).dim, Dimension::new(17, 1)); // All fits in one line
    ///
    /// let mode = MeasureMode::pref_width(10, WrapMode::Wrap);
    /// assert_eq!(layout.measure(mode).dim, Dimension::new(10, 2)); // "Hello nice" and "World!"
    ///
    /// let mode = MeasureMode::pref_width(6, WrapMode::Wrap);
    /// assert_eq!(layout.measure(mode).dim, Dimension::new(6, 3)); // "Hello", "nice", and "World!"
    ///
    /// let mode = MeasureMode::pref_width(4, WrapMode::Wrap);
    /// assert_eq!(layout.measure(mode).dim, Dimension::new(4, 5)); // "Hell", "o", "nice", "Worl", and "d!"
    ///
    /// let mode = MeasureMode::pref_width(4, WrapMode::default_truncate());
    /// assert_eq!(layout.measure(mode).dim, Dimension::new(4, 3)); // "Hel…", "nice", and, "Wor…"
    ///
    /// ```
    PrefWidth {
        /// The maximum available display width
        max_width: usize,
        /// The [`WrapMode`]
        wrap_mode: WrapMode,
    },

    /// The `FixedWidth` sizing mode behaves similar to the `PrefWidth`, but returns always a
    /// [`Measurements`] having a [`Dimension`] with the specifed `width`.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::{Dimension, MeasureMode, WrapMode, Layout};
    /// use termlayout::widgets::Paragraph;
    ///
    /// let layout = Paragraph::left("Hello nice World!"); // 17 display characters
    ///
    /// let mode = MeasureMode::fixed_width(100, WrapMode::Wrap);
    /// assert_eq!(layout.measure(mode).dim, Dimension::new(100, 1));
    ///
    /// let mode = MeasureMode::fixed_width(17, WrapMode::Wrap);
    /// assert_eq!(layout.measure(mode).dim, Dimension::new(17, 1));
    ///
    /// let mode = MeasureMode::fixed_width(16, WrapMode::Wrap);
    /// assert_eq!(layout.measure(mode).dim, Dimension::new(16, 2));
    /// ```
    FixedWidth {
        /// The display width
        width: usize,
        /// The [`WrapMode`]
        wrap_mode: WrapMode,
    },
    /// The [`Exact`](MeasureMode::Exact) sizing mode predefines the [`Dimension`] of the [`Measurements`]. The
    /// given `dimension` and `wrap_mode` might influence the internal specific sizings stored
    /// in the [`MeasurementSpecifics`], if any.
    Exact {
        /// The [`Dimension`]
        dimension: Dimension,
        /// The [`WrapMode`]
        wrap_mode: WrapMode,
    },
}

impl MeasureMode {
    /// Creates a [`Min`](MeasureMode::Min) variant.
    ///
    /// # Returns
    /// The `MeasureMode`.
    #[must_use]
    pub const fn min() -> Self {
        Self::Min
    }

    /// Creates a [`PrefWidth`](MeasureMode::PrefWidth) variant.
    ///
    /// # Parameters
    /// - `max_width`: The maximum available display width
    /// - `wrap_mode`: The [`WrapMode`]
    ///
    /// # Returns
    /// The `MeasureMode`.
    #[must_use]
    pub const fn pref_width(max_width: usize, wrap_mode: WrapMode) -> Self {
        Self::PrefWidth {
            max_width,
            wrap_mode,
        }
    }

    /// Creates a [`FixedWidth`](MeasureMode::FixedWidth) variant.
    ///
    /// # Parameters
    /// - `width`: The display width
    /// - `wrap_mode`: The [`WrapMode`]
    ///
    /// # Returns
    /// The `MeasureMode`.
    #[must_use]
    pub const fn fixed_width(width: usize, wrap_mode: WrapMode) -> Self {
        Self::FixedWidth { width, wrap_mode }
    }

    /// Creates a [`Exact`](MeasureMode::Exact) variant.
    ///
    /// # Parameters
    /// - `dimension`: The [`Dimension`]
    /// - `wrap_mode`: The [`WrapMode`]
    ///
    /// # Returns
    /// The `MeasureMode`.
    #[must_use]
    pub const fn exact(dimension: Dimension, wrap_mode: WrapMode) -> Self {
        Self::Exact {
            dimension,
            wrap_mode,
        }
    }

    /// Returns the [`WrapMode`] of the [`MeasureMode`].
    /// The `WrapMode` for the `Min` variant is the default `WrapMode`; for the other the mode
    /// is explictly given.
    ///
    /// # Returns
    /// The [`WrapMode`].
    #[must_use]
    pub fn wrap_mode(&self) -> WrapMode {
        match self {
            Self::Min => WrapMode::default(),
            Self::PrefWidth { wrap_mode, .. }
            | Self::FixedWidth { wrap_mode, .. }
            | Self::Exact { wrap_mode, .. } => *wrap_mode,
        }
    }

    /// Returns, if defined, the (maximum) width.
    /// There is a width for all variants except `Min`.
    ///
    /// # Returns
    /// The width or `None`.
    #[must_use]
    pub fn width(&self) -> Option<usize> {
        match self {
            Self::Min => None,
            Self::PrefWidth { max_width, .. } => Some(*max_width),
            Self::FixedWidth { width, .. } => Some(*width),
            Self::Exact { dimension, .. } => Some(dimension.width),
        }
    }

    /// Coerces a concrete width.
    /// Depending on the variant and the `width` parameter, it returns the best fitting width.
    ///
    /// # Parameters
    /// - `width`: The reference width.
    ///
    /// # Returns
    /// The best fitting width.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::{MeasureMode, WrapMode};
    ///
    /// let mode = MeasureMode::min();
    /// assert_eq!(mode.coerce_width(10), 10);
    ///
    /// let mode = MeasureMode::pref_width(15, WrapMode::default());
    /// assert_eq!(mode.coerce_width(10), 10);
    /// assert_eq!(mode.coerce_width(20), 15);
    ///
    /// let mode = MeasureMode::fixed_width(15, WrapMode::default());
    /// assert_eq!(mode.coerce_width(10), 15);
    /// assert_eq!(mode.coerce_width(20), 15);
    /// ```
    #[must_use]
    pub fn coerce_width(&self, width: usize) -> usize {
        match self {
            MeasureMode::Min => width,
            MeasureMode::PrefWidth { max_width, .. } => min(*max_width, width),
            MeasureMode::FixedWidth { width, .. } => *width,
            MeasureMode::Exact { dimension, .. } => dimension.width,
        }
    }

    /// Returns, if defined, the height.
    /// There is a height for variant `Exact` only.
    ///
    /// # Returns
    /// The height or `None`.
    #[must_use]
    pub fn height(&self) -> Option<usize> {
        match self {
            Self::Exact { dimension, .. } => Some(dimension.height),
            _ => None,
        }
    }

    /// Coerces a concrete height.
    /// Depending on the variant and the `height` parameter, it returns the best fitting height,
    /// which is typically the `height` parameter itself, except in the `Exact` variant, where
    /// the height is taken from the exact dimension.
    ///
    /// # Parameters
    /// - `height`: The reference height.
    ///
    /// # Returns
    /// The best fitting height.
    #[must_use]
    pub fn coerce_height(&self, height: usize) -> usize {
        self.height().unwrap_or(height)
    }

    /// Coerces a concrete dimension.
    /// This combines [`coerce_width`](MeasureMode::coerce_width) and
    /// [`coerce_height`](MeasureMode::coerce_height) to coerce a [`Dimension`]
    ///
    /// # Parameters
    /// - `dim`:  The reference `Dimension`.
    ///
    /// # Returns
    /// The best fitting `Dimension`
    #[must_use]
    pub fn coerce_dim(&self, dim: Dimension) -> Dimension {
        Dimension {
            width: self.coerce_width(dim.width),
            height: self.coerce_height(dim.height),
        }
    }

    /// Checks whether the `MeasureMode` will result in an empty dimension.
    ///
    /// # Returns
    /// `true` if the `MeasureMode` will result in an empty dimension, `false` otherwise.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        match self {
            MeasureMode::Min => false,
            MeasureMode::PrefWidth { max_width, .. } => *max_width == 0,
            MeasureMode::FixedWidth { width, .. } => *width == 0,
            MeasureMode::Exact { dimension, .. } => dimension.is_empty(),
        }
    }
}

/// Represents the measurements of a component.
///
/// Technically, this is the result of [`measure`](crate::Layout::measure). It contains the overall
/// size of the component (field `dim`) and - depending on the component - further component-specific
/// data (field `specifics`).
///
/// `Measurements` play a central role in the layout process, since they provide the information
///  regarding the size of the component and possibly nested structures.
///
/// You may create an empty instance of `Measurements` using the [`empty`](Measurements::empty)
/// method, a general new with [`new`](Measurements::new), or utilize the conversion from
/// [`Dimension`]
#[derive(Clone)]
pub struct Measurements {
    /// The overall size of the component.
    pub dim: Dimension,
    /// Depending on the component, further component-specific data.
    pub specifics: MeasurementSpecifics,
}

impl Measurements {
    /// Creates a new instance of `Measurements`.
    ///
    /// # Parameters
    /// - `dim`: The overall [`Dimension`]
    /// - `specifics`: Further component-specific data, encapsulated in [`MeasurementSpecifics`]
    ///
    /// # Returns
    /// A new instance of `Measurements`.
    #[must_use]
    pub fn new(dim: Dimension, specifics: MeasurementSpecifics) -> Self {
        Self { dim, specifics }
    }

    /// Creates an empty instance of `Measurements`.
    ///
    /// # Returns
    /// An empty instance of `Measurements`.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::Measurements;
    ///
    /// let measurements = Measurements::empty();
    /// assert_eq!(measurements.dim.is_empty(), true);
    /// assert_eq!(measurements.specifics.is_none(),true);
    /// ```
    #[must_use]
    pub fn empty() -> Self {
        Self {
            dim: Dimension::empty(),
            specifics: MeasurementSpecifics::None,
        }
    }

    /// Creates a new instance based on the given one having the given `specifics`.
    ///
    /// # Parameters
    /// - `specifics`: The new [`MeasurementSpecifics`] to be used.
    ///
    /// # Returns
    /// A new instance of `Measurements` with the given `specifics`.
    #[must_use]
    pub fn with_specifics(self, specifics: MeasurementSpecifics) -> Self {
        Self {
            dim: self.dim,
            specifics,
        }
    }

    /// Split this instance into a left and right side, split at the given `width`.
    /// The left and right side have equal specifics.
    ///
    /// # Parameters
    /// - `width`: The width at which to split this instance.
    ///
    /// # Returns
    /// A pair of left and right parts, split at the given `width`.
    ///
    /// # Example
    /// ```rust
    ///
    /// use termlayout::{Dimension, MeasurementSpecifics, Measurements};
    /// let m = Measurements::new(Dimension::new(10, 10), MeasurementSpecifics::None);
    /// let (l,r) = m.split_horizontal(4);
    ///
    /// assert_eq!(l.dim, Dimension::new(4, 10));
    /// assert_eq!(r.dim, Dimension::new(6, 10));
    /// ```
    #[must_use]
    pub fn split_horizontal(self, width: usize) -> (Self, Self) {
        let (left, right) = self.dim.split_horizontal(width);
        (
            Self::new(left, self.specifics.clone()),
            Self::new(right, self.specifics),
        )
    }

    /// Split this instance into a top and bottom side, split at the given `height`.
    /// The top and bottom side have equal specifics.
    ///
    /// # Parameters
    /// - `height`: The height at which to split this instance.
    ///
    /// # Returns
    /// A pair of top and bottom part, split at the given `height`.
    ///
    /// # Example
    /// ```rust
    ///
    /// use termlayout::{Dimension, MeasurementSpecifics, Measurements};
    /// let m = Measurements::new(Dimension::new(10, 10), MeasurementSpecifics::None);
    /// let (t, b) = m.split_vertical(4);
    ///
    /// assert_eq!(t.dim, Dimension::new(10, 4));
    /// assert_eq!(b.dim, Dimension::new(10, 6));
    /// ```
    #[must_use]
    pub fn split_vertical(self, height: usize) -> (Self, Self) {
        let (top, bottom) = self.dim.split_vertical(height);
        (
            Self::new(top, self.specifics.clone()),
            Self::new(bottom, self.specifics),
        )
    }

    /// Checks, whether this instance is empty.
    /// This instance is empty, if its size is empty
    ///
    /// # Returns
    /// `true`, if this instance is empty, `false` otherwise
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.dim.is_empty()
    }

    /// Folds the given iterator of [`RcLayout`]s vertically by measuring each
    /// iterator element and forming a final [`Measurements`].
    ///
    /// # Parameters
    /// - `iterator`: The iterator of [`RcLayout`]s to fold vertically
    /// - `mode`: The [`MeasureMode`] to use for measuring the children
    ///
    /// # Returns
    /// The final [`Measurements`] instance
    ///
    /// # Example
    /// ```rust
    /// use termlayout::{Dimension, MeasureMode, Measurements};
    /// use termlayout::widgets::Lines;
    ///
    /// let item1 = Lines::left("abc");
    /// let item2 = Lines::right("defgh");
    ///
    /// let m = Measurements::fold_vertically([item1.into(), item2.into()].iter(), MeasureMode::Min);
    ///
    /// assert_eq!(m.dim, Dimension::new(5, 2));
    /// assert_eq!(m.specifics.children().unwrap().len(), 2);
    /// ```
    #[must_use]
    pub fn fold_vertically<'a>(
        iterator: impl Iterator<Item = &'a RcLayout>,
        mode: MeasureMode,
    ) -> Self {
        // First figure out all default measurements, they might differ regarding width
        let children: Vec<(RcLayout, Measurements)> = iterator
            .map(|layout| (layout.clone(), layout.measure(mode)))
            .collect();

        // Compute the overall dimension
        let dim = children.iter().fold(Dimension::empty(), |acc, child| {
            acc.vertical_union(child.1.dim)
        });

        // And finally adjust the width of all items to the same
        let children = children
            .into_iter()
            .map(|(layout, measurement)| {
                if measurement.dim.width == dim.width {
                    measurement
                } else {
                    layout.measure(MeasureMode::exact(
                        Dimension::new(dim.width, measurement.dim.height),
                        mode.wrap_mode(),
                    ))
                }
            })
            .collect();
        Self::new(dim, MeasurementSpecifics::Children(children))
    }
}

impl From<Dimension> for Measurements {
    fn from(value: Dimension) -> Self {
        Self::new(value, MeasurementSpecifics::None)
    }
}

/// Provides component-specific data attached to a [`Measurements`] instance.
/// There is no one-to-one relation between component/widget type and variant
/// of this enum. Instead, several widget types can share the same variant.
#[derive(Clone)]
pub enum MeasurementSpecifics {
    /// Represents the case that there is no further specific information
    /// available for this measurement. Typically, widgets like `Lines`,
    /// `Filler`, or `Paragraph` have no further information.
    None,
    /// Represents the case with exactly one child measurement. This
    /// variant is used by widgets like `Cell` or `Frame`.
    Child(Box<Measurements>),
    /// Represents the case with any number of child measurements,
    /// such as the `Vertical` widget.
    Children(Vec<Measurements>),
    /// Represents the case with `Rows`, this is used by widgets like
    /// `Horizontal` or `Table`.  
    Rows(Vec<Row>),
    /// Represents the case with custom measurement specifics.
    Custom(Rc<dyn Any>),
}

impl MeasurementSpecifics {
    /// Returns `true`, if this instance is `None`.
    ///
    /// # Returns
    /// `true`, if this instance is `None`.
    #[must_use]
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    /// Returns - if present - a list of child [`Measurements`].
    ///
    /// # Returns
    /// A slice of [`Measurements`] or `None`, if the variant is not `Children`.
    #[must_use]
    pub fn children(&self) -> Option<&[Measurements]> {
        match self {
            MeasurementSpecifics::Children(children) => Some(children),
            _ => None,
        }
    }

    /// Returns - if present - a single [`Measurements`].
    ///
    /// # Returns
    /// A s[`Measurements`] or `None`, if the variant is not `Child`.
    #[must_use]
    pub fn child(&self) -> Option<&Measurements> {
        match self {
            MeasurementSpecifics::Child(child) => Some(child.as_ref()),
            _ => None,
        }
    }

    /// Returns - if present - a list of child [`Row`](crate::ext::Row)s.
    ///
    /// # Returns
    /// A slice of [`Row`](crate::ext::Row)s or `None`, if the variant is not `Rows`.
    #[must_use]
    pub fn rows(&self) -> Option<&[Row]> {
        match self {
            MeasurementSpecifics::Rows(rows) => Some(rows),
            _ => None,
        }
    }
}

impl TryInto<Vec<Row>> for MeasurementSpecifics {
    type Error = &'static str;

    fn try_into(self) -> Result<Vec<Row>, Self::Error> {
        match self {
            MeasurementSpecifics::Rows(rows) => Ok(rows),
            _ => Err("MeasurementSpecifics is not Rows"),
        }
    }
}

impl TryInto<Measurements> for MeasurementSpecifics {
    type Error = &'static str;

    fn try_into(self) -> Result<Measurements, Self::Error> {
        match self {
            MeasurementSpecifics::Child(child) => Ok(*child),
            _ => Err("MeasurementSpecifics is not Child"),
        }
    }
}

impl TryInto<Vec<Measurements>> for MeasurementSpecifics {
    type Error = &'static str;

    fn try_into(self) -> Result<Vec<Measurements>, Self::Error> {
        match self {
            MeasurementSpecifics::Children(children) => Ok(children),
            _ => Err("MeasurementSpecifics is not Children"),
        }
    }
}
