pub mod decoration;

use crate::ext::{Effect, LayoutWithOptions, Style};
use crate::widgets::key_value_view::decoration::KeyValueViewDecoration;
use crate::widgets::{CellAnchor, CellWidth, Lines, Table, TableColumn};
use crate::{BoxedFormattedLayout, Dimension, Layout, LayoutOptions, RcLayout, WrapMode};
use std::any::Any;

pub struct KeyValueView {
    pub decoration: KeyValueViewDecoration,
    pub entries: Vec<Entry>,
}

impl KeyValueView {
    pub fn new(entries: Vec<impl Into<Entry>>) -> Self {
        Self {
            decoration: KeyValueViewDecoration::default(),
            entries: entries.into_iter().map(Into::into).collect(),
        }
    }

    fn to_table(&self) -> Table {
        Table::new(
            self.decoration.table.clone(),
            vec![
                TableColumn::new(
                    Some(
                        Lines::left_with_style(
                            Style::default().with_effect(Effect::Bold).into(),
                            self.decoration.key_header.as_str(),
                        )
                        .into(),
                    ),
                    CellWidth::Minimal,
                    CellAnchor::West,
                    WrapMode::default(),
                ),
                TableColumn::new(
                    Some(
                        Lines::left_with_style(
                            Style::default().with_effect(Effect::Bold).into(),
                            self.decoration.value_header.as_str(),
                        )
                        .into(),
                    ),
                    CellWidth::Minimal,
                    CellAnchor::Fill,
                    WrapMode::default(),
                ),
            ],
            self.entries
                .iter()
                .map(|entry| entry.into())
                .collect::<Vec<_>>(),
        )
    }
}

impl Layout for KeyValueView {
    fn pref_dim(&self, max_width: usize, wrap_mode: WrapMode) -> Dimension {
        self.to_table().pref_dim(max_width, wrap_mode)
    }

    fn min_dim(&self) -> Dimension {
        self.to_table().min_dim()
    }

    fn layout_strict(&'_ self, options: LayoutOptions) -> BoxedFormattedLayout<'_> {
        LayoutWithOptions::of(self.to_table().into(), options).into()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct Entry {
    key: String,
    value: Value,
}

impl Entry {
    pub fn new<K: Into<String>, V: Into<Value>>(key: K, value: V) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

impl From<&Entry> for Vec<RcLayout> {
    fn from(value: &Entry) -> Self {
        vec![
            Lines::left_with_style(
                Style::default().with_effect(Effect::Bold).into(),
                value.key.as_str(),
            )
            .into(),
            (&value.value).into(),
        ]
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Text(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
}

impl From<&Value> for RcLayout {
    fn from(value: &Value) -> Self {
        match value {
            Value::Text(text) => Lines::left(text).into(),
            Value::Integer(integer) => Lines::right(format!("{integer}")).into(),
            Value::Float(float) => Lines::right(format!("{float}")).into(),
            Value::Boolean(boolean) => {
                if *boolean {
                    Lines::center("[x]").into()
                } else {
                    Lines::center("[ ]").into()
                }
            }
        }
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::Text(value.to_owned())
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::Layout;
    use crate::widgets::KeyValueView;
    use crate::widgets::key_value_view::Entry;

    #[test]
    fn test() {
        let entries = vec![
            Entry::new("key1", "value1"),
            Entry::new("key2", 123),
            Entry::new("key3", 456.78),
            Entry::new("key4", true),
        ];
        let view = KeyValueView::new(entries);

        let formatted = view.layout(20);
        assert_eq!(
            format!("{formatted}"),
            concat!(
                "┌────┬──────┐\n",
                "│\u{1b}[1m\u{1b}[1m\u{1b}[38;5;33mKey\u{1b}[0m │\u{1b}[1m\u{1b}[1m\u{1b}[38;5;33mValue \u{1b}[0m│\n",
                "├────┼──────┤\n│\u{1b}[1mkey1\u{1b}[0m│value1│\n",
                "│\u{1b}[1mkey2\u{1b}[0m│   123│\n",
                "│\u{1b}[1mkey3\u{1b}[0m│456.78│\n",
                "│\u{1b}[1mkey4\u{1b}[0m│ [x]  │\n",
                "└────┴──────┘\n"
            )
        );
    }
}
