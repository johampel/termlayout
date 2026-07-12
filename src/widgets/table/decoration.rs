use crate::ext::DisplayStr;
use crate::widgets::table::Table;
use crate::widgets::{Filler, TableColumn};
use crate::{RcLayout, WrapMode};
use std::cmp::max;
use std::collections::BTreeMap;
use std::fmt::Debug;

/// Decoration of a [`Table`].
/// The decoration defines whether to show the header row of a table and boxes around the table
/// cells. Instances are created via a "spec-string" that defines all aspects of the decoration.
///
/// # Spec string syntax
/// The spec-string syntax is best explained by an example. The following is a spec-string:
/// ```
/// let spec = concat!(
///     "┌─┬─┐\n", // Optional top line
///     "│H│H│\n", // Optional header-marker line
///     "├─┼─┤\n", // Optional header separator
///     "│C│C│\n", // Mandatory first cell-marker line
///     "├─┼─┤\n", // Optional row separator
///     "│C│C│\n", // Mandatory second cell-marker line
///     "└─┴─┘");  // Optional bottom line
/// ```
/// The spec-string is a visual representation of a decorated table, whereas header cells are
/// represented by the character "H" and data cells with "C". The characters around these cells
/// placeholders represent the decoration characters. For example, the "│" character between the
/// cell placeholders and on the left and right side defines that the cells are separated by this
/// character and the character is also used on the right and left side of the table.
/// It is also possible to have decorations consisting of more than one character:
/// ```
/// let spec = concat!(
///  //  +-------------- Left side decoration
///  //  |   +---------- Inter cell separator
///  //  ll mmm rr------ Right side decoration
///     "┌───┬───┐\n", // Optional top line
///     "│ H │ H │\n", // Optional header-marker line
///     "├───┼───┤\n", // Optional header separator
///     "│ C │ C │\n", // First cell-marker line (required)
///     "├───┼───┤\n", // Optional row separator
///     "│ C │ C │\n", // Second cell-marker line (required)
///     "└───┴───┘");  // Optional bottom line
/// ```
/// Mandatory are only the two cell-marker lines; if the header-marker line is omitted, no headers
/// are displayed. The following is the minimal decoration having no decoration at all and no headers:
/// ```
/// let spec = concat!(
///     "CC\n",
///     "CC");
/// ```
/// In general, the following syntactical constraints apply to the spec-string:
/// * There are exactly two cell-marker lines; the decoration for the cell line is taken from the
///   first one.
/// * There might be one header-marker line for the headers, which must appear before the first
///   cell marker line.
/// * All lines must have the same length.
/// * All markers must occur on the same column.
/// * Between two marker lines there must be at most one non-marker line.
/// * Before the first and after the last marker line there might be an additional line for the
///   top and bottom decoration.
#[derive(Debug, Clone)]
pub struct TableDecoration {
    rows: BTreeMap<DecorationRowType, DecorationRow>,
    has_left: bool,
    has_sep: bool,
    has_right: bool,
}

impl TableDecoration {
    fn new<T: Into<BTreeMap<DecorationRowType, DecorationRow>>>(
        rows: T,
        has_left: bool,
        has_sep: bool,
        has_right: bool,
    ) -> Self {
        Self {
            rows: rows.into(),
            has_left,
            has_sep,
            has_right,
        }
    }

    /// Creates a new [`TableDecoration`] from a string representation.
    /// Please refer to the documentation of [`TableDecoration`] for the format of the string.
    ///
    /// # Parameters
    /// - `cell_marker`: The character used to mark the cells
    /// - `header_marker`: The character used to mark the headers
    /// - `spec`: The string representation of the decoration
    ///
    /// # Returns
    /// The decoration or an error.
    ///
    /// # Errors
    /// Returns an error if the string representation of the specification is invalid.
    pub fn from_spec_custom(
        cell_marker: char,
        header_marker: char,
        spec: &str,
    ) -> Result<Self, String> {
        DecorationSpecParser::parse(cell_marker, header_marker, spec)
    }

