//! `termlayout` is a library for creating complex terminal layouts using a declarative approach.
//! It provides a set of widgets and layout primitives that can be composed to build
//! sophisticated terminal user interfaces or formatted text output.
//!
//! # Core Concepts
//! - **[`Layout`]**: The fundamental trait representing anything that can be laid out.
//! - **[`Dimension`]**: Represents the width and height of a layout or part of it.
//! - **[`Rect`]**: A rectangular area defined by a position and a dimension.
//! - **[`LayoutOptions`]**: Configuration for the layout process, including target dimensions and wrapping modes.
//!
//! # Widgets
//! The library provides several built-in widgets in the [`widgets`] module:
//! - [`widgets::Filler`]: Fills an area with a specific pattern.
//! - [`widgets::Lines`]: Displays text line-by-line with horizontal alignment.
//! - [`widgets::Paragraph`]: Displays text word-by-word with support for various alignments, including block alignment.
//! - [`widgets::Table`]: A powerful table widget with support for headers, borders, and flexible column sizing.
//! - [`widgets::Vertical`]: Composes multiple layouts vertically.
//!
//! # Extensions
//! The [`ext`] module contains additional traits and utilities that extend the core layout
//! functionality, such as support for reference-counted layouts ([`RcLayout`]).

mod core;
pub mod ext;
pub mod widgets;

// Reexports
pub use core::measurements::MeasureMode;
pub use core::measurements::Measurements;
pub use core::context::LayoutContext;
pub use core::geometry::Dimension;
pub use core::geometry::Rect;
pub use core::layout::BoxedFormattedLayout;
pub use core::layout::Layout;
pub use core::layout::RcLayout;
pub use core::context::LayoutOptions;
pub use core::context::WrapMode;

#[cfg(feature = "markdown")]
pub use widgets::FrameConfig;
#[cfg(feature = "markdown")]
pub use widgets::MarkdownConfig;
