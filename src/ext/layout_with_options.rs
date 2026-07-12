use crate::ext::{BoxedLayoutWriter, FormattedLayout};
use crate::{BoxedFormattedLayout, LayoutOptions, RcLayout};
use ouroboros::self_referencing;

/// A wrapper that pairs a [`RcLayout`] with specific [`LayoutOptions`].
/// This struct is useful when you want to use the `Layout`/`LayoutWithOptions` pair directly
/// as a [`FormattedLayout`], which is the main aim of this type: it is a container
/// for the result of the [`Layout_strict`](Layout::layout_strict) method.
#[self_referencing]
pub struct LayoutWithOptions {
    layout: RcLayout,
    options: LayoutOptions,
    #[borrows(layout, options)]
    #[covariant]
    formatted: BoxedFormattedLayout<'this>,
}

impl LayoutWithOptions {
    /// Creates a new `LayoutWithOptions` instance with the given layout and options.
    ///
    /// # Parameters
    /// - `layout`: The [`crate::Layout`]
    /// - `options`: The [`LayoutOptions`]
    ///
    /// # Returns
    /// A new `LayoutWithOptions` instance
    pub fn of(layout: RcLayout, options: LayoutOptions) -> Self {
        LayoutWithOptionsBuilder {
            layout,
            options,
            formatted_builder: |wrt, opt| wrt.layout_strict(*opt),
        }
        .build()
    }
}

impl FormattedLayout for LayoutWithOptions {
    fn options(&self) -> &LayoutOptions {
        self.borrow_options()
    }

    fn new_writer(&'_ self) -> BoxedLayoutWriter<'_> {
        self.borrow_formatted().new_writer()
    }
}

impl From<LayoutWithOptions> for BoxedFormattedLayout<'static> {
    fn from(value: LayoutWithOptions) -> Self {
        Box::new(value)
    }
}