    /// Creates a new [`TableDecoration`] from a string representation.
    /// Please refer to the documentation of [`TableDecoration`] for the format of the string.
    ///
    /// # Parameters
    /// - `spec`: The string representation of the decoration
    ///
    /// # Returns
    /// The decoration or an error.
    ///
    /// # Errors
    /// Returns an error if the string representation of the specification is invalid.
    pub fn from_spec(spec: &str) -> Result<Self, String> {
        DecorationSpecParser::parse('C', 'H', spec)
    }

    /// Creates a [`TableDecoration`] having only one space between the cells in one row
    /// ```text
    /// Head1 Head2
    /// Cell1 Cell2
    /// Cell3 Cell4
    /// ```
    ///
    /// # Returns
    /// A [`TableDecoration`]
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn no_grid() -> Self {
        Self::from_spec(concat!(
            "H H\n", //
            "C C\n", //
            "C C"
        ))
        .unwrap()
    }

    /// Creates a [`TableDecoration`] having no header and only one space between the cells in one
    /// row
    /// ```text
    /// Cell1 Cell2
    /// Cell3 Cell4
    /// ```
    ///
    /// # Returns
    /// A [`TableDecoration`]
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn headless_no_grid() -> Self {
        Self::from_spec(concat!(
            "C C\n", //
            "C C"
        ))
        .unwrap()
    }

    /// Creates a [`TableDecoration`] having an inner grid and a outer box.
    /// The resulting table might look like:
    /// ```text
    /// ┌─────┬─────┐
    /// │Head1│Head2│
    /// ├─────┼─────┤
    /// │Cell1│Cell2│
    /// ├─────┼─────┤
    /// │Cell3│Cell4│
    /// └─────┴─────┘
    /// ```
    ///
    /// # Returns
    /// A [`TableDecoration`]
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn boxed_grid() -> Self {
        Self::from_spec(concat!(
            "┌─┬─┐\n", //
            "│H│H│\n", //
            "├─┼─┤\n", //
            "│C│C│\n", //
            "├─┼─┤\n", //
            "│C│C│\n", //
            "└─┴─┘"
        ))
        .unwrap()
    }
    /// Creates a [`TableDecoration`] having an inner grid and a outer double-lined box.
    /// The resulting table might look like:
    /// ```text
    /// ╔═════╤═════╗
    /// ║Head1│Head2║
    /// ╠═════╪═════╣
    /// ║Cell1│Cell2║
    /// ╟─────┼─────╢
    /// ║Cell3│Cell4║
    /// ╚═════╧═════╝
    /// ```
    ///
    /// # Returns
    /// A [`TableDecoration`]
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn double_boxed_grid() -> Self {
        Self::from_spec(concat!(
            "╔═╤═╗\n",
            "║H│H║\n",
            "╠═╪═╣\n",
            "║C│C║\n",
            "╟─┼─╢\n",
            "║C│C║\n",
            "╚═╧═╝\n",
        ))
        .unwrap()
    }

    /// Creates a [`TableDecoration`] having no headers, an inner grid and a outer box.
    /// The resulting table might look like:
    /// ```text
    /// ┌─────┬─────┐
    /// │Cell1│Cell2│
    /// ├─────┼─────┤
    /// │Cell3│Cell4│
    /// └─────┴─────┘
    /// ```
    ///
    /// # Returns
    /// A [`TableDecoration`]
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn headless_boxed_grid() -> Self {
        Self::from_spec(concat!(
            "┌─┬─┐\n", //
            "│C│C│\n", //
            "├─┼─┤\n", //
            "│C│C│\n", //
            "└─┴─┘"
        ))
        .unwrap()
    }

    /// Creates a [`TableDecoration`] having an inner grid.
    /// The resulting table might look like:
    /// ```text
    /// Head1│Head2
    /// ─────┼─────
    /// Cell1│Cell2
    /// ─────┼─────
    /// Cell3│Cell4
    /// ```
    ///
    /// # Returns
    /// A [`TableDecoration`]
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn grid() -> Self {
        Self::from_spec(concat!(
            "H│H\n", //
            "─┼─\n", //
            "C│C\n", //
            "─┼─\n", //
            "C│C\n", //
        ))
        .unwrap()
    }

