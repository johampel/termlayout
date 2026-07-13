//! A newspaper-style layout example showcasing text wrapping and alignment modes:
//! Left, Center, Right, and Block (justified).
//!
//! You can run this example using:
//! ```bash
//! cargo run --example newspaper
//! ```

use termlayout::ext::{Color, Effect, Style, TextBuilder};
use termlayout::widgets::{
    Cell, CellWidth, Filler, Frame, FrameDecoration, Horizontal, Lines, Paragraph, Vertical,
};
use termlayout::{Layout, WrapMode};

fn main() {
    // 1. Newspaper Header (Masthead)
    let masthead = Frame::new(
        FrameDecoration::double_boxed(),
        None,
        Vertical::from([
            Lines::center("THE DECLARATIVE GAZETTE").into(),
            Lines::center("Your Daily Source of Terminal Layout News").into(),
        ]),
    );

    // 2. Subtitle with Date and Edition (using a split-column look)
    let mut date_builder = TextBuilder::new();
    date_builder.push_style(Style::default().with_effect(Effect::Bold));
    date_builder.append("Edition #42");
    date_builder.pop_last_style();

    let mut price_builder = TextBuilder::new();
    price_builder.push_style(Style::default().with_foreground(Color::Green));
    price_builder.append("Price: FREE");
    price_builder.pop_last_style();

    let metadata_row = Horizontal::new(
        vec![
            Cell::of(Lines::left(date_builder.as_ref().to_string())).with_width(CellWidth::Fill),
            Cell::of(Lines::center("July 13, 2026")).with_width(CellWidth::Fill),
            Cell::of(Lines::right(price_builder.as_ref().to_string())).with_width(CellWidth::Fill),
        ],
        None,
    );

    // 3. Three-column articles with different alignments
    // Column 1: Justified (Block) Text
    let col1_text = "The layout engine is designed to be fully declarative, \
                     allowing developers to compose sophisticated text interfaces \
                     by combining basic blocks. This is a justified column \
                     demonstrating the 'Block' alignment mode, which distributes \
                     spacing evenly between words to flush-fit both margins.";
    let col1 = Paragraph::block(col1_text);

    // Column 2: Left-Aligned Text
    let col2_text = "By nesting vertical and horizontal stacks, you can build \
                     complex grids and layouts. Auto-wrapping ensures words are \
                     never broken mid-character unless they exceed the available \
                     column width entirely, in which case clipping or custom wrapping occurs.";
    let col2 = Paragraph::left(col2_text);

    // Column 3: Right-Aligned Text
    let col3_text = "ANSI styling parameters are tracked through a styling stack \
                     using a TextBuilder. This ensures style transitions emit the \
                     minimum escape sequences necessary, keeping the stdout stream \
                     incredibly light and highly responsive for terminal rendering.";
    let col3 = Paragraph::right(col3_text);

    // Assemble the columns with a vertical separator " │ "
    let columns_section = Horizontal::new(
        vec![
            Cell::of(col1).with_width(CellWidth::Fill),
            Cell::of(col2).with_width(CellWidth::Fill),
            Cell::of(col3).with_width(CellWidth::Fill),
        ],
        Some(Filler::vertical(" │ ").into()),
    );

    // 4. A footer banner with some styled call-to-action
    let mut footer_builder = TextBuilder::new();
    footer_builder.push_style(
        Style::default()
            .with_foreground(Color::Magenta)
            .with_effect(Effect::Inverse),
    );
    footer_builder.append(" READ ALL ABOUT IT! ");
    footer_builder.pop_last_style();
    footer_builder.append(" Powered by termlayout crate. Declarative formatting made easy.");
    let footer = Lines::center(footer_builder.as_ref().to_string());

    // 5. Combine everything vertically
    let page = Vertical::from([
        masthead.into(),
        metadata_row.into(),
        Filler::horizontal("─").into(),
        columns_section.into(),
        Filler::horizontal("═").into(),
        footer.into(),
    ]);

    // Render with 90 characters width
    println!("{}", page.layout_with_wrap_mode(90, WrapMode::Wrap));
}
