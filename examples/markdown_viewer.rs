//! Beispiel für die Verwendung des Markdown-Widgets

use std::fs;
use termlayout::widgets::Markdown;
use termlayout::{Layout, MarkdownConfig};

#[path = "shared/mod.rs"]
mod shared;
use crate::shared::{Menu, MenuItem, clear_screen};
use shared::get_output_width;

fn show_example() {
    let readme = fs::read_to_string("examples/doc/EXAMPLE.md")
        .expect("README.md konnte nicht gelesen werden");
    let markdown = Markdown::with_config(readme, &MarkdownConfig::bright());

    let cols = get_output_width();
    let formatted = markdown.layout(cols);
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