    /// Creates a [`TableDecoration`] having no headers and an inner grid.
    /// The resulting table might look like:
    /// ```text
    /// Cell1│Cell2
    /// ─────┼─────
    /// Cell3│Cell4
    /// ```
    ///
    /// # Returns
    /// A [`TableDecoration`]
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn headless_grid() -> Self {
        Self::from_spec(concat!(
            "C│C\n", //
            "─┼─\n", //
            "C│C\n", //
        ))
        .unwrap()
    }
}

impl Default for TableDecoration {
    fn default() -> Self {
        Self::boxed_grid()
    }
}

pub(crate) struct DecoratedTable<'a> {
    pub table: &'a Table,
    pub cols: usize,
    pub rows: usize,
    table_cols: usize,
    table_rows: usize,
    empty_cell: RcLayout,
    empty_column: TableColumn,
}

impl<'a> DecoratedTable<'a> {
    pub(crate) fn new(table: &'a Table) -> Self {
        let table_rows = table.cells.len();
        let table_cols = max(
            table.columns.len(),
            table.cells.iter().map(Vec::len).max().unwrap_or(0),
        );
        let deco_count = table.decoration.rows.len();
        let row_sep = table
            .decoration
            .rows
            .contains_key(&DecorationRowType::RowSep);
        let rows = if table_rows > 0 {
            if row_sep {
                2 * table_rows + deco_count - 3
            } else {
                table_rows + deco_count - 1
            }
        } else {
            deco_count - usize::from(row_sep)
        };
        let cols = if table_cols > 0 && table.decoration.has_sep {
            2 * table_cols - 1
        } else {
            table_cols
        };
        Self {
            table,
            rows,
            cols: cols
                + usize::from(table.decoration.has_left)
                + usize::from(table.decoration.has_right),
            table_cols,
            table_rows,
            empty_cell: Filler::once("").into(),
            empty_column: TableColumn::DEFAULT,
        }
    }

    #[must_use]
    pub(crate) fn cell_at(&self, row: usize, col: usize) -> Option<RcLayout> {
        let row_index = self.row_index(row)?;
        let deco = self.table.decoration.rows.get(row_index.row_type())?;

        let col_index = self.column_index(col)?;
        let cell = match col_index {
            ColumnIndex::Left => deco.left.clone(),
            ColumnIndex::Sep => deco.sep.clone(),
            ColumnIndex::Right => deco.right.clone(),
            ColumnIndex::Cell(c) => match row_index {
                RowIndex::Deco(_) => deco.fill.clone(),
                RowIndex::Cell(r) => self.table.cells.get(r).and_then(|r| r.get(c).cloned()),
                RowIndex::Header => self.table.columns.get(c).and_then(|c| c.header.clone()),
            },
        };
        Some(cell.unwrap_or_else(|| self.empty_cell.clone()))
    }

    pub(crate) fn table_column_at(&self, col: usize) -> Option<&TableColumn> {
        let col_index = self.column_index(col)?;
        match col_index {
            ColumnIndex::Cell(c) => self.table.columns.get(c),
            _ => Some(&self.empty_column),
        }
    }

    pub(crate) fn row_index(&self, row: usize) -> Option<RowIndex> {
        let mut row = row;
        if self.has_decoration_row(DecorationRowType::Top) {
            if row == 0 {
                return Some(RowIndex::Deco(DecorationRowType::Top));
            }
            row -= 1;
        }
        if self.has_decoration_row(DecorationRowType::HeaderRow) {
            if row == 0 {
                return Some(RowIndex::Header);
            }
            row -= 1;
        }
        if self.has_decoration_row(DecorationRowType::HeaderSep) {
            if row == 0 {
                return Some(RowIndex::Deco(DecorationRowType::HeaderSep));
            }
            row -= 1;
        }
        let row_spacing = 1 + usize::from(self.has_decoration_row(DecorationRowType::RowSep));

        if row.div_ceil(row_spacing) < self.table_rows {
            if row.is_multiple_of(row_spacing) {
                return Some(RowIndex::Cell(row / row_spacing));
            }
            return Some(RowIndex::Deco(DecorationRowType::RowSep));
        } else if row.div_ceil(row_spacing) == self.table_rows
            && self.has_decoration_row(DecorationRowType::Bottom)
        {
            return Some(RowIndex::Deco(DecorationRowType::Bottom));
        }
        None
    }

