//! This module contains extensions and utilities for the core layout system.
//!
//! It provides macros for simplifying the implementation of layouts, as well as
//! traits and structs that enhance the functionality of the base `Layout` trait.
//!
//! Key components include:
//! - **Macros**: `rc_layout!` and `box_formatted_layout!` for boilerplate reduction.
//! - **Layout Writers**: `BaseLayoutWriter` for common writing operations.
//! - **String Utilities**: `DisplayStr`, `Fragment`, `DisplayLines`, and `DisplayWords` for advanced text processing.
//! - **Reference Counting**: Support for wrapping layouts in `Rc` for shared ownership.

/// Provides macros for simplifying the implementation of layouts.
#[macro_use]
pub mod macros;
mod base_layout_writer;
mod layout_with_options;

pub use crate::box_formatted_layout;
pub use crate::rc_layout;

pub use crate::core::geometry::RangeExt;
pub use crate::core::str::DisplayLines;
pub use crate::core::str::DisplayStr;
pub use crate::core::str::DisplayWords;
pub use crate::core::str::Fragment;
pub use crate::core::style::Color;
pub use crate::core::style::Effect;
pub use crate::core::style::Style;
pub use crate::core::style::Transition;
pub use crate::core::textbuilder::TextBuilder;
pub use crate::core::utils::TakeBackIterator;

pub use crate::core::layout::BoxedFormattedLayout;
pub use crate::core::layout::BoxedLayoutWriter;
pub use crate::core::layout::FormattedLayout;
pub use crate::core::layout::LayoutWriter;
pub use crate::core::layout::SizedLayoutResult;

pub use crate::ext::base_layout_writer::BaseLayoutWriter;
pub use crate::ext::base_layout_writer::StrLayoutResult;
pub use crate::ext::base_layout_writer::VoidLayoutResult;

pub use crate::ext::layout_with_options::LayoutWithOptions;
