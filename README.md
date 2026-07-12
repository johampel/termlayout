# termlayout

[![Crates.io](https://img.shields.io/crates/v/termlayout.svg)](https://crates.io/crates/termlayout)
[![Documentation](https://docs.rs/termlayout/badge.svg)](https://docs.rs/termlayout)
[![License](https://img.shields.io/crates/l/termlayout.svg)](https://github.com/johampel/termlayout#license)

A declarative terminal layout library for Rust.

`termlayout` provides a set of widgets and layout primitives to build complex terminal user interfaces or formatted text output with automatic wrapping, alignment, and ANSI styling support.

## Why termlayout?

- **Declarative & Composable**: Build complex layouts by composing simple widgets
- **ANSI-Aware**: Proper handling of ANSI escape sequences for colors and styles
- **Flexible Layout Engine**: Automatic dimension calculation, word wrapping, and clipping
- **Rich Widget Library**: Tables, paragraphs, columns, fillers, and more
- **Extensible**: Easy to create custom widgets using the `Layout` trait
- **Zero Runtime Dependencies**: Optional features for extended functionality

## Installation

Add `termlayout` to your `Cargo.toml`:

```toml
[dependencies]
termlayout = "0.1.0"
```

Or use `cargo add`:

```bash
cargo add termlayout
```

### Optional Features

- `markdown`: Markdown rendering support

To enable the `markdown` feature, add it to your dependencies:

```toml
[dependencies]
termlayout = { version = "0.1.0", features = ["markdown"] }
```

## Features

### Rich Widget Library

- **`Paragraph`**: Word-wrapped text with left, right, center, or block alignment
- **`Lines`**: Line-by-line text display with alignment options
- **`Table`**: Flexible tables with headers, borders, and configurable column widths
- **`Horizontal`/`Vertical`**: Stack layouts horizontally or vertically
- **`Cell`**: Container with padding, clipping, and splitting capabilities
- **`Filler`**: Pattern-based area filling (horizontal, vertical, or both)
- **`Markdown`**: Render Markdown documents (requires `markdown` feature)

### Styling System

- Full ANSI color support (8-bit, 24-bit RGB)
- Text effects: bold, italic, underline, strikethrough, etc.
- Style composition with `TextBuilder`
- Automatic style transition optimization

### Layout System

- Automatic dimension calculation (`pref_dim`, `min_dim`)
- Multiple wrap modes: `Truncate` with suffix, or `Wrap` to next line
- Clipping support for constrained areas
- Flexible column sizing: `Fixed`, `Minimal`, `Fill`, `Relative`

## Quick Start

### Simple Paragraph

```rust
use termlayout::*;
use termlayout::widgets::Paragraph;

fn main() {
    let text = "This is a sample paragraph that will be automatically \
                wrapped to fit the target width. It supports word wrapping \
                and various alignment options.";
    
    let layout = Paragraph::left(text);
    let output = layout.layout(40);
    println!("{}", output);
}
```

### Table Example

```rust
use termlayout::widgets::{Table, TableColumn, TableDecoration, Lines, CellWidth};
use termlayout::Layout;

fn main() {
    let table = Table::new(
        TableDecoration::default(),
        vec![
            TableColumn::default()
                .with_header(Lines::left("Name"))
                .with_width(CellWidth::Minimal),
            TableColumn::default()
                .with_header(Lines::left("Description"))
                .with_width(CellWidth::Fill),
        ],
        vec![
            vec![
                Lines::left("Paragraph").into(),
                Lines::left("Word-wrapped text with alignment").into(),
            ],
            vec![
                Lines::left("Table").into(),
                Lines::left("Flexible tables with headers and borders").into(),
            ],
        ],
    );

    let formatted = table.layout(60);
    println!("{}", formatted);
}
```

### Styled Text

```rust
use termlayout::ext::{Style, TextBuilder, Color, Effect};

fn main() {
    let mut builder = TextBuilder::new();
    
    builder.push_str("Normal text ");
    builder.push_style(Style::default().with_foreground(Color::Red));
    builder.push_str("red text ");
    builder.pop_style();
    builder.push_style(Style::default().with_effect(Effect::Bold));
    builder.push_str("bold text");
    builder.pop_style();
    
    println!("{}", builder.build());
}
```

### Custom Widgets

Create your own widgets by implementing the `Layout` trait:

```rust
use termlayout::*;

struct MyWidget {
    content: String,
}

impl Layout for MyWidget {
    fn pref_dim(&self, max_width: usize, wrap_mode: WrapMode) -> Dimension {
        // Calculate preferred dimensions
        Dimension::new(max_width.min(self.content.len()), 1)
    }

    fn min_dim(&self) -> Dimension {
        Dimension::new(self.content.len(), 1)
    }

    fn layout_strict(&self, options: LayoutOptions) -> BoxedFormattedLayout {
        // Implement layout logic
        todo!()
    }
}
```

See `examples/two_columns.rs` for a complete custom widget implementation.

## Examples

The repository includes several examples demonstrating different features:

```bash
# Table with multiple widgets
cargo run --example table_example

# Custom two-column layout widget
cargo run --example two_columns
```

## Development

### Run Tests

```bash
cargo test
```

### Run Tests with All Features

```bash
cargo test --all-features
```

### Build Documentation

```bash
cargo doc --open
```

### Check Lints

```bash
cargo clippy --all-features
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

Licensed under the MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be licensed under the MIT License, without any additional terms or conditions.
