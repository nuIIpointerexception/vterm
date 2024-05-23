use std::{
    cmp::{max, min, Ordering},
    mem,
};

use crate::{
    grid::{row::Row, Dimensions, Grid, GridCell},
    index::{Boundary, Column, Line},
    term::cell::{Flags, ResetDiscriminant},
};

impl<T: GridCell + Default + PartialEq + Clone> Grid<T> {
    pub fn resize<D>(&mut self, reflow: bool, lines: usize, columns: usize)
        where
            T: ResetDiscriminant<D>,
            D: PartialEq,
    {
        let template = mem::take(&mut self.cursor.template);

        match self.lines.cmp(&lines) {
            Ordering::Less => self.grow_lines(lines),
            Ordering::Greater => self.shrink_lines(lines),
            Ordering::Equal => (),
        }

        match self.columns.cmp(&columns) {
            Ordering::Less => self.grow_columns(reflow, columns),
            Ordering::Greater => self.shrink_columns(reflow, columns),
            Ordering::Equal => (),
        }

        self.cursor.template = template;
    }

    fn grow_lines<D>(&mut self, target: usize)
        where
            T: ResetDiscriminant<D>,
            D: PartialEq,
    {
        let lines_added = target - self.lines;

        self.raw.grow_visible_lines(target);
        self.lines = target;

        let history_size = self.history_size();
        let from_history = min(history_size, lines_added);

        if from_history != lines_added {
            let delta = lines_added - from_history;
            self.scroll_up(&(Line(0)..Line(target as i32)), delta);
        }

        self.saved_cursor.point.line += from_history;
        self.cursor.point.line += from_history;

        self.display_offset = self.display_offset.saturating_sub(lines_added);
        self.decrease_scroll_limit(lines_added);
    }

    fn shrink_lines<D>(&mut self, target: usize)
        where
            T: ResetDiscriminant<D>,
            D: PartialEq,
    {
        let required_scrolling =
            (self.cursor.point.line.0 as usize + 1).saturating_sub(target);
        if required_scrolling > 0 {
            self.scroll_up(
                &(Line(0)..Line(self.lines as i32)),
                required_scrolling,
            );

            self.cursor.point.line =
                min(self.cursor.point.line, Line(target as i32 - 1));
        }

        self.saved_cursor.point.line =
            min(self.saved_cursor.point.line, Line(target as i32 - 1));

        self.raw.rotate((self.lines - target) as isize);
        self.raw.shrink_visible_lines(target);
        self.lines = target;
    }

    fn grow_columns(&mut self, reflow: bool, columns: usize) {
        let should_reflow = |row: &Row<T>| -> bool {
            let len = Column(row.len());
            reflow
                && len.0 > 0
                && len < columns
                && row[len - 1].flags().contains(Flags::WRAPLINE)
        };

        self.columns = columns;

        let mut reversed: Vec<Row<T>> = Vec::with_capacity(self.raw.len());
        let mut cursor_line_delta = 0;

        if self.cursor.input_needs_wrap && reflow {
            self.cursor.input_needs_wrap = false;
            self.cursor.point.column += 1;
        }

        let mut rows = self.raw.take_all();

        for (i, mut row) in rows.drain(..).enumerate().rev() {
            let last_row = match reversed.last_mut() {
                Some(last_row) if should_reflow(last_row) => last_row,
                _ => {
                    reversed.push(row);
                    continue;
                }
            };

            if let Some(cell) = last_row.last_mut() {
                cell.flags_mut().remove(Flags::WRAPLINE);
            }

            let mut last_len = last_row.len();
            if last_len >= 1
                && last_row[Column(last_len - 1)]
                .flags()
                .contains(Flags::LEADING_WIDE_CHAR_SPACER)
            {
                last_row.shrink(last_len - 1);
                last_len -= 1;
            }

            let mut num_wrapped = columns - last_len;
            let len = min(row.len(), num_wrapped);

            let mut cells =
                if row[Column(len - 1)].flags().contains(Flags::WIDE_CHAR) {
                    num_wrapped -= 1;

                    let mut cells = row.front_split_off(len - 1);

                    let mut spacer = T::default();
                    spacer.flags_mut().insert(Flags::LEADING_WIDE_CHAR_SPACER);
                    cells.push(spacer);

                    cells
                } else {
                    row.front_split_off(len)
                };

            last_row.append(&mut cells);

            let cursor_buffer_line =
                self.lines - self.cursor.point.line.0 as usize - 1;

            if i == cursor_buffer_line && reflow {
                let mut target =
                    self.cursor.point.sub(self, Boundary::Cursor, num_wrapped);

                if target.column.0 == 0 && row.is_clear() {
                    self.cursor.input_needs_wrap = true;
                    target = target.sub(self, Boundary::Cursor, 1);
                }
                self.cursor.point.column = target.column;

                let line_delta = self.cursor.point.line - target.line;

                if line_delta != 0 && row.is_clear() {
                    continue;
                }

                cursor_line_delta += line_delta.0 as usize;
            } else if row.is_clear() {
                if i < self.display_offset {
                    self.display_offset = self.display_offset.saturating_sub(1);
                }

                if i < cursor_buffer_line {
                    self.cursor.point.line += 1;
                }

                continue;
            }

            if let Some(cell) = last_row.last_mut() {
                cell.flags_mut().insert(Flags::WRAPLINE);
            }

            reversed.push(row);
        }

        if reversed.len() < self.lines {
            let delta = (self.lines - reversed.len()) as i32;
            self.cursor.point.line =
                max(self.cursor.point.line - delta, Line(0));
            reversed.resize_with(self.lines, || Row::new(columns));
        }

        if cursor_line_delta != 0 {
            let cursor_buffer_line =
                self.lines - self.cursor.point.line.0 as usize - 1;
            let available =
                min(cursor_buffer_line, reversed.len() - self.lines);
            let overflow = cursor_line_delta.saturating_sub(available);
            reversed.truncate(reversed.len() + overflow - cursor_line_delta);
            self.cursor.point.line =
                max(self.cursor.point.line - overflow, Line(0));
        }

        let mut new_raw = Vec::with_capacity(reversed.len());
        for mut row in reversed.drain(..).rev() {
            if row.len() < columns {
                row.grow(columns);
            }
            new_raw.push(row);
        }

        self.raw.replace_inner(new_raw);

        self.display_offset = min(self.display_offset, self.history_size());
    }

