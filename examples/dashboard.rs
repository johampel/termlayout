//! An interactive dashboard example showcasing termlayout's grid, vertical/horizontal splitting,
//! frames, lists, table formatting, and styling.
//!
//! You can run this example using:
//! ```bash
//! cargo run --example dashboard
//! ```

use termlayout::ext::{Color, Effect, Style, TextBuilder};
use termlayout::widgets::{
    Cell, CellAnchor, CellWidth, Filler, Frame, FrameDecoration, Horizontal, Lines, List, Paragraph,
    Table, TableColumn, TableDecoration, Vertical,
};
use termlayout::{Layout, RcLayout, WrapMode};

fn main() {
    // 1. Build a styled Header using TextBuilder
    let mut header_builder = TextBuilder::new();
    header_builder.push_style(
        Style::default()
            .with_foreground(Color::Cyan)
            .with_effect(Effect::Bold),
    );
    header_builder.append(" 🖥️  SYSTEM CONTROL DASHBOARD ");
    header_builder.pop_last_style();
    header_builder.push_style(
        Style::default()
            .with_foreground(Color::White)
            .with_effect(Effect::Dim),
    );
    header_builder.append(" | v0.1.0 | ");
    header_builder.pop_last_style();
    header_builder.push_style(Style::default().with_foreground(Color::Yellow));
    header_builder.append("Status: ONLINE ");
    
    // Convert current builder content to layout
    let header_text = Lines::center(header_builder.as_ref().to_string());
    let header_separator = Filler::horizontal("═");

    // 2. Build the Sidebar (Left Column)
    // Quick statistics with colored badges using TextBuilder
    let mut item1 = TextBuilder::new();
    item1.append("CPU Usage:  ");
    item1.push_style(Style::default().with_foreground(Color::Green).with_effect(Effect::Bold));
    item1.append("⚡ 14% ");

    let mut item2 = TextBuilder::new();
    item2.append("Memory:     ");
    item2.push_style(Style::default().with_foreground(Color::Green).with_effect(Effect::Bold));
    item2.append("📟 4.2/16 GB ");

    let mut item3 = TextBuilder::new();
    item3.append("Disk Space: ");
    item3.push_style(Style::default().with_foreground(Color::Yellow).with_effect(Effect::Bold));
    item3.append("💾 74% ");

    let mut item4 = TextBuilder::new();
    item4.append("Network:    ");
    item4.push_style(Style::default().with_foreground(Color::Green).with_effect(Effect::Bold));
    item4.append("📶 Up ");

    let stats_list = List::fixed(vec![
        Paragraph::left(item1.as_ref().to_string()),
        Paragraph::left(item2.as_ref().to_string()),
        Paragraph::left(item3.as_ref().to_string()),
        Paragraph::left(item4.as_ref().to_string()),
    ]);

    let sidebar = Frame::new(
        FrameDecoration::boxed(),
        Some("System Info".to_string()),
        stats_list,
    );

    // 3. Build the Processes Table (Right Column)
    // We will define columns with custom widths (Fixed, Minimal, Fill) and alignments/anchors.
    let table_columns = vec![
        TableColumn::default()
            .with_header(Lines::center("PID"))
            .with_width(CellWidth::Fixed(6))
            .with_anchor(CellAnchor::Center),
        TableColumn::default()
            .with_header(Lines::left("Process Name"))
            .with_width(CellWidth::Fill)
            .with_anchor(CellAnchor::West),
        TableColumn::default()
            .with_header(Lines::center("CPU %"))
            .with_width(CellWidth::Fixed(8))
            .with_anchor(CellAnchor::Center),
        TableColumn::default()
            .with_header(Lines::left("Status"))
            .with_width(CellWidth::Fixed(12))
            .with_anchor(CellAnchor::West),
    ];

    // Table rows showing active processes with styled status badges
    let row_data = vec![
        vec!["1001", "nginx-gateway", "0.8%", "Running"],
        vec!["1002", "redis-cache", "1.2%", "Running"],
        vec!["1003", "postgresql-db", "4.5%", "Running"],
        vec!["1004", "node-api-worker", "7.1%", "Running"],
        vec!["1005", "rust-aggregator", "0.0%", "Idle"],
        vec!["1006", "cron-backup", "0.0%", "Finished"],
    ];

    let mut table_rows: Vec<Vec<RcLayout>> = Vec::new();
    for row in row_data {
        let pid = Lines::center(row[0]);
        let name = Paragraph::left(row[1]);
        let cpu = Lines::center(row[2]);

        // Color status green if Running, yellow if Idle, default if Finished
        let mut status_builder = TextBuilder::new();
        match row[3] {
            "Running" => {
                status_builder.push_style(Style::default().with_foreground(Color::Green));
                status_builder.append("● Running");
            }
            "Idle" => {
                status_builder.push_style(Style::default().with_foreground(Color::Yellow));
                status_builder.append("○ Idle");
            }
            _ => {
                status_builder.push_style(Style::default().with_foreground(Color::White).with_effect(Effect::Dim));
                status_builder.append("◌ Finished");
            }
        }
        let status = Lines::left(status_builder.as_ref().to_string());

        table_rows.push(vec![pid.into(), name.into(), cpu.into(), status.into()]);
    }

    let process_table = Table::new(
        TableDecoration::boxed_grid(),
        table_columns,
        table_rows,
    );

    let details_panel = Frame::new(
        FrameDecoration::boxed(),
        Some("Active Processes".to_string()),
        process_table,
    );

    // 4. Combine Left (Sidebar) and Right (Details) into a Horizontal Layout
    // We set the sidebar column to Fixed width, and the details panel to fill the rest.
    let left_cell = Cell::of(sidebar).with_width(CellWidth::Fixed(26));
    let right_cell = Cell::of(details_panel).with_width(CellWidth::Fill);

    let main_content = Horizontal::new(vec![left_cell, right_cell], None);

    // 5. Build a styled Footer with hotkey hints
    let mut footer_builder = TextBuilder::new();
    footer_builder.push_style(Style::default().with_foreground(Color::Cyan).with_effect(Effect::Bold));
    footer_builder.append(" [Q] ");
    footer_builder.pop_last_style();
    footer_builder.append("Quit   ");
    
    footer_builder.push_style(Style::default().with_foreground(Color::Cyan).with_effect(Effect::Bold));
    footer_builder.append(" [R] ");
    footer_builder.pop_last_style();
    footer_builder.append("Refresh   ");
    
    footer_builder.push_style(Style::default().with_foreground(Color::Cyan).with_effect(Effect::Bold));
    footer_builder.append(" [S] ");
    footer_builder.pop_last_style();
    footer_builder.append("Settings   ");

    let footer_text = Lines::left(footer_builder.as_ref().to_string());
    let footer_separator = Filler::horizontal("─");

    // 6. Stack everything into a Vertical Layout
    let dashboard = Vertical::from([
        header_text.into(),
        header_separator.into(),
        main_content.into(),
        footer_separator.into(),
        footer_text.into(),
    ]);

    // 7. Render layout targeting a terminal width of 80 characters
    let width = 80;
    println!("{}", dashboard.layout_with_wrap_mode(width, WrapMode::Wrap));
}
