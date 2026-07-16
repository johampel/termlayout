use crate::ext::{Color, TextBuilder};
use crate::widgets::TableDecoration;

pub struct KeyValueViewDecoration {
    pub table: TableDecoration,
    pub key_header: String,
    pub value_header: String,
}

impl KeyValueViewDecoration {
    pub fn new(
        key_header: impl Into<String>,
        value_header: impl Into<String>,
        table: TableDecoration,
    ) -> Self {
        Self {
            table,
            key_header: key_header.into(),
            value_header: value_header.into(),
        }
    }
}

impl Default for KeyValueViewDecoration {
    fn default() -> Self {
        Self::new(
            TextBuilder::new()
                .with_bold()
                .with_color(Color::Custom8(33))
                .with_text("Key"),
            TextBuilder::new()
                .with_bold()
                .with_color(Color::Custom8(33))
                .with_text("Value"),
            TableDecoration::from_spec(concat!(
                "┌─┬─┐\n", //
                "│H│H│\n", //
                "├─┼─┤\n", //
                "│C│C│\n", //
                "│C│C│\n", //
                "└─┴─┘"
            ))
            .unwrap(),
        )
    }
}
