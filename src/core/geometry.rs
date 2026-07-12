use std::cmp::min;
use std::ops::Range;

/// Represents a dimension measured in width and height.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Dimension {
    /// The width of the dimension
    pub width: usize,
    /// The height of the dimension
    pub height: usize,
}

impl Dimension {
    /// The maximum dimension
    pub const MAX: Self = Self {
        width: usize::MAX,
        height: usize::MAX,
    };

    /// Creates a new dimension with the given width and height.
    ///
    /// # Parameters
    /// - `width`: The width
    /// - `height`: The height
    ///
    /// # Returns
    /// A new `Dimension` instance with the specified width and height.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::Dimension;
    ///
    /// let dim = Dimension::new(3,2);
    ///
    /// assert_eq!(dim.width, 3);
    /// assert_eq!(dim.height, 2);
    /// ```
    #[must_use]
    pub const fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }

    /// Returns a new, empty dimension.
    ///
    /// # Returns
    /// An empty `Dimension` instance.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::Dimension;
    ///
    /// let dim = Dimension::empty();
    ///
    /// assert_eq!(dim.width, 0);
    /// assert_eq!(dim.height, 0);
    /// ```
    #[must_use]
    pub const fn empty() -> Self {
        Self::new(0, 0)
    }

    /// Checks if the dimension is empty (i.e., either width or height is zero).
    ///
    /// # Returns
    /// `true`, if empty, `false` otherwise.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::Dimension;
    ///
    /// let dim = Dimension::new(3 ,2);
    /// assert_eq!(dim.is_empty(), false);
    ///
    /// let dim = Dimension::new(3, 0);
    /// assert_eq!(dim.is_empty(), true);
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    /// Combines two instances vertically by stacking their height and choosing the maximum width.
    ///
    /// # Parameters
    /// - `other`: Another instance.
    ///
    /// # Returns
    /// The vertical union of `self` and `other`. The resulting instance will have:
    /// - width equal to the maximum of `self.width` and `other.width`.
    /// - height equal to the sum of `self.height` and `other.height`.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::Dimension;
    ///
    /// let dim1 = Dimension::new(2, 3);
    /// let dim2 = Dimension::new(5, 7);
    ///
    /// assert_eq!(dim1.vertical_union(dim2), Dimension::new(5, 10));
    /// ```
    #[must_use]
    pub fn vertical_union(&self, other: Self) -> Self {
        Self {
            width: std::cmp::max(self.width, other.width),
            height: self.height + other.height,
        }
    }

    /// Combines two instances horizontally by stacking their width and choosing the maximum height.
    ///
    /// # Parameters
    /// - `other`: Another instance.
    ///
    /// # Returns
    /// The horizontal union of `self` and `other`. The resulting instance will have:
    /// - width equal to the sum of `self.width` and `other.width`.
    /// - height equal to the maximum of `self.height` and `other.height`.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::Dimension;
    ///
    /// let dim1 = Dimension::new(11, 13);
    /// let dim2 = Dimension::new(17, 19);
    ///
    /// assert_eq!(dim1.horizontal_union(dim2), Dimension::new(28, 19));
    /// ```
    #[must_use]
    pub fn horizontal_union(&self, other: Self) -> Self {
        Self {
            width: self.width + other.width,
            height: std::cmp::max(self.height, other.height),
        }
    }
}

/// Defines a rectangle with its position and [`Dimension`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Rect {
    /// The horizontal position.
    pub x: usize,
    /// The vertical position.
    pub y: usize,
    /// A `Dimension` instance representing the width and height.
    pub dim: Dimension,
}

impl Rect {
    /// Creates a new instance of the struct.
    ///
    /// # Parameters
    /// - `x`: The horizontal position.
    /// - `y`: The vertical position.
    /// - `dim`: A `Dimension` instance representing the width and height.
    ///
    /// # Returns
    /// A new instance of the struct initialized with the provided `x`, `y`, and `dim` values.
    ///
    /// # Examples
    /// ```rust
    /// use termlayout::Dimension;
    /// use termlayout::Rect;
    ///
    /// let instance = Rect::new(1, 2, Dimension::new(3, 4));
    /// assert_eq!(instance.x, 1);
    /// assert_eq!(instance.y, 2);
    /// assert_eq!(instance.dim.width, 3);
    /// assert_eq!(instance.dim.height, 4);
    /// ```
    #[must_use]
    pub const fn new(x: usize, y: usize, dim: Dimension) -> Self {
        Self { x, y, dim }
    }