    fn has_decoration_row(&self, row_type: DecorationRowType) -> bool {
        self.table.decoration.rows.contains_key(&row_type)
    }

    pub(crate) fn column_index(&self, col: usize) -> Option<ColumnIndex> {
        let first_index = usize::from(self.table.decoration.has_left);
        if col < first_index {
            return Some(ColumnIndex::Left);
        }
        let col = col - first_index;
        let col_spacing = 1 + usize::from(self.table.decoration.has_sep);
        if col.div_ceil(col_spacing) < self.table_cols {
            if col.is_multiple_of(col_spacing) {
                return Some(ColumnIndex::Cell(col / col_spacing));
            }
            return Some(ColumnIndex::Sep);
        } else if col.div_ceil(col_spacing) == self.table_cols && self.table.decoration.has_right {
            return Some(ColumnIndex::Right);
        }
        None
    }
}

impl<'a> From<&'a Table> for DecoratedTable<'a> {
    fn from(table: &'a Table) -> Self {
        Self::new(table)
    }
}

pub(crate) enum RowIndex {
    Deco(DecorationRowType),
    Cell(usize),
    Header,
}

impl RowIndex {
    fn row_type(&self) -> &DecorationRowType {
        match self {
            RowIndex::Deco(row_type) => row_type,
            RowIndex::Cell(_) => &DecorationRowType::Row,
            RowIndex::Header => &DecorationRowType::HeaderRow,
        }
    }

    pub(crate) fn is_deco(&self) -> bool {
        matches!(self, RowIndex::Deco(_))
    }
}

pub(crate) enum ColumnIndex {
    Left,
    Sep,
    Right,
    Cell(usize),
}

impl ColumnIndex {
    pub(crate) fn is_deco(&self) -> bool {
        matches!(
            self,
            ColumnIndex::Left | ColumnIndex::Sep | ColumnIndex::Right
        )
    }
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub(crate) enum DecorationRowType {
    Top,
    HeaderRow,
    HeaderSep,
    Row,
    RowSep,
    Bottom,
}

impl DecorationRowType {
    fn use_fill_from_spec(self) -> bool {
        !matches!(self, DecorationRowType::HeaderRow | DecorationRowType::Row)
    }
}

#[derive(Clone)]
struct DecorationRow {
    left: Option<RcLayout>,
    fill: Option<RcLayout>,
    sep: Option<RcLayout>,
    right: Option<RcLayout>,
}

impl DecorationRow {
    fn new(
        left: Option<RcLayout>,
        fill: Option<RcLayout>,
        sep: Option<RcLayout>,
        right: Option<RcLayout>,
    ) -> Self {
        Self {
            left,
            fill,
            sep,
            right,
        }
    }
}

impl Debug for DecorationRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "DecorationRow {{left: {}, fill: {}, sep: {}, right: {}}}",
            self.left
                .as_ref()
                .map(|l| format!(
                    "{}",
                    l.layout_with_wrap_mode(10, WrapMode::default_truncate())
                ))
                .unwrap_or_default(),
            self.fill
                .as_ref()
                .map(|l| format!(
                    "{}",
                    l.layout_with_wrap_mode(10, WrapMode::default_truncate())
                ))
                .unwrap_or_default(),
            self.sep
                .as_ref()
                .map(|l| format!(
                    "{}",
                    l.layout_with_wrap_mode(10, WrapMode::default_truncate())
                ))
                .unwrap_or_default(),
            self.right
                .as_ref()
                .map(|l| format!(
                    "{}",
                    l.layout_with_wrap_mode(10, WrapMode::default_truncate())
                ))
                .unwrap_or_default(),
        ))
    }
}