    fn shrink_columns(&mut self, reflow: bool, columns: usize) {
        self.columns = columns;

        if self.cursor.input_needs_wrap && reflow {
            self.cursor.input_needs_wrap = false;
            self.cursor.point.column += 1;
        }

        let mut new_raw = Vec::with_capacity(self.raw.len());
        let mut buffered: Option<Vec<T>> = None;

        let mut rows = self.raw.take_all();
        for (i, mut row) in rows.drain(..).enumerate().rev() {
            if let Some(buffered) = buffered.take() {
                let cursor_buffer_line =
                    self.lines - self.cursor.point.line.0 as usize - 1;
                if i == cursor_buffer_line {
                    self.cursor.point.column += buffered.len();
                }

                row.append_front(buffered);
            }

            loop {
                let mut wrapped = match row.shrink(columns) {
                    Some(wrapped) if reflow => wrapped,
                    _ => {
                        let cursor_buffer_line =
                            self.lines - self.cursor.point.line.0 as usize - 1;
                        if reflow
                            && i == cursor_buffer_line
                            && self.cursor.point.column > columns
                        {
                            Vec::new()
                        } else {
                            new_raw.push(row);
                            break;
                        }
                    }
                };

                if row.len() >= columns
                    && row[Column(columns - 1)]
                    .flags()
                    .contains(Flags::WIDE_CHAR)
                {
                    let mut spacer = T::default();
                    spacer.flags_mut().insert(Flags::LEADING_WIDE_CHAR_SPACER);

                    let wide_char =
                        mem::replace(&mut row[Column(columns - 1)], spacer);
                    wrapped.insert(0, wide_char);
                }

                let len = wrapped.len();
                if len > 0
                    && wrapped[len - 1]
                    .flags()
                    .contains(Flags::LEADING_WIDE_CHAR_SPACER)
                {
                    if len == 1 {
                        row[Column(columns - 1)]
                            .flags_mut()
                            .insert(Flags::WRAPLINE);
                        new_raw.push(row);
                        break;
                    } else {
                        wrapped[len - 2].flags_mut().insert(Flags::WRAPLINE);
                        wrapped.truncate(len - 1);
                    }
                }

                new_raw.push(row);

                if let Some(cell) =
                    new_raw.last_mut().and_then(|r| r.last_mut())
                {
                    cell.flags_mut().insert(Flags::WRAPLINE);
                }

                if wrapped
                    .last()
                    .map(|c| c.flags().contains(Flags::WRAPLINE) && i >= 1)
                    .unwrap_or(false)
                    && wrapped.len() < columns
                {
                    if let Some(cell) = wrapped.last_mut() {
                        cell.flags_mut().remove(Flags::WRAPLINE);
                    }

                    buffered = Some(wrapped);
                    break;
                } else {
                    let cursor_buffer_line =
                        self.lines - self.cursor.point.line.0 as usize - 1;
                    if (i == cursor_buffer_line
                        && self.cursor.point.column < columns)
                        || i < cursor_buffer_line
                    {
                        self.cursor.point.line =
                            max(self.cursor.point.line - 1, Line(0));
                    }

                    if i == cursor_buffer_line
                        && self.cursor.point.column >= columns
                    {
                        self.cursor.point.column -= columns;
                    }

                    let occ = wrapped.len();
                    if occ < columns {
                        wrapped.resize_with(columns, T::default);
                    }
                    row = Row::from_vec(wrapped, occ);

                    if i < self.display_offset {
                        self.display_offset += 1;
                    }
                }
            }
        }

        let mut reversed: Vec<Row<T>> = new_raw.drain(..).rev().collect();
        reversed.truncate(self.max_scroll_limit + self.lines);
        self.raw.replace_inner(reversed);

        self.display_offset = min(self.display_offset, self.history_size());

        if !reflow {
            self.cursor.point.column =
                min(self.cursor.point.column, Column(columns - 1));
        } else if self.cursor.point.column == columns
            && !self[self.cursor.point.line][Column(columns - 1)]
            .flags()
            .contains(Flags::WRAPLINE)
        {
            self.cursor.input_needs_wrap = true;
            self.cursor.point.column -= 1;
        } else {
            self.cursor.point =
                self.cursor.point.grid_clamp(self, Boundary::Cursor);
        }

        self.saved_cursor.point.column =
            min(self.saved_cursor.point.column, Column(columns - 1));
    }
}