    /// Creates a new instance of the struct using ranges for the `x` and `y` values.
    ///
    /// # Parameters
    /// - `x`: The range of X values
    /// - `y`: The range of Y values
    ///
    /// # Returns
    /// A new instance of the struct initialized with the provided `x` and `y` values.
    ///
    /// # Examples
    /// ```rust
    /// use termlayout::Rect;
    ///
    /// let instance = Rect::from_ranges(1..4, 2..6);
    /// assert_eq!(instance.x, 1);
    /// assert_eq!(instance.y, 2);
    /// assert_eq!(instance.dim.width, 3);
    /// assert_eq!(instance.dim.height, 4);
    /// ```
    #[must_use]
    pub fn from_ranges(x: Range<usize>, y: Range<usize>) -> Self {
        Self {
            x: x.start,
            y: y.start,
            dim: Dimension::new(x.len(), y.len()),
        }
    }

    /// Creates and returns a new empty instance.
    ///
    /// # Returns
    /// An empty instance.
    ///
    /// # Example
    /// ```
    /// use termlayout::Dimension;
    /// use termlayout::Rect;
    ///
    /// let instance = Rect::empty();
    ///
    /// assert_eq!(instance.x, 0);
    /// assert_eq!(instance.y, 0);
    /// assert!(instance.is_empty());
    /// ```
    #[must_use]
    pub const fn empty() -> Self {
        Self::new(0, 0, Dimension::empty())
    }

    /// Calculates a new [`Rect`] having the same dimension as this one, but the given
    /// position.
    ///
    /// # Parameters
    /// - `x`: horizontal position of the new rect
    /// - `y`: vertical position of the new rect
    ///
    /// # Returns
    /// A rect with the new position.
    ///
    /// # Examples
    /// ```rust
    /// use termlayout::Dimension;
    /// use termlayout::Rect;
    ///
    /// let instance: Rect = Dimension::new(3, 4).into();
    /// let new_instance = instance.with_pos(1, 2);
    ///
    /// assert_eq!(new_instance.x, 1);
    /// assert_eq!(new_instance.y, 2);
    /// assert_eq!(new_instance.dim.width, 3);
    /// assert_eq!(new_instance.dim.height, 4);
    /// ```
    #[must_use]
    pub fn with_pos(self, x: usize, y: usize) -> Self {
        Self::new(x, y, self.dim)
    }

    /// Calculates a new [`Rect`] having `x` and `y` added to the position of this rect.
    ///
    /// # Parameters
    /// - `x`: horizontal offset of the new rect
    /// - `y`: vertical offset of the new rect
    ///
    /// # Returns
    /// A rect with the new position.
    ///
    /// # Examples
    /// ```rust
    /// use termlayout::Dimension;
    /// use termlayout::Rect;
    ///
    /// let instance: Rect = Rect::new(1, 2, Dimension::new(3, 4));
    /// let new_instance = instance.with_offset(10, 20);
    ///
    /// assert_eq!(new_instance.x, 11);
    /// assert_eq!(new_instance.y, 22);
    /// assert_eq!(new_instance.dim.width, 3);
    /// assert_eq!(new_instance.dim.height, 4);
    /// ```
    #[must_use]
    pub fn with_offset(self, x: usize, y: usize) -> Self {
        Self::new(self.x + x, self.y + y, self.dim)
    }

    /// Calculates a new [`Rect`] having the same position as this one, but the given
    /// dimension.
    ///
    /// # Parameters
    /// - `dim`: The new [`Dimension`].
    ///
    /// # Returns
    /// A rect with the new dimension.
    ///
    /// # Examples
    /// ```rust
    /// use termlayout::Dimension;
    /// use termlayout::Rect;
    ///
    /// let instance: Rect = Dimension::new(3, 4).into();
    /// let new_instance = instance.with_dim(Dimension::new(5, 6));
    ///
    /// assert_eq!(new_instance.x, 0);
    /// assert_eq!(new_instance.y, 0);
    /// assert_eq!(new_instance.dim.width, 5);
    /// assert_eq!(new_instance.dim.height, 6);
    /// ```
    #[must_use]
    pub fn with_dim(self, dim: Dimension) -> Self {
        Self::new(self.x, self.y, dim)
    }