#[derive(Debug)]
struct DecorationSpecParser<'a> {
    cell_marker: char,
    header_marker: char,
    lines: Vec<&'a str>,
    marker1_col: usize,
    marker2_col: usize,
    cell_row1: usize,
    cell_row2: usize,
    header_row: Option<usize>,
}

impl<'a> DecorationSpecParser<'a> {
    fn parse(
        cell_marker: char,
        header_marker: char,
        spec: &'a str,
    ) -> Result<TableDecoration, String> {
        let lines = spec.lines().collect::<Vec<&str>>();
        let mut parser = Self {
            cell_marker,
            header_marker,
            lines,
            marker1_col: 0,
            marker2_col: 0,
            cell_row1: 0,
            cell_row2: 0,
            header_row: None,
        };
        parser.collect_markers()?;

        Ok(parser.build_decoration())
    }

    fn collect_markers(&mut self) -> Result<(), String> {
        let line_len = self.lines.first().map_or(0, |l| l.display_len());

        // Check line lengths and cell markers
        let mut cell_marker_count = 0;
        for (line_index, &line) in self.lines.iter().enumerate() {
            let line_index = line_index + 1;
            if line.display_len() != line_len {
                return Err(format!(
                    "line {line_index} has different length than the first line line"
                ));
            }
            if let Some((marker1, marker2)) =
                Self::get_markers_in_line(line, self.cell_marker, line_index)?
            {
                cell_marker_count += 1;
                if cell_marker_count == 1 {
                    self.marker1_col = marker1;
                    self.marker2_col = marker2;
                    self.cell_row1 = line_index - 1;
                } else if cell_marker_count == 2 {
                    self.check_marker_position(self.cell_marker, marker1, marker2, line_index)?;
                    self.cell_row2 = line_index - 1;
                }
            }
        }
        if cell_marker_count != 2 {
            return Err(format!(
                "found {cell_marker_count} cell marker lines instead of 2"
            ));
        }

        // Check header marker
        for (line_index, &line) in self.lines.iter().enumerate() {
            let line_index = line_index + 1;
            if let Some((marker1, marker2)) =
                Self::get_markers_in_line(line, self.header_marker, line_index)?
            {
                self.check_marker_position(self.header_marker, marker1, marker2, line_index)?;
                if self.header_row.is_some() {
                    return Err(format!(
                        "found more than one header marker in line {line_index}"
                    ));
                }
                self.header_row = Some(line_index - 1);
            }
        }

        // Validate the marker rows
        if self.cell_row1 + 2 < self.cell_row2 {
            return Err("cell marker lines have a too big distance".into());
        }
        if self.cell_row2 + 2 < self.lines.len() {
            return Err("cell marker lines are far away from the end of the table".into());
        }
        if let Some(header_row) = self.header_row {
            if header_row + 2 < self.cell_row1 {
                return Err("header marker is too far away from the cell marker lines".into());
            }
            if header_row > 1 {
                return Err("header marker is too far away from the top of the table".into());
            }
        }
        Ok(())
    }

    fn check_marker_position(
        &self,
        marker: char,
        marker1_col: usize,
        marker2_col: usize,
        line_index: usize,
    ) -> Result<(), String> {
        if marker1_col != self.marker1_col {
            return Err(format!(
                "marker '{marker}' in line {line_index} and column {marker1_col} is misplaced"
            ));
        }
        if marker2_col != self.marker2_col {
            return Err(format!(
                "marker '{marker}' in line {line_index} and column {marker2_col} is misplaced"
            ));
        }
        Ok(())
    }

