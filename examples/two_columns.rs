//! Example demonstrating how to author an own [`Layout`] widget.

use std::any::Any;
use termlayout::ext::{DisplayStr, LayoutWithContext};
use termlayout::widgets::{Cell, Filler, Horizontal, Lines};
use termlayout::{BoxedFormattedLayout, Dimension, Layout, LayoutContext, MeasureMode, Measurements, RcLayout, WrapMode};

#[path = "shared/mod.rs"]
mod shared;

/// Demonstrates how to use `termlayout` to implement a new reusable widget that can display its
/// content in two columns.
///
/// This example makes use of the [`Horizontal`] and [`Lines`] widget to form a new reusable
/// widget named `TwoColumns`. `TwoColumns` is a widget that displays its content in two columns.
struct TwoColumns {
    /// The content
    content: RcLayout,
    /// The spacer string between the two columns
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
    /// Measures the dimensions of this layout based on the given mode.
    fn measure(&self, mode: MeasureMode) -> Measurements {
        match mode {
            MeasureMode::Min => {
                // Get the minimum dimension of the content.
                // The minimum is two columns plus spacer at half the height.
                let dim = self.content.measure(MeasureMode::Min).dim;
                Dimension::new(
                    2 * dim.width + self.spacer.display_len(),
                    dim.height.div_ceil(2),
                )
                .into()
            }
            MeasureMode::PrefWidth { max_width, wrap_mode } => {
                if self.can_display_with_two_columns(max_width) {
                    let col_width = (max_width - self.spacer.display_len()) / 2;
                    let dim = self
                        .content
                        .measure(MeasureMode::pref_width(col_width, wrap_mode))
                        .dim;
                    Dimension::new(
                        2 * col_width + self.spacer.display_len(),
                        dim.height.div_ceil(2),
                    )
                    .into()
                } else {
                    self.content.measure(mode)
                }
            }
            MeasureMode::FixedWidth { width, wrap_mode } => {
                if self.can_display_with_two_columns(width) {
                    let col_width = (width - self.spacer.display_len()) / 2;
                    let dim = self
                        .content
                        .measure(MeasureMode::fixed_width(col_width, wrap_mode))
                        .dim;
                    Dimension::new(width, dim.height.div_ceil(2)).into()
                } else {
                    self.content.measure(mode)
                }
            }
            MeasureMode::Exact { dimension, .. } => dimension.into(),
        }
    }

    fn layout_with_context(&'_ self, context: LayoutContext) -> BoxedFormattedLayout<'_> {
        let LayoutContext { options, measurements } = context;

        // if there is not enough space for two columns, we just display the content in one column
        if !self.can_display_with_two_columns(options.dim.width) {
            return self.content.layout_with_context(LayoutContext::new(options, measurements));
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

        let horizontal: RcLayout = Horizontal::new(
            vec![left, right],
            Some(Filler::vertical(&self.spacer).into()),
        )
        .into();
        let measurements = horizontal.measure(MeasureMode::exact(options.dim, options.wrap_mode));
        LayoutWithContext::of(horizontal, LayoutContext::new(options, measurements)).into()
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
