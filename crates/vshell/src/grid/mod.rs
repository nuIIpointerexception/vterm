use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

use self::{
    cell::{Cell, Style},
    row::Row,
};

pub mod cell;
pub mod row;

#[derive(Debug)]
pub struct Grid {
    rows: Vec<Row>,
    scrollback: Vec<Row>,
    index: usize,
    columns: usize,
}

impl Grid {
    pub fn new(columns: usize, lines: usize) -> Self {
        let mut rows = Vec::with_capacity(lines);
        rows.resize(lines, Row::new(columns));

        Self {
            rows,
            index: 0,
            scrollback: vec![],
            columns,
        }
    }

    /// Scrolls the grid up by one
    pub fn scroll_up(&mut self) {
        let len = self.rows.len();
        for i in 1..len {
            self.rows.swap(i - 1, i);
        }
        self.scrollback.push(self.rows[len - 1].clone());
        self.rows[len - 1].reset();
    }

    /// Scrolls the grid down by one, taking the last row from the scrollback
    pub fn scroll_down(&mut self) {
        let len = self.rows.len();
        for i in (1..len).rev() {
            self.rows.swap(i - 1, i);
        }
        if !self.scrollback.is_empty() {
            self.rows[0] = self.scrollback.pop().unwrap();
        }
    }

    /// Returns the different style sections to render.
    /// Note: this thing allocates too much, make it so that it returns ranges
    /// instead and stop allocating things in a tight renderer loop.
    pub fn sections(&self) -> Vec<TextSection> {
        let mut res = vec![];

        let mut current_style = self.rows[0][0].style;
        let mut text = String::new();

        for row in &self.rows {
            for col in &row.inner {
                if col.style != current_style {
                    res.push(TextSection {
                        text: text.clone(),
                        style: current_style,
                    });
                    text = "".to_string();
                    current_style = col.style;
                }
                if let Some(c) = col.c {
                    text.push_str(&String::from(c));
                } else {
                    text.push(' ');
                }
            }
            text.push('\n');
        }

        if !text.is_empty() {
            let ts = TextSection {
                text: text.clone(),
                style: current_style,
            };
            res.push(ts);
        }

        res
    }

    pub fn resize(&mut self, new_columns: usize, new_lines: usize) {
        let mut new_rows: Vec<Row> = Vec::new();
        let mut current_row = Row::new(new_columns);
        let mut current_column_index = 0;

        // Flatten all cells from existing rows into a single vector
        let all_cells: Vec<Cell> =
            self.rows.iter().flat_map(|r| r.inner.clone()).collect();

        let mut advance = false;
        // Wrap cells into new rows based on the new column width
        for cell in all_cells {
            if advance && cell.c.is_none() {
                continue;
            } else {
                advance = false;
            }

            // If we arrived at the end of the row
            if current_column_index == new_columns {
                new_rows.push(current_row);
                current_row = Row::new(new_columns);
                current_column_index = 0;
            }

            if cell.c.is_some() {
                current_row[current_column_index] = cell;
                current_column_index += 1;
            } else {
                new_rows.push(current_row);
                current_row = Row::new(new_columns);
                current_column_index = 0;
                advance = true;
            }

            // Break out of the loop early if we've filled up the new_lines
            if new_rows.len() == new_lines {
                break;
            }
        }

        // Add the last row if it's not empty and we haven't exceeded new_lines
        if !current_row.inner.is_empty() && new_rows.len() < new_lines {
            new_rows.push(current_row);
        }

        // If the new size has more lines than the current content, add empty rows
        while new_rows.len() < new_lines {
            new_rows.push(Row::new(new_columns));
        }

        // Update grid rows and columns
        self.rows = new_rows;
        self.columns = new_columns; // Update the column count
    }

    fn print_vec(
        &self,
        v: &[Row],
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        for _ in 0..self.columns {
            write!(f, "_")?;
        }

        for row in v {
            write!(f, "|")?;
            for cell in &row.inner {
                if let Some(c) = cell.c {
                    if c == '\t' {
                        write!(f, " ")?;
                    } else {
                        write!(f, "{}", c)?;
                    }
                }
            }
            writeln!(f, "|")?;
        }

        for _ in 0..self.columns {
            write!(f, "-")?;
        }

        Ok(())
    }
}

pub struct TextSection {
    pub text: String,
    pub style: Style,
}

impl Index<usize> for Grid {
    type Output = Row;

    fn index(&self, index: usize) -> &Self::Output {
        &self.rows[index]
    }
}

impl IndexMut<usize> for Grid {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.rows[index]
    }
}

impl Iterator for Grid {
    type Item = Row;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.rows.len() {
            let row = &self.rows[self.index];
            self.index += 1;
            Some(row.clone())
        } else {
            None
        }
    }
}

impl Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\n\n#################################\n\n")?;
        self.print_vec(&self.scrollback, f)?;
        writeln!(f, "-------------------------------------")?;
        self.print_vec(&self.rows, f)?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_scroll_up() {
        let mut g = Grid::new(2, 2);
        g[1][0].c = Some('a');
        assert!(g[0][0].c == None);
        g.scroll_up();
        assert!(g[0][0].c == Some('a'));
    }

    #[test]
    fn test_resize() {
        let mut g = Grid::new(2, 2);
        g[0][0].c = Some('a');
        g[1][0].c = Some('b');

        g.resize(3, 2);

        assert!(g[0][0].c == Some('a'));
        assert!(g[1][0].c == Some('b'));
    }

    #[test]
    fn test_resize_with_empty() {
        let mut g = Grid::new(2, 2);
        g[0][0].c = Some('a');
        g[1][0].c = Some(' ');
        g[1][1].c = Some('a');

        println!("{}", g);
        g.resize(3, 3);
        println!("{}", g);

        assert!(g[1][1].c == Some('a'));
    }
}
