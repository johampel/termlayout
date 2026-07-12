use crate::ext::{
    BaseLayoutWriter, BoxedLayoutWriter, FormattedLayout, LayoutWriter, SizedLayoutResult,
};
use crate::{BoxedFormattedLayout, LayoutOptions, WrapMode, box_formatted_layout};
use std::fmt::Write;

pub(crate) struct FormattedTreeNode<'fmt> {
    prefix: (String, String),
    item: BoxedFormattedLayout<'fmt>,
    options: LayoutOptions,
}

impl<'fmt> FormattedTreeNode<'fmt> {
    pub(crate) fn new(
        prefix: (String, String),
        item: BoxedFormattedLayout<'fmt>,
        options: LayoutOptions,
    ) -> Self {
        Self {
            prefix,
            item,
            options,
        }
    }
}

impl FormattedLayout for FormattedTreeNode<'_> {
    fn options(&self) -> &LayoutOptions {
        &self.options
    }

    fn new_writer(&'_ self) -> BoxedLayoutWriter<'_> {
        Box::new(TreeNodeWriter::new(
            (self.prefix.0.as_str(), self.prefix.1.as_str()),
            self.item.new_writer(),
            &self.options,
        ))
    }
}

box_formatted_layout!(FormattedTreeNode);

struct TreeNodeWriter<'wrt> {
    base: BaseLayoutWriter<'wrt>,
    prefix: (&'wrt str, &'wrt str),
    item: BoxedLayoutWriter<'wrt>,
}

impl<'wrt> TreeNodeWriter<'wrt> {
    fn new(
        prefix: (&'wrt str, &'wrt str),
        item: BoxedLayoutWriter<'wrt>,
        options: &'wrt LayoutOptions,
    ) -> Self {
        Self {
            base: BaseLayoutWriter::new(options),
            prefix,
            item,
        }
    }
}

impl<'wrt> LayoutWriter<'wrt> for TreeNodeWriter<'wrt> {
    fn options(&self) -> &'wrt LayoutOptions {
        self.base.options()
    }

    fn write_row(&mut self, w: &mut dyn Write) -> SizedLayoutResult {
        let prefix = if self.base.row() == 0 {
            self.prefix.0
        } else {
            self.prefix.1
        };
        self.base.write_str(prefix, WrapMode::empty_truncate(), w)?;
        self.base.write_row(self.item.as_mut(), w)?;
        self.base.end_row(w)
    }
}