    /// Returns the range of valid x values.
    ///
    /// # Returns
    /// A range with the possible x values.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::Dimension;
    /// use termlayout::Rect;
    ///
    /// let instance: Rect = Rect::new(2, 3, Dimension::new(5, 7));
    ///
    /// assert_eq!(instance.x_range(), 2..7);
    /// ```
    #[must_use]
    pub fn x_range(&self) -> Range<usize> {
        self.x..(self.x + self.dim.width)
    }

    /// Returns the range of valid y values.
    ///
    /// # Returns
    /// A range with the possible y values.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::Dimension;
    /// use termlayout::Rect;
    ///
    /// let instance: Rect = Rect::new(2, 3, Dimension::new(5, 7));
    ///
    /// assert_eq!(instance.y_range(), 3..10);
    /// ```
    #[must_use]
    pub fn y_range(&self) -> Range<usize> {
        self.y..(self.y + self.dim.height)
    }

    /// Calculates the intersection with the `other` rectangle and returns a new rectangle
    /// representing the overlapping area.
    ///
    /// # Parameters
    /// - `other`: The other rectangle to calculate the intersection with.
    ///
    /// # Returns
    /// A new rectangle representing the overlapping (intersection) area. If there is no
    /// overlap, the returned rectangle will have a width and/or height of 0.
    ///
    /// # Example
    /// ```
    /// use termlayout::Dimension;
    /// use termlayout::Rect;
    ///
    /// let rect1 = Rect::new(1, 2, Dimension::new(3, 4));
    /// let rect2 = Rect::new(1, 2, Dimension::new(3, 4));
    /// let intersection = rect1.intersect(rect2);
    /// assert_eq!(intersection.x, 1);
    /// assert_eq!(intersection.y, 2);
    /// assert_eq!(intersection.dim, Dimension::new(3, 4));
    ///
    /// let rect1 = Rect::new(1, 2, Dimension::new(3, 4));
    /// let rect2 = Rect::new(2, 1, Dimension::new(4, 3));
    /// let intersection = rect1.intersect(rect2);
    /// assert_eq!(intersection.x, 2);
    /// assert_eq!(intersection.y, 2);
    /// assert_eq!(intersection.dim, Dimension::new(2, 2));
    ///
    /// let rect1 = Rect::new(1, 2, Dimension::new(3, 4));
    /// let rect2 = Rect::new(20, 10, Dimension::new(4, 3));
    /// let intersection = rect1.intersect(rect2);
    /// assert_eq!(intersection.x, 20);
    /// assert_eq!(intersection.y, 10);
    /// assert_eq!(intersection.dim, Dimension::empty());
    /// ```
    #[must_use]
    pub fn intersect(&self, other: Self) -> Self {
        let col = std::cmp::max(self.x, other.x);
        let row = std::cmp::max(self.y, other.y);
        let width = min(self.x + self.dim.width, other.x + other.dim.width).saturating_sub(col);
        let height = min(self.y + self.dim.height, other.y + other.dim.height).saturating_sub(row);
        Self::new(col, row, Dimension::new(width, height))
    }

    /// Calculates the relative intersection of the current rectangle with another rectangle.
    ///
    /// This method computes the intersection between `self` and `other` rectangles
    /// and adjusts the resulting intersected rectangle's position relative to the
    /// origin of the current rectangle (`self`).
    ///
    /// # Parameters
    /// - `other`: The rectangle to intersect with the current rectangle (`self`).
    ///
    /// # Returns
    /// A new rectangle representing the intersection of `self` and `other`,
    /// with its `x` and `y` attributes adjusted to be relative to the
    /// current rectangle's origin.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::Dimension;
    /// use termlayout::Rect;
    ///
    /// let rect1 = Rect::new(1, 1, Dimension::new(5, 5));
    /// let rect2 = Rect::new(3, 3, Dimension::new(4, 3));
    /// let intersection = rect1.intersect_relative(rect2);
    /// assert_eq!(intersection.x, 2);
    /// assert_eq!(intersection.y, 2);
    /// assert_eq!(intersection.dim, Dimension::new(3, 3));
    /// ```
    #[must_use]
    pub fn intersect_relative(&self, other: Self) -> Self {
        let x = self.x;
        let y = self.y;
        let mut rect = self.intersect(other);
        rect.x -= x;
        rect.y -= y;
        rect
    }