    /// Finds the positions of two marker characters in a line.
    ///
    /// # Returns
    /// A tuple of `(first_marker_col, second_marker_col)` if exactly two markers are found.
    fn get_markers_in_line(
        line: &str,
        marker: char,
        line_index: usize,
    ) -> Result<Option<(usize, usize)>, String> {
        let mut iter = line.chars().enumerate();
        while let Some((first, c)) = iter.next() {
            if c == marker {
                while let Some((second, c)) = iter.next() {
                    if c == marker {
                        if iter.find(|(_, c)| *c == marker).is_some() {
                            return Err(format!(
                                "found more than two '{marker}' marker in line {line_index}"
                            ));
                        }
                        return Ok(Some((first, second)));
                    }
                }
                return Err(format!(
                    "found only one '{marker}' marker in line {line_index}"
                ));
            }
        }
        Ok(None)
    }

    /// Builds the [`TableDecoration`] based on the collected markers.
    fn build_decoration(&self) -> TableDecoration {
        let mut rows = BTreeMap::new();
        if let Some(header_row) = self.header_row {
            if header_row == 1 {
                rows.insert(
                    DecorationRowType::Top,
                    self.build_decoration_row(DecorationRowType::Top, 0),
                );
            }
            rows.insert(
                DecorationRowType::HeaderRow,
                self.build_decoration_row(DecorationRowType::HeaderRow, header_row),
            );
            if header_row + 1 < self.cell_row1 {
                rows.insert(
                    DecorationRowType::HeaderSep,
                    self.build_decoration_row(DecorationRowType::HeaderSep, header_row + 1),
                );
            }
        } else if self.cell_row1 == 1 {
            rows.insert(
                DecorationRowType::Top,
                self.build_decoration_row(DecorationRowType::Top, 0),
            );
        }
        rows.insert(
            DecorationRowType::Row,
            self.build_decoration_row(DecorationRowType::Row, self.cell_row1),
        );
        if self.cell_row1 + 1 < self.cell_row2 {
            rows.insert(
                DecorationRowType::RowSep,
                self.build_decoration_row(DecorationRowType::RowSep, self.cell_row1 + 1),
            );
        }
        if self.cell_row2 + 1 < self.lines.len() {
            rows.insert(
                DecorationRowType::Bottom,
                self.build_decoration_row(DecorationRowType::Bottom, self.cell_row2 + 1),
            );
        }
        TableDecoration::new(
            rows,
            self.marker1_col > 0,
            self.marker1_col + 1 < self.marker2_col,
            self.marker2_col + 1 < self.lines[0].len(),
        )
    }

    /// Builds a [`DecorationRow`] for a specific type and line index.
    fn build_decoration_row(
        &self,
        row_type: DecorationRowType,
        line_index: usize,
    ) -> DecorationRow {
        let line = self.lines[line_index];
        let left = Some(line.display_slice(..self.marker1_col))
            .filter(|&l| !l.is_empty())
            .map(Filler::vertical)
            .map(RcLayout::from);
        let fill = Some(line.display_slice(self.marker1_col..=self.marker1_col))
            .filter(|_| row_type.use_fill_from_spec())
            .filter(|&l| !l.is_empty())
            .map(Filler::horizontal)
            .map(RcLayout::from);
        let sep = Some(line.display_slice(self.marker1_col + 1..self.marker2_col))
            .filter(|&l| !l.is_empty())
            .map(Filler::vertical)
            .map(RcLayout::from);
        let right = Some(line.display_slice(self.marker2_col + 1..))
            .filter(|&l| !l.is_empty())
            .map(Filler::vertical)
            .map(RcLayout::from);
        DecorationRow::new(left, fill, sep, right)
    }
}

#[cfg(test)]
mod tests {
    use crate::widgets::table::decoration::{
        DecoratedTable, DecorationRowType, DecorationSpecParser,
    };
    use crate::widgets::table::{Table, TableColumn};
    use crate::widgets::{CellWidth, Lines, TableDecoration};

    #[test]
    fn decoration_factory_methods() {
        assert!(
            TableDecoration::no_grid()
                .rows
                .contains_key(&DecorationRowType::HeaderRow)
        );
        assert!(
            TableDecoration::headless_no_grid()
                .rows
                .contains_key(&DecorationRowType::Row)
        );
        assert!(
            !TableDecoration::headless_no_grid()
                .rows
                .contains_key(&DecorationRowType::HeaderRow)
        );

        let boxed = TableDecoration::boxed_grid();
        assert!(boxed.rows.contains_key(&DecorationRowType::Top));
        assert!(boxed.rows.contains_key(&DecorationRowType::Bottom));
        assert!(boxed.has_left);
        assert!(boxed.has_right);
    }

