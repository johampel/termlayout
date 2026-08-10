//! Shared utilities for the interactive examples.
//!
//! Provides a simple terminal menu system ([`Menu`], [`MenuItem`]) and helpers for
//! clearing the screen and querying the output width.
#![allow(dead_code)]

use std::io::Write;
use std::sync::Mutex;
use termlayout::ext::{Color, Effect, Style, TextBuilder};
use termlayout::widgets::{Filler, Paragraph, Vertical};
use termlayout::{Layout, widgets};

/// Fallback output width used when the terminal size cannot be determined.
const DEFAULT_WIDTH: usize = 80;
/// Overrides the output width when set to a value greater than zero.
/// Set to `0` to use the actual terminal width instead.
static WIDTH: Mutex<usize> = Mutex::new(0);

/// Returns the current output width for the example.
///
/// If [`WIDTH`] has been overridden to a positive value (via the "Set fixed output width" menu
/// option), that value is returned. Otherwise the width is read from the terminal via
/// [`termsize`], falling back to [`DEFAULT_WIDTH`] if the terminal size is unavailable.
pub(crate) fn get_output_width() -> usize {
    let width = WIDTH.lock().unwrap();
    if *width > 0 {
        *width
    } else {
        termsize::get().map_or(DEFAULT_WIDTH, |ts| ts.cols as usize)
    }
}

/// Clears the terminal screen and moves the cursor to the top-left corner.
pub(crate) fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
}

/// A simple terminal menu that renders a list of [`MenuItem`]s and handles user input.
pub(crate) struct Menu {
    items: Vec<MenuItem>,
}

impl Menu {
    /// Creates a new `Menu` from the given slice of [`MenuItem`]s.
    pub(crate) fn new(items: &[MenuItem]) -> Self {
        Self {
            items: items.to_vec(),
        }
    }

    /// Renders the menu and processes user input in a loop until an action signals completion.
    ///
    /// Each iteration displays the menu, reads a key, executes the corresponding action, and
    /// repeats if the action returns `false`. Returns when an action returns `true`.
    pub(crate) fn show_and_handle_menu(&self) {
        loop {
            self.show();
            match self.select() {
                Some(item) => {
                    if (item.action)() {
                        break;
                    }
                }
                _ => Menu::show_error("Invalid input. Please try again."),
            }
        }
    }

    /// Prints a styled error message in bold red to stdout.
    pub(crate) fn show_error<T>(message: T)
    where T: AsRef<str> {
        let mut error = TextBuilder::new();
        error.push_style(
            Style::default()
                .with_effect(Effect::Bold)
                .with_foreground(Color::Red),
        );
        error.append(message);
        error.pop_last_style();
        println!("{}", error.as_ref());
    }

    fn show(&self) {
        let menu = Vertical::from([
            Paragraph::left_with_style(
                Style::default().with_effect(Effect::Bold),
                "Select an action:",
            )
            .into(),
            Filler::horizontal("─").into(),
            widgets::Menu::new(self.items.clone().into_iter().map(|a| a.item).collect()).into(),
            Filler::horizontal("─").into(),
        ]);

        let cols = get_output_width();
        let formatted = menu.layout(cols);
        print!("{formatted}");
    }

    fn select(&self) -> Option<MenuItem> {
        let response = Menu::prompt("Enter your choice");
        let response = response.trim();

        // Try to figure out the choice based on the first character
        if response.len() == 1 {
            let ch = response.chars().next().unwrap();
            self.items
                .iter()
                .find(|a| a.item.key == ch)
                .cloned()
        } else {
            None
        }
    }

    /// Prints a prompt to stdout, reads a line from stdin, and returns the trimmed input.
    pub(crate) fn prompt(prompt: &str) -> String {
        print!("{prompt}: ");
        std::io::stdout().flush().unwrap();
        let mut response = String::new();
        std::io::stdin().read_line(&mut response).unwrap();
        response.trim_end().to_string()
    }
}

/// A menu item with a key binding, display text, and an associated action.
///
/// When the user presses the item's key, the `action` function is called. The action returns
/// `true` to signal that the menu loop should exit, or `false` to keep the menu open.
#[derive(Clone)]
pub(crate) struct MenuItem {
    item: widgets::MenuItem,
    action: fn() -> bool,
}

impl MenuItem {
    /// Creates a new `MenuItem` with the given key, display text, and action.
    pub(crate) fn new<T>(key: char, text: T, action: fn() -> bool) -> Self
    where
        T: Into<String>,
    {
        Self {
            item: widgets::MenuItem::new(key, Paragraph::left(text)),
            action,
        }
    }

    /// Returns a [`MenuItem`] that exits the example process immediately.
    pub(crate) fn quit() -> MenuItem {
        MenuItem::new('q', "Quit the example", || std::process::exit(0))
    }

    /// Returns a [`MenuItem`] that closes a nested menu and goes back to the caller.
    pub(crate) fn back() -> MenuItem {
        MenuItem::new('b', "Go back to example", || true)
    }

    /// Returns a [`MenuItem`] that opens a sub-menu for changing the layout output width.
    pub(crate) fn options() -> MenuItem {
        MenuItem::new('o', "Change the layout width", || {
            let menu = Menu::new(&[
                MenuItem::use_terminal_width(),
                MenuItem::use_fixed_width(),
                MenuItem::back(),
                MenuItem::quit(),
            ]);
            menu.show_and_handle_menu();
            true
        })
    }

    /// Returns a [`MenuItem`] that prompts the user for an explicit output width.
    pub(crate) fn use_fixed_width() -> MenuItem {
        MenuItem::new('f', "Set fixed output width", || {
            let width = Menu::prompt("Enter the new output width");
            if let Ok(w) = width.trim().parse() {
                *WIDTH.lock().unwrap() = w;
                true
            } else {
                Menu::show_error("Invalid input.");
                false
            }
        })
    }
    /// Returns a [`MenuItem`] that resets the output width to the actual terminal width.
    pub(crate) fn use_terminal_width() -> MenuItem {
        MenuItem::new('t', "Use terminal width as output width", || {
            *WIDTH.lock().unwrap() = 0;
            true
        })
    }
}
