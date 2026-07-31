use crate::ext::{BoxedLayoutWriter, FormattedLayout};
use crate::{BoxedFormattedLayout, LayoutContext, LayoutOptions, RcLayout};
use ouroboros::self_referencing;

/// A wrapper that pairs a [`RcLayout`] with specific [`LayoutContext`].
/// This struct is useful when you want to use the `Layout`/`LayoutWithContext` pair directly
/// as a [`FormattedLayout`], which is the main aim of this type: it is a container
/// for the result of the [`layout_with_context`](Layout::layout_with_context) method.
#[self_referencing]
pub struct LayoutWithContext {
    layout: RcLayout,
    context: LayoutContext,
    #[borrows(layout, context)]
    #[covariant]
    formatted: BoxedFormattedLayout<'this>,
}

impl LayoutWithContext {
    /// Creates a new `LayoutWithContext` instance with the given layout and options.
    ///
    /// # Parameters
    /// - `layout`: The [`crate::Layout`]
    /// - `context`: The [`LayoutContext`]
    ///
    /// # Returns
    /// A new `LayoutWithContext` instance
    pub fn of(layout: RcLayout, context: LayoutContext) -> Self {
        LayoutWithContextBuilder {
            layout,
            context,
            formatted_builder: |lyt, ctxt| lyt.layout_with_context(ctxt.clone()),
        }
            .build()
    }
}

impl FormattedLayout for LayoutWithContext {
    fn options(&self) -> &LayoutOptions {
        &self.borrow_context().options
    }

    fn new_writer(&'_ self) -> BoxedLayoutWriter<'_> {
        self.borrow_formatted().new_writer()
    }
}

impl From<LayoutWithContext> for BoxedFormattedLayout<'static> {
    fn from(value: LayoutWithContext) -> Self {
        Box::new(value)
    }
}