    #[test]
    fn decorated_table_new_no_deco() {
        // Arrange
        let table = Table::new(
            TableDecoration::from_spec("HH\nCC\nCC").unwrap(),
            vec![
                TableColumn::default()
                    .with_header(Lines::left("Column 1"))
                    .with_width(CellWidth::Fill),
            ],
            vec![
                vec![Lines::left("Cell 1").into()],
                vec![Lines::left("Cell 3").into(), Lines::left("Cell 4").into()],
            ],
        );

        // act
        let decorated = DecoratedTable::new(&table);

        // Assert
        assert_eq!(decorated.cols, 2);
        assert_eq!(decorated.rows, 3);
        let mut text = String::new();
        for r in 0..decorated.rows {
            for c in 0..decorated.cols {
                text.push_str(
                    &format!("{}", decorated.cell_at(r, c).unwrap().layout(20)).replace('\n', " "),
                );
            }
            text.push('\n');
        }
        assert_eq!(
            text,
            concat!(
                "Column 1  \n", //
                "Cell 1  \n",   //
                "Cell 3 Cell 4 \n"
            )
        );
    }

    #[test]
    fn decorated_table_new_full_deco() {
        // Arrange
        let table = Table::new(
            TableDecoration::boxed_grid(),
            vec![
                TableColumn::default()
                    .with_header(Lines::left("Column 1"))
                    .with_width(CellWidth::Fill),
            ],
            vec![
                vec![Lines::left("Cell 1").into()],
                vec![Lines::left("Cell 3").into(), Lines::left("Cell 4").into()],
            ],
        );

        // act
        let decorated = DecoratedTable::new(&table);

        // Assert
        assert_eq!(decorated.cols, 5);
        assert_eq!(decorated.rows, 7);
        let mut text = String::new();
        for r in 0..decorated.rows {
            for c in 0..decorated.cols {
                text.push_str(
                    &format!("{}", decorated.cell_at(r, c).unwrap().layout(20)).replace('\n', " "),
                );
            }
            text.push('\n');
        }
        assert_eq!(
            text,
            concat!(
                "┌ ─ ┬ ─ ┐ \n",           //
                "│ Column 1 │  │ \n",     //
                "├ ─ ┼ ─ ┤ \n",           //
                "│ Cell 1 │  │ \n",       //
                "├ ─ ┼ ─ ┤ \n",           //
                "│ Cell 3 │ Cell 4 │ \n", //
                "└ ─ ┴ ─ ┘ \n",           //
            )
        );
    }

