//! Example demonstrating the [`Tree`] widget.
//! It is an interactive example that renders a directory as a tree structure (only directories are
//! shown, files are omitted); using a simple menu you may change the displayed directory
//! interactively when the example runs.

use std::borrow::Cow;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use termlayout::widgets::{Paragraph, Tree, TreeDecoration, TreeNode};
use termlayout::Layout;

#[path = "shared/mod.rs"]
mod shared;
use crate::shared::{Menu, MenuItem, clear_screen};
use shared::get_output_width;

/// The directory whose structure is displayed. You may change it interactively when the example
/// runs.
static DIRECTORY: Mutex<Cow<str>> = Mutex::new(Cow::Borrowed("."));

/// Recursively builds a [`TreeNode`] for the given directory `path`.
///
/// Only sub-directories are turned into child nodes; files are ignored. Hidden directories (those
/// whose name starts with a `.`, e.g. `.git`) are skipped to keep the output focused on the
/// project structure. The children are sorted alphabetically by name so the output is
/// deterministic.
///
/// # Parameters
/// - `path`: The directory to build the tree node for
/// - `label`: The text displayed for this node (usually the directory's name)
///
/// # Returns
/// A [`TreeNode`] representing `path` and all of its sub-directories
fn build_node(path: &Path, label: &str) -> TreeNode {
    // Collect all sub-directories of `path`.
    let mut sub_dirs: Vec<_> = fs::read_dir(path)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .filter(|entry| !entry.file_name().to_string_lossy().starts_with('.'))
        .collect();

    // Sort them by their file name for a deterministic and readable output.
    sub_dirs.sort_by_key(std::fs::DirEntry::file_name);

    // Recursively build the child nodes.
    let children: Vec<TreeNode> = sub_dirs
        .iter()
        .map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            build_node(&entry.path(), &name)
        })
        .collect();

    TreeNode::new(Paragraph::left(label), children)
}

/// This is the code you need to render a directory as a tree of its sub-directories.
fn show_example() {
    let directory = (*DIRECTORY.lock().unwrap()).to_string();
    let path = Path::new(&directory);

    // Make sure the directory actually exists before we try to walk it.
    if !path.is_dir() {
        Menu::show_error(format!("not a directory: {directory}"));
        return;
    }

    // Build the tree of sub-directories and wrap it in a Tree widget.
    let root = build_node(path, &directory);
    let tree = Tree::new(TreeDecoration::lines(1), root, true);

    // Get the current output width and render.
    let cols = get_output_width();
    let formatted = tree.layout(cols);
    print!("{formatted}");
}

/// Returns a [`MenuItem`] that allows the user to select the directory to show.
fn select_directory() -> MenuItem {
    let directory = (*DIRECTORY.lock().unwrap()).to_string();
    MenuItem::new(
        'f',
        format!("Select directory to show (current: {directory})"),
        || {
            let directory = Menu::prompt("Enter directory");
            *DIRECTORY.lock().unwrap() = directory.into();
            true
        },
    )
}

/// Main.
/// Runs in a loop:
/// 1. Clear the screen to start
/// 2. Show the directory tree.
/// 3. Wait for user input to select and execute an option. One of the options is to quit.
fn main() {
    loop {
        clear_screen();

        show_example();

        let menu = Menu::new(&[
            select_directory(),
            MenuItem::options(),
            MenuItem::quit(),
        ]);
        menu.show_and_handle_menu();
    }
}
