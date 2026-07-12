//! Beispiel für die Verwendung des Table-Widgets

use termlayout::Layout;
use termlayout::widgets::{CellWidth, Lines, Paragraph, Table, TableColumn, TableDecoration};

#[path = "shared/mod.rs"]
mod shared;
use crate::shared::{Menu, MenuItem, clear_screen};
use shared::get_output_width;

fn show_example() {
    // Simply shows in tabular fashion all available Widgets

    let table = Table::new(
        TableDecoration::default(),
        vec![
            TableColumn::default()
                .with_header(Lines::left("Widget"))
                .with_width(CellWidth::Minimal),
            TableColumn::default()
                .with_header(Lines::left("Description"))
                .with_width(CellWidth::Fill),
            TableColumn::default()
                .with_header(Lines::left("Example"))
                .with_width(CellWidth::Minimal),
        ],
        vec![
            vec![
                Lines::left("Filler").into(),
                Paragraph::left(concat!(
                    "Fills the available area with a specified pattern. ",
                    "The pattern might repeated in horizontal and/or vertical direction. ",
                ))
                .into(),
                Lines::left("Filler::both(\"#\")").into(),
            ],
            vec![
                Lines::left("Lines").into(),
                Paragraph::left(concat!(
                    "Displays the text line-wise. ",
                    "Lines are demarked by new-line characters in the floating text. ",
                    "You may align it left, centered, or right."
                ))
                .into(),
                Lines::left("Lines::left(\"First line\\nSecond line\")").into(),
            ],
            vec![
                Lines::left("Paragraph").into(),
                Paragraph::left(concat!(
                "Displays the text word-wise. ",
                "The text is split into words and the widget writes the words one by one to fill ",
                "the available area. You may align it left, centered, right, or even block-wise."
                )).into(),
                Lines::left("Paragrah::left(\"This is a paragraph\")").into(),
            ],
        ],
    );

    let cols = get_output_width();
    let formatted = table.layout(cols);
    print!("{formatted}");
}

fn main() {
    let menu = Menu::new(&[MenuItem::options(), MenuItem::quit()]);
    loop {
        show_example();
        menu.show_and_handle_menu();
        clear_screen();
    }
}