    #[test]
    fn decoration_spec_parser_parse_ok() {
        // Minimal
        let result = DecorationSpecParser::parse(
            'C',
            'H',
            concat!(
                "CC\n", //
                "CC"
            ),
        );
        assert_eq!(
            format!("{:?}", result.unwrap()),
            concat!(
                "TableDecoration { rows: {",
                "Row: DecorationRow {left: , fill: , sep: , right: }",
                "}, ",
                "has_left: false, has_sep: false, has_right: false }"
            )
        );

        // Maximimal
        let result = DecorationSpecParser::parse(
            '1',
            '2',
            concat!(
                "AaBCcDEe\n", //
                "Ff2Gg2Hh\n", //
                "IijKkLMm\n", //
                "Nn1Oo1Pp\n", //
                "QqRSsTUu\n", //
                "Vv1Ww1Xx\n", //
                "YyZAaBCc"
            ),
        );
        assert_eq!(
            format!("{:?}", result.unwrap()),
            concat!(
                "TableDecoration { rows: {",                                            //
                "Top: DecorationRow {left: Aa\n, fill: B\n, sep: Cc\n, right: Ee\n}, ", //
                "HeaderRow: DecorationRow {left: Ff\n, fill: , sep: Gg\n, right: Hh\n}, ", //
                "HeaderSep: DecorationRow {left: Ii\n, fill: j\n, sep: Kk\n, right: Mm\n}, ", //
                "Row: DecorationRow {left: Nn\n, fill: , sep: Oo\n, right: Pp\n}, ",    //
                "RowSep: DecorationRow {left: Qq\n, fill: R\n, sep: Ss\n, right: Uu\n}, ", //
                "Bottom: DecorationRow {left: Yy\n, fill: Z\n, sep: Aa\n, right: Cc\n}", //
                "}, ",
                "has_left: true, has_sep: true, has_right: true }"
            )
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn decoration_spec_parser_parse_fail() {
        // Different line lengths
        let result = DecorationSpecParser::parse(
            'C',
            'H',
            concat!(
                "CC\n", //
                "CC "   //
            ),
        );
        assert_eq!(
            result.unwrap_err(),
            "line 2 has different length than the first line line".to_string()
        );

        // Less than one marker
        let result = DecorationSpecParser::parse('C', 'H', "C");
        assert_eq!(
            result.unwrap_err(),
            "found only one 'C' marker in line 1".to_string()
        );

        // More than two marker
        let result = DecorationSpecParser::parse('C', 'H', "CCC");
        assert_eq!(
            result.unwrap_err(),
            "found more than two 'C' marker in line 1".to_string()
        );

        // Less than two cell marker lines
        let result = DecorationSpecParser::parse('C', 'H', "CC");
        assert_eq!(
            result.unwrap_err(),
            "found 1 cell marker lines instead of 2".to_string()
        );

        // More than two cell marker lines
        let result = DecorationSpecParser::parse(
            'C',
            'H',
            concat!(
                "CC\n", //
                "CC\n", //
                "CC",   //
            ),
        );
        assert_eq!(
            result.unwrap_err(),
            "found 3 cell marker lines instead of 2".to_string()
        );

        // Markers at different positions
        let result = DecorationSpecParser::parse(
            'C',
            'H',
            concat!(
                "CCX\n", //
                "CXC",   //
            ),
        );
        assert_eq!(
            result.unwrap_err(),
            "marker 'C' in line 2 and column 2 is misplaced".to_string()
        );

        // More than one header marker
        let result = DecorationSpecParser::parse(
            'C',
            'H',
            concat!(
                "HH\n", //
                "HH\n", //
                "CC\n", //
                "CC",   //
            ),
        );
        assert_eq!(
            result.unwrap_err(),
            "found more than one header marker in line 2".to_string()
        );

        // Last marker line too big distance to bottom
        let result = DecorationSpecParser::parse(
            'C',
            'H',
            concat!(
                "HH\n", //
                "CC\n", //
                "CC\n", //
                "--\n", //
                "--",   //
            ),
        );
        assert_eq!(
            result.unwrap_err(),
            "cell marker lines are far away from the end of the table".to_string()
        );

        // Distance between cell marker lines too big
        let result = DecorationSpecParser::parse(
            'C',
            'H',
            concat!(
                "HH\n", //
                "CC\n", //
                "--\n", //
                "--\n", //
                "CC",   //
            ),
        );
        assert_eq!(
            result.unwrap_err(),
            "cell marker lines have a too big distance".to_string()
        );

        // Distance between header & cell marker lines too big
        let result = DecorationSpecParser::parse(
            'C',
            'H',
            concat!(
                "HH\n", //
                "--\n", //
                "--\n", //
                "CC\n", //
                "CC",   //
            ),
        );
        assert_eq!(
            result.unwrap_err(),
            "header marker is too far away from the cell marker lines".to_string()
        );

        // Distance to header too big
        let result = DecorationSpecParser::parse(
            'C',
            'H',
            concat!(
                "--\n", //
                "--\n", //
                "HH\n", //
                "CC\n", //
                "CC",   //
            ),
        );
        assert_eq!(
            result.unwrap_err(),
            "header marker is too far away from the top of the table".to_string()
        );
    }
}
