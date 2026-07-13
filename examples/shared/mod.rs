//! Shared utilities for examples
#![allow(dead_code)]

use std::io::Write;
use std::sync::Mutex;
use termlayout::ext::{Color, Effect, Style, TextBuilder};
use termlayout::widgets::{Filler, Paragraph, Vertical};
use termlayout::{Layout, widgets};

const DEFAULT_WIDTH: usize = 80;
static WIDTH: Mutex<usize> = Mutex::new(0);

/// Gets the terminal width, defaulting to 80 if unavailable
pub(crate) fn get_output_width() -> usize {
    let width = WIDTH.lock().unwrap();
    if *width > 0 {
        *width
    } else {
        termsize::get().map_or(DEFAULT_WIDTH, |ts| ts.cols as usize)
    }
}

pub(crate) fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
}

pub(crate) struct Menu {
    items: Vec<MenuItem>,
}

impl Menu {
    pub(crate) fn new(items: &[MenuItem]) -> Self {
        Self {
            items: items.to_vec(),
        }
    }

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

    pub(crate) fn prompt(prompt: &str) -> String {
        print!("{prompt}: ");
        std::io::stdout().flush().unwrap();
        let mut response = String::new();
        std::io::stdin().read_line(&mut response).unwrap();
        response.trim_end().to_string()
    }
}

#[derive(Clone)]
pub(crate) struct MenuItem {
    item: widgets::MenuItem,
    action: fn() -> bool,
}

impl MenuItem {
    pub(crate) fn new<T>(key: char, text: T, action: fn() -> bool) -> Self
    where
        T: Into<String>,
    {
        Self {
            item: widgets::MenuItem::new(key, Paragraph::left(text)),
            action,
        }
    }

    pub(crate) fn quit() -> MenuItem {
        MenuItem::new('q', "Quit the example", || std::process::exit(0))
    }

    pub(crate) fn back() -> MenuItem {
        MenuItem::new('b', "Go back to example", || true)
    }

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
    pub(crate) fn use_terminal_width() -> MenuItem {
        MenuItem::new('t', "Use terminal width as output width", || {
            *WIDTH.lock().unwrap() = 0;
            true
        })
    }
}
