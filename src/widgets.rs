//! Built-in layout widgets for common terminal UI elements.
//!
//! This module provides a collection of widgets that implement the [`Layout`](crate::Layout) trait.
//! These widgets can be used to build complex terminal layouts by composing them together.
//!
//! # Available Widgets
//! - [`Filler`]: Repeating patterns to fill space.
//! - [`Lines`]: Simple line-based text with alignment.
//! - [`Paragraph`]: Advanced word-wrapped text with multiple alignment modes.
//! - [`Table`]: Complex tabular data layout with flexible configuration.
//! - [`Vertical`]: Vertical stacking of multiple layouts.

pub(crate) mod cell;
pub(crate) mod filler;
pub(crate) mod frame;
pub(crate) mod horizontal;
pub(crate) mod lines;
pub(crate) mod list;
#[cfg(feature = "markdown")]
pub(crate) mod markdown;
pub(crate) mod menu;
pub(crate) mod paragraph;
pub(crate) mod table;
pub(crate) mod tree;
pub(crate) mod vertical;
pub use cell::Cell;
pub use cell::CellAnchor;
pub use cell::dimension::CellDimension;
pub use cell::dimension::CellWidth;

pub use filler::FillMode;
pub use filler::Filler;
pub use horizontal::Horizontal;
pub use horizontal::row::Row;
pub use lines::Lines;
pub use lines::LinesAlignment;

pub use frame::Frame;
pub use frame::decoration::FrameDecoration;
pub use frame::decoration::FrameDecorationKey;
pub use frame::decoration::TitlePlacement;
pub use list::List;
pub use list::ListItemEnumerator;
pub use list::ListItemMarker;
pub use menu::Menu;
pub use menu::MenuItem;
pub use menu::MenuItemMarker;
pub use paragraph::Paragraph;
pub use paragraph::ParagraphAlignment;
pub use table::Table;
pub use table::TableColumn;
pub use table::decoration::TableDecoration;
pub use tree::Tree;
pub use tree::TreeNode;
pub use tree::TreePath;
pub use tree::decoration::TreeDecoration;
pub use vertical::Vertical;

#[cfg(feature = "markdown")]
pub use markdown::FrameConfig;
#[cfg(feature = "markdown")]
pub use markdown::Markdown;
#[cfg(feature = "markdown")]
pub use markdown::MarkdownConfig;
