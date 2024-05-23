use std::{
    cmp::{max, min},
    ops::{Bound, Deref, Index, IndexMut, Range, RangeBounds},
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    index::{Column, Line, Point},
    term::cell::{Flags, ResetDiscriminant},
    vte::ansi::{CharsetIndex, StandardCharset},
};

pub mod resize;
mod row;
mod storage;
#[cfg(test)]
mod tests;

pub use self::row::Row;
use self::storage::Storage;

pub trait GridCell: Sized {
    fn is_empty(&self) -> bool;

    fn reset(&mut self, template: &Self);

    fn flags(&self) -> &Flags;
    fn flags_mut(&mut self) -> &mut Flags;
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Cursor<T> {
    pub point: Point,

    pub template: T,

    pub charsets: Charsets,

    pub input_needs_wrap: bool,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct Charsets([StandardCharset; 4]);

impl Index<CharsetIndex> for Charsets {
    type Output = StandardCharset;

    fn index(&self, index: CharsetIndex) -> &StandardCharset {
        &self.0[index as usize]
    }
}

impl IndexMut<CharsetIndex> for Charsets {
    fn index_mut(&mut self, index: CharsetIndex) -> &mut StandardCharset {
        &mut self.0[index as usize]
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Scroll {
    Delta(i32),
    PageUp,
    PageDown,
    Top,
    Bottom,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Grid<T> {
    #[cfg_attr(feature = "serde", serde(skip))]
    pub cursor: Cursor<T>,

    #[cfg_attr(feature = "serde", serde(skip))]
    pub saved_cursor: Cursor<T>,

    raw: Storage<T>,

    columns: usize,

    lines: usize,

    display_offset: usize,

    max_scroll_limit: usize,
}

impl<T: GridCell + Default + PartialEq + Clone> Grid<T> {
    pub fn new(
        lines: usize,
        columns: usize,
        max_scroll_limit: usize,
    ) -> Grid<T> {
        Grid {
            raw: Storage::with_capacity(lines, columns),
            max_scroll_limit,
            display_offset: 0,
            saved_cursor: Cursor::default(),
            cursor: Cursor::default(),
            lines,
            columns,
        }
    }

    pub fn update_history(&mut self, history_size: usize) {
        let current_history_size = self.history_size();
        if current_history_size > history_size {
            self.raw.shrink_lines(current_history_size - history_size);
        }
        self.display_offset = min(self.display_offset, history_size);
        self.max_scroll_limit = history_size;
    }

    pub fn scroll_display(&mut self, scroll: Scroll) {
        self.display_offset = match scroll {
            Scroll::Delta(count) => min(
                max((self.display_offset as i32) + count, 0) as usize,
                self.history_size(),
            ),
            Scroll::PageUp => {
                min(self.display_offset + self.lines, self.history_size())
            }
            Scroll::PageDown => self.display_offset.saturating_sub(self.lines),
            Scroll::Top => self.history_size(),
            Scroll::Bottom => 0,
        };
    }

    fn increase_scroll_limit(&mut self, count: usize) {
        let count = min(count, self.max_scroll_limit - self.history_size());
        if count != 0 {
            self.raw.initialize(count, self.columns);
        }
    }

    fn decrease_scroll_limit(&mut self, count: usize) {
        let count = min(count, self.history_size());
        if count != 0 {
            self.raw.shrink_lines(min(count, self.history_size()));
            self.display_offset = min(self.display_offset, self.history_size());
        }
    }

    #[inline]
    pub fn scroll_down<D>(&mut self, region: &Range<Line>, positions: usize)
        where
            T: ResetDiscriminant<D>,
            D: PartialEq,
    {
        if region.end - region.start <= positions {
            for i in (region.start.0..region.end.0).map(Line::from) {
                self.raw[i].reset(&self.cursor.template);
            }

            return;
        }

        if self.max_scroll_limit == 0 {
            let screen_lines = self.screen_lines() as i32;
            for i in (region.end.0..screen_lines).map(Line::from) {
                self.raw.swap(i, i - positions as i32);
            }

            self.raw.rotate_down(positions);

            for i in (0..positions).map(Line::from) {
                self.raw[i].reset(&self.cursor.template);
            }

            for i in (0..region.start.0).map(Line::from) {
                self.raw.swap(i, i + positions);
            }
        } else {
            let range = (region.start + positions).0..region.end.0;
            for line in range.rev().map(Line::from) {
                self.raw.swap(line, line - positions);
            }

            let range = region.start.0..(region.start + positions).0;
            for line in range.rev().map(Line::from) {
                self.raw[line].reset(&self.cursor.template);
            }
        }
    }

    pub fn scroll_up<D>(&mut self, region: &Range<Line>, positions: usize)
        where
            T: ResetDiscriminant<D>,
            D: PartialEq,
    {
        if region.end - region.start <= positions && region.start != 0 {
            for i in (region.start.0..region.end.0).map(Line::from) {
                self.raw[i].reset(&self.cursor.template);
            }

            return;
        }

        if self.display_offset != 0 {
            self.display_offset =
                min(self.display_offset + positions, self.max_scroll_limit);
        }

        if region.start == 0 {
            self.increase_scroll_limit(positions);

            for i in (0..region.start.0).rev().map(Line::from) {
                self.raw.swap(i, i + positions);
            }

            self.raw.rotate(-(positions as isize));

            let screen_lines = self.screen_lines() as i32;
            for i in (region.end.0..screen_lines).rev().map(Line::from) {
                self.raw.swap(i, i - positions);
            }
        } else {
            for i in (region.start.0..region.end.0 - positions as i32)
                .map(Line::from)
            {
                self.raw.swap(i, i + positions);
            }
        }

        for i in (region.end.0 - positions as i32..region.end.0).map(Line::from)
        {
            self.raw[i].reset(&self.cursor.template);
        }
    }

    pub fn clear_viewport<D>(&mut self)
        where
            T: ResetDiscriminant<D>,
            D: PartialEq,
    {
        let end =
            Point::new(Line(self.lines as i32 - 1), Column(self.columns()));
        let mut iter = self.iter_from(end);
        while let Some(cell) = iter.prev() {
            if !cell.is_empty() || cell.point.line < 0 {
                break;
            }
        }
        debug_assert!(iter.point.line >= -1);
        let positions = (iter.point.line.0 + 1) as usize;
        let region = Line(0)..Line(self.lines as i32);

        self.scroll_up(&region, positions);

        for line in (0..(self.lines - positions)).map(Line::from) {
            self.raw[line].reset(&self.cursor.template);
        }
    }

    pub fn reset<D>(&mut self)
        where
            T: ResetDiscriminant<D>,
            D: PartialEq,
    {
        self.clear_history();

        self.saved_cursor = Cursor::default();
        self.cursor = Cursor::default();
        self.display_offset = 0;

        let range = self.topmost_line().0..(self.screen_lines() as i32);
        for line in range.map(Line::from) {
            self.raw[line].reset(&self.cursor.template);
        }
    }
}

impl<T> Grid<T> {
    pub fn reset_region<D, R: RangeBounds<Line>>(&mut self, bounds: R)
        where
            T: ResetDiscriminant<D> + GridCell + Clone + Default,
            D: PartialEq,
    {
        let start = match bounds.start_bound() {
            Bound::Included(line) => *line,
            Bound::Excluded(line) => *line + 1,
            Bound::Unbounded => Line(0),
        };

        let end = match bounds.end_bound() {
            Bound::Included(line) => *line + 1,
            Bound::Excluded(line) => *line,
            Bound::Unbounded => Line(self.screen_lines() as i32),
        };

        debug_assert!(start < self.screen_lines() as i32);
        debug_assert!(end <= self.screen_lines() as i32);

        for line in (start.0..end.0).map(Line::from) {
            self.raw[line].reset(&self.cursor.template);
        }
    }

    #[inline]
    pub fn clear_history(&mut self) {
        self.raw.shrink_lines(self.history_size());

        self.display_offset = 0;
    }

    #[inline]
    pub fn initialize_all(&mut self)
        where
            T: GridCell + Clone + Default,
    {
        self.truncate();

        self.raw.initialize(
            self.max_scroll_limit - self.history_size(),
            self.columns,
        );
    }

    #[inline]
    pub fn truncate(&mut self) {
        self.raw.truncate();
    }

    #[inline]
    pub fn iter_from(&self, point: Point) -> GridIterator<'_, T> {
        let end = Point::new(self.bottommost_line(), self.last_column());
        GridIterator {
            grid: self,
            point,
            end,
        }
    }

    #[inline]
    pub fn display_iter(&self) -> GridIterator<'_, T> {
        let last_column = self.last_column();
        let start =
            Point::new(Line(-(self.display_offset() as i32) - 1), last_column);
        let end_line =
            min(start.line + self.screen_lines(), self.bottommost_line());
        let end = Point::new(end_line, last_column);

        GridIterator {
            grid: self,
            point: start,
            end,
        }
    }

    #[inline]
    pub fn display_offset(&self) -> usize {
        self.display_offset
    }

    #[inline]
    pub fn cursor_cell(&mut self) -> &mut T {
        let point = self.cursor.point;
        &mut self[point.line][point.column]
    }
}

impl<T: PartialEq> PartialEq for Grid<T> {
    fn eq(&self, other: &Self) -> bool {
        self.raw.eq(&other.raw)
            && self.columns.eq(&other.columns)
            && self.lines.eq(&other.lines)
            && self.display_offset.eq(&other.display_offset)
    }
}

impl<T> Index<Line> for Grid<T> {
    type Output = Row<T>;

    #[inline]
    fn index(&self, index: Line) -> &Row<T> {
        &self.raw[index]
    }
}

impl<T> IndexMut<Line> for Grid<T> {
    #[inline]
    fn index_mut(&mut self, index: Line) -> &mut Row<T> {
        &mut self.raw[index]
    }
}

impl<T> Index<Point> for Grid<T> {
    type Output = T;

    #[inline]
    fn index(&self, point: Point) -> &T {
        &self[point.line][point.column]
    }
}

impl<T> IndexMut<Point> for Grid<T> {
    #[inline]
    fn index_mut(&mut self, point: Point) -> &mut T {
        &mut self[point.line][point.column]
    }
}

pub trait Dimensions {
    fn total_lines(&self) -> usize;

    fn screen_lines(&self) -> usize;

    fn columns(&self) -> usize;

    #[inline]
    fn last_column(&self) -> Column {
        Column(self.columns() - 1)
    }

    #[inline]
    fn topmost_line(&self) -> Line {
        Line(-(self.history_size() as i32))
    }

    #[inline]
    fn bottommost_line(&self) -> Line {
        Line(self.screen_lines() as i32 - 1)
    }

    #[inline]
    fn history_size(&self) -> usize {
        self.total_lines().saturating_sub(self.screen_lines())
    }
}

impl<G> Dimensions for Grid<G> {
    #[inline]
    fn total_lines(&self) -> usize {
        self.raw.len()
    }

    #[inline]
    fn screen_lines(&self) -> usize {
        self.lines
    }

    #[inline]
    fn columns(&self) -> usize {
        self.columns
    }
}

#[cfg(test)]
impl Dimensions for (usize, usize) {
    fn total_lines(&self) -> usize {
        self.0
    }

    fn screen_lines(&self) -> usize {
        self.0
    }

    fn columns(&self) -> usize {
        self.1
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Indexed<T> {
    pub point: Point,
    pub cell: T,
}

impl<T> Deref for Indexed<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        &self.cell
    }
}

pub struct GridIterator<'a, T> {
    grid: &'a Grid<T>,

    point: Point,

    end: Point,
}

impl<'a, T> GridIterator<'a, T> {
    pub fn point(&self) -> Point {
        self.point
    }

    pub fn cell(&self) -> &'a T {
        &self.grid[self.point]
    }
}

impl<'a, T> Iterator for GridIterator<'a, T> {
    type Item = Indexed<&'a T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.point >= self.end {
            return None;
        }

        match self.point {
            Point { column, .. } if column == self.grid.last_column() => {
                self.point.column = Column(0);
                self.point.line += 1;
            }
            _ => self.point.column += Column(1),
        }

        Some(Indexed {
            cell: &self.grid[self.point],
            point: self.point,
        })
    }
}

pub trait BidirectionalIterator: Iterator {
    fn prev(&mut self) -> Option<Self::Item>;
}

impl<'a, T> BidirectionalIterator for GridIterator<'a, T> {
    fn prev(&mut self) -> Option<Self::Item> {
        let topmost_line = self.grid.topmost_line();
        let last_column = self.grid.last_column();

        if self.point == Point::new(topmost_line, Column(0)) {
            return None;
        }

        match self.point {
            Point {
                column: Column(0), ..
            } => {
                self.point.column = last_column;
                self.point.line -= 1;
            }
            _ => self.point.column -= Column(1),
        }

        Some(Indexed {
            cell: &self.grid[self.point],
            point: self.point,
        })
    }
}