    /// Checks whether the current instance is empty.
    /// The rect is empty if its dimension is empty.
    ///
    /// # Returns
    ///
    /// - `true` if it is empty.
    /// - `false` otherwise.
    ///
    /// # Example
    /// ```
    /// use termlayout::Dimension;
    /// use termlayout::Rect;
    ///
    /// let instance = Rect::new(1, 2, Dimension::new(3, 4));
    /// assert_eq!(instance.is_empty(), false);
    /// let instance = Rect::new(1, 2, Dimension::new(0, 0));
    /// assert_eq!(instance.is_empty(), true);
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.dim.is_empty()
    }

    /// Splits this instance horizontally at the given `width`.
    ///
    /// # Parameters
    /// - `width`: The width where to split
    ///
    /// # Example
    /// ```rust
    /// use termlayout::{Dimension, Rect};
    /// let rect = Rect::new(10, 20, Dimension::new(30, 40));
    ///
    /// let (left, right) = rect.split_horizontal(17);
    /// assert_eq!(left, Rect::new(10, 20, Dimension::new(17, 40)));
    /// assert_eq!(right, Rect::new(27, 20, Dimension::new(13, 40)));
    /// ```
    #[must_use]
    pub fn split_horizontal(&self, width: usize) -> (Rect, Rect) {
        let left = min(self.dim.width, width);
        let rect1 = Rect::new(self.x, self.y, Dimension::new(left, self.dim.height));
        let rect2 = Rect::new(
            self.x + left,
            self.y,
            Dimension::new(self.dim.width - left, self.dim.height),
        );
        (rect1, rect2)
    }

    /// Splits this instance vertically at the given `height`.
    ///
    /// # Parameters
    /// - `height`: The width where to split
    ///
    /// # Example
    /// ```rust
    /// use termlayout::{Dimension, Rect};
    /// let rect = Rect::new(10, 20, Dimension::new(30, 40));
    ///
    /// let (top, bottom) = rect.split_vertical(23);
    /// assert_eq!(top, Rect::new(10, 20, Dimension::new(30, 23)));
    /// assert_eq!(bottom, Rect::new(10, 43, Dimension::new(30, 17)));
    /// ```
    #[must_use]
    pub fn split_vertical(&self, height: usize) -> (Rect, Rect) {
        let top = min(self.dim.height, height);
        let rect1 = Rect::new(self.x, self.y, Dimension::new(self.dim.width, top));
        let rect2 = Rect::new(
            self.x,
            self.y + top,
            Dimension::new(self.dim.width, self.dim.height - top),
        );
        (rect1, rect2)
    }
}

impl From<Dimension> for Rect {
    fn from(value: Dimension) -> Self {
        Rect::new(0, 0, value)
    }
}

/// Provides extension methods for the [`Range`] type.
pub trait RangeExt<T: PartialOrd> {
    /// Normalizes the range by ensuring that the resulting range has the same length as the
    /// original one and the start is 0.
    ///
    /// # Returns
    /// The normalized range.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::ext::RangeExt;
    ///
    /// let range = 11..17usize;
    /// assert_eq!(range.normalize(), 0..6usize)
    /// ```
    #[must_use]
    fn normalize(&self) -> Self;

    /// Returns a range where `offset` is added to the start and end.
    ///
    /// # Parameters
    /// - `offset`: The offset to add to the range.
    ///
    /// # Returns
    /// The range with offset added to start and end.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::ext::RangeExt;
    ///
    /// let range = 11..17usize;
    /// assert_eq!(range.add_offset(2), 13..19usize)
    /// ```
    #[must_use]
    fn add_offset(&self, offset: T) -> Self;

    /// Returns a range where `offset` is subtracted from the start and end.
    ///
    /// # Parameters
    /// - `offset`: The offset to subtract from the range.
    ///
    /// # Returns
    /// The range with offset subtracted from start and end.
    ///
    /// # Example
    /// ```rust
    /// use termlayout::ext::RangeExt;
    ///
    /// let range = 11..17usize;
    /// assert_eq!(range.sub_offset(2), 9..15usize)
    /// ```
    #[must_use]
    fn sub_offset(&self, offset: T) -> Self;

    /// Builds the intersection of this instance and `other`.
    ///
    /// # Parameters
    /// - `other`: The other instance
    ///
    /// # Returns
    /// The intersection
    ///
    /// # Example
    /// ```rust
    /// use termlayout::ext::RangeExt;
    ///
    /// let range1 = 11..17usize;
    /// let range2 = 13..19usize;
    /// assert_eq!(range1.intersect(range2), 13..17usize)
    /// ```
    #[must_use]
    fn intersect(&self, other: Self) -> Self;
}

