//! Example demonstrating the [`Markdown`](crate::widgets::Markdown) widget.
//! It is an interactive example that shows the content of a markdown file; using a simple menu
//! you may change the file interactively when the example runs.

use std::borrow::Cow;
use std::fs;
use std::sync::Mutex;
use termlayout::widgets::Markdown;
use termlayout::{Layout, MarkdownConfig};

#[path = "shared/mod.rs"]
mod shared;
use crate::shared::{Menu, MenuItem, clear_screen};
use shared::get_output_width;

/// This is the file with the markdown content. You may change it interactively when the example
/// runs.
static FILENAME: Mutex<Cow<str>> = Mutex::new(Cow::Borrowed("examples/doc/EXAMPLE.md"));
/// This is the indicator whether the dark style should be used. You may change it interactively
/// when the example runs.
static DARK_STYLE: Mutex<bool> = Mutex::new(true);


/// This is the code you need to load a file and display its content as markdown.
fn show_example() {
    // Open the file and read its content.
    let filename = (*FILENAME.lock().unwrap()).to_string();
    match fs::read_to_string(filename.as_str()) {
        Ok(content) => {
            // Create the markdown widget with the content and the appropriate configuration.
            let config = match *DARK_STYLE.lock().unwrap() {
                true => &MarkdownConfig::dark(),
                false => &MarkdownConfig::light()
            };
            let markdown = Markdown::with_config(content, config);

            // Get the current output width to display.
            let cols = get_output_width();

            // Go
            let formatted = markdown.layout(cols);
            print!("{formatted}");
        }
        Err(e) => {
            Menu::show_error(format!("failed to load {filename}: {e}"));
        }
    }
}


/// Returns a [`MenuItem`] that allows the user to select a file to show.
fn select_file() -> MenuItem {
    let filename = (*FILENAME.lock().unwrap()).to_string();
    MenuItem::new('f', format!("Select file to show (current: {filename})"), || {
        let filename = Menu::prompt("Enter file name");
        *FILENAME.lock().unwrap() = filename.to_string().into();
        true
    })
}

/// Returns a [`MenuItem`] that allows the user to toggle between light and dark style.
fn toggle_style() -> MenuItem {
    let style = *DARK_STYLE.lock().unwrap();
    let style_name = match style {
        true => String::from("dark"),
        false => String::from("light"),
    };
    MenuItem::new('s', format!("Toggle between light and dark style (current: {style_name})"), || {
        let style = *DARK_STYLE.lock().unwrap();
        *DARK_STYLE.lock().unwrap() = (!style).into();
        true
    })

}

/// Main.
/// Runs a in loop:
/// 1. Clear the screen to start
/// 2. Show the markdown file.
/// 3. Wait for user input to select and execute an option. One of the options is to quit
fn main() {
    loop {
        clear_screen();

        show_example();

        let menu = Menu::new(&[
            select_file(),
            toggle_style(),
            MenuItem::options(),
            MenuItem::quit()]);
        menu.show_and_handle_menu();
    }
}
