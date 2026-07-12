//! Beispiel für ein benutzerdefiniertes Zwei-Spalten-Layout-Widget

use std::any::Any;
use termlayout::ext::{DisplayStr, LayoutWithOptions};
use termlayout::widgets::{Cell, Filler, Horizontal, Lines};
use termlayout::{BoxedFormattedLayout, Dimension, Layout, LayoutOptions, RcLayout, WrapMode};

#[path = "shared/mod.rs"]
mod shared;

/// Demonstrates how to use `termlayout` to implement a new reusable widget that can display its
/// content in two columns.
///
/// This example makes use of the [`Horizontal`] and [`Lines`] widget to form a new reusable
/// widget named `TwoColumns`. `TwoColumns` is a widget that displays its content in two columns.
struct TwoColumns {
    content: RcLayout,
    spacer: String,
}

impl From<RcLayout> for TwoColumns {
    fn from(content: RcLayout) -> Self {
        Self {
            content,
            spacer: " | ".to_string(),
        }
    }
}

impl TwoColumns {
    /// Helper to check whether the maximum width allows for displaying content in two columns.
    /// This is the case if the `max_width` is big enough to display at least one character per
    /// column.
    ///
    /// # Parameters
    /// - `max_width`: The maximum available width for displaying the content.
    ///
    /// # Returns
    /// `true` if `max_width` provides sufficient space for the space and at least one character
    /// for each column
    fn can_display_with_two_columns(&self, max_width: usize) -> bool {
        max_width >= 2 + self.spacer.display_len()
    }
}
impl Layout for TwoColumns {
    /// Calculates the preferred dimension.
    /// The most important constraint is that the resulting dimension must never exceed the
    /// `max_width` regarding the width of the returned dimension.
    fn pref_dim(&self, max_width: usize, wrap_mode: WrapMode) -> Dimension {
        // First check, whether we have sufficient space for two columns
        if self.can_display_with_two_columns(max_width) {
            // We have enough space for two columns. We determine the width of the columns
            let col_width = (max_width - self.spacer.display_len()) / 2;
            // We calculate the preferred dimension of the content based on the column width
            let dim = self.content.pref_dim(col_width, wrap_mode);
            // The total width is then the sum of the two columns and the spacer width.
            // The height is half of the height of the content.
            Dimension::new(
                2 * col_width + self.spacer.display_len(),
                dim.height.div_ceil(2),
            )
        } else {
            // We reach this branch if we display only one column. In this case we just return
            // the preferred dimension of the content.
            self.content.pref_dim(max_width, wrap_mode)
        }
    }

    /// Calculates the minimum dimension.
    /// The minimum dimension should return the dimension with the minimum width so that we can
    /// display the content without wrapping, truncation, and loss of information.
    fn min_dim(&self) -> Dimension {
        // Get the minimum dimension of the content
        let dim = self.content.min_dim();
        // so the minimum dimension should be the double width of the content plus the spacer and
        // half the height of the content.
        Dimension::new(
            2 * dim.width + self.spacer.display_len(),
            dim.height.div_ceil(2),
        )
    }

    fn layout_strict(&'_ self, options: LayoutOptions) -> BoxedFormattedLayout<'_> {
        // if there is not enough space for two columns, we just display the content in one column
        if !self.can_display_with_two_columns(options.dim.width) {
            return self.content.layout_strict(options);
        }

        // Compute the dimension of the content. This is basically the half of the available width
        // minus the spacer width and the double of the height. Using the Cell::of method we can
        // create a cell for the entire content, with `split_vertical` it is split in the half to
        // form the two columns.
        let content_dim = Dimension::new(
            (options.dim.width - self.spacer.display_len()) / 2,
            2 * options.dim.height,
        );

        let (left, right) = Cell::of(self.content.clone())
            .with_dim(content_dim)
            .split_vertical(options.dim.height);

        let horizontal = Horizontal::new(
            vec![left, right],
            Some(Filler::vertical(&self.spacer).into()),
        )
        .into();
        LayoutWithOptions::of(horizontal, options).into()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn main() {
    let content: RcLayout = Lines::left(include_str!("../README.md")).into();
    let layout: TwoColumns = content.into();

    let cols = termsize::get().map_or(80, |ts| ts.cols);
    let formatted = layout.layout_with_wrap_mode(cols.into(), WrapMode::default());
    print!("{formatted}");
}