impl RangeExt<usize> for Range<usize> {
    fn normalize(&self) -> Self {
        0..self.end.saturating_sub(self.start)
    }

    fn add_offset(&self, offset: usize) -> Self {
        self.start + offset..self.end + offset
    }

    fn sub_offset(&self, offset: usize) -> Self {
        self.start.saturating_sub(offset)..self.end.saturating_sub(offset)
    }

    fn intersect(&self, other: Self) -> Self {
        self.start.max(other.start)..self.end.min(other.end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dimension_new() {
        let dim = Dimension::new(10, 20);
        assert_eq!(dim.width, 10);
        assert_eq!(dim.height, 20);
    }

    #[test]
    fn dimension_empty() {
        let dim = Dimension::empty();
        assert_eq!(dim.width, 0);
        assert_eq!(dim.height, 0);
        assert!(dim.is_empty());
    }

    #[test]
    fn dimension_is_empty() {
        assert!(Dimension::new(0, 0).is_empty());
        assert!(Dimension::new(10, 0).is_empty());
        assert!(Dimension::new(0, 20).is_empty());
        assert!(!Dimension::new(10, 20).is_empty());
    }

    #[test]
    fn dimension_vertical_union() {
        let dim1 = Dimension::new(10, 5);
        let dim2 = Dimension::new(15, 10);
        let union = dim1.vertical_union(dim2);
        assert_eq!(union.width, 15);
        assert_eq!(union.height, 15);
    }

    #[test]
    fn dimension_horizontal_union() {
        let dim1 = Dimension::new(10, 5);
        let dim2 = Dimension::new(15, 10);
        let union = dim1.horizontal_union(dim2);
        assert_eq!(union.width, 25);
        assert_eq!(union.height, 10);
    }

    #[test]
    fn rect_new() {
        let dim = Dimension::new(10, 20);
        let rect = Rect::new(5, 7, dim);
        assert_eq!(rect.x, 5);
        assert_eq!(rect.y, 7);
        assert_eq!(rect.dim, dim);
    }

    #[test]
    fn rect_empty() {
        let rect = Rect::empty();
        assert_eq!(rect.x, 0);
        assert_eq!(rect.y, 0);
        assert!(rect.is_empty());
    }

    #[test]
    fn rect_with_pos() {
        let rect = Rect::new(1, 1, Dimension::new(10, 10)).with_pos(5, 5);
        assert_eq!(rect.x, 5);
        assert_eq!(rect.y, 5);
        assert_eq!(rect.dim, Dimension::new(10, 10));
    }

    #[test]
    fn rect_with_dim() {
        let rect = Rect::new(1, 1, Dimension::new(10, 10)).with_dim(Dimension::new(5, 5));
        assert_eq!(rect.x, 1);
        assert_eq!(rect.y, 1);
        assert_eq!(rect.dim, Dimension::new(5, 5));
    }

    #[test]
    fn rect_ranges() {
        let rect = Rect::new(2, 3, Dimension::new(5, 7));
        assert_eq!(rect.x_range(), 2..7);
        assert_eq!(rect.y_range(), 3..10);
    }

    #[test]
    fn rect_intersect() {
        let r1 = Rect::new(0, 0, Dimension::new(10, 10));
        let r2 = Rect::new(5, 5, Dimension::new(10, 10));
        let intersection = r1.intersect(r2);
        assert_eq!(intersection, Rect::new(5, 5, Dimension::new(5, 5)));

        let r3 = Rect::new(20, 20, Dimension::new(5, 5));
        let no_intersection = r1.intersect(r3);
        assert!(no_intersection.is_empty());
    }

    #[test]
    fn rect_intersect_relative() {
        let r1 = Rect::new(10, 10, Dimension::new(20, 20));
        let r2 = Rect::new(15, 15, Dimension::new(10, 10));
        let intersection = r1.intersect_relative(r2);
        assert_eq!(intersection, Rect::new(5, 5, Dimension::new(10, 10)));
    }

    #[test]
    fn rect_from_dimension() {
        let dim = Dimension::new(10, 20);
        let rect: Rect = dim.into();
        assert_eq!(rect.x, 0);
        assert_eq!(rect.y, 0);
        assert_eq!(rect.dim, dim);
    }
}
