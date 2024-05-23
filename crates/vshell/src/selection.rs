use std::{
    cmp::min,
    mem,
    ops::{Bound, Range, RangeBounds},
};

use crate::{
    grid::{Dimensions, GridCell, Indexed},
    index::{Boundary, Column, Line, Point, Side},
    term::{
        cell::{Cell, Flags},
        Term,
    },
    vte::ansi::CursorShape,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Anchor {
    point: Point,
    side: Side,
}

impl Anchor {
    fn new(point: Point, side: Side) -> Anchor {
        Anchor { point, side }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct SelectionRange {
    pub start: Point,
    pub end: Point,
    pub is_block: bool,
}

impl SelectionRange {
    pub fn new(start: Point, end: Point, is_block: bool) -> Self {
        assert!(start <= end);
        Self {
            start,
            end,
            is_block,
        }
    }
}

impl SelectionRange {
    pub fn contains(&self, point: Point) -> bool {
        self.start.line <= point.line
            && self.end.line >= point.line
            && (self.start.column <= point.column
            || (self.start.line != point.line && !self.is_block))
            && (self.end.column >= point.column
            || (self.end.line != point.line && !self.is_block))
    }

    pub fn contains_cell(
        &self,
        indexed: &Indexed<&Cell>,
        point: Point,
        shape: CursorShape,
    ) -> bool {
        if shape == CursorShape::Block
            && point == indexed.point
            && (self.start == indexed.point
            || self.end == indexed.point
            || (self.is_block
            && ((self.start.line == indexed.point.line
            && self.end.column == indexed.point.column)
            || (self.end.line == indexed.point.line
            && self.start.column == indexed.point.column))))
        {
            return false;
        }

        if self.contains(indexed.point) {
            return true;
        }

        indexed.cell.flags().contains(Flags::WIDE_CHAR)
            && self.contains(Point::new(
            indexed.point.line,
            indexed.point.column + 1,
        ))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SelectionType {
    Simple,
    Block,
    Semantic,
    Lines,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    pub ty: SelectionType,
    region: Range<Anchor>,
}

impl Selection {
    pub fn new(ty: SelectionType, location: Point, side: Side) -> Selection {
        Self {
            region: Range {
                start: Anchor::new(location, side),
                end: Anchor::new(location, side),
            },
            ty,
        }
    }

    pub fn update(&mut self, point: Point, side: Side) {
        self.region.end = Anchor::new(point, side);
    }

    pub fn rotate<D: Dimensions>(
        mut self,
        dimensions: &D,
        range: &Range<Line>,
        delta: i32,
    ) -> Option<Selection> {
        let bottommost_line = dimensions.bottommost_line();
        let range_bottom = range.end;
        let range_top = range.start;

        let (mut start, mut end) =
            (&mut self.region.start, &mut self.region.end);
        if start.point > end.point {
            mem::swap(&mut start, &mut end);
        }

        if (start.point.line >= range_top || range_top == 0)
            && start.point.line < range_bottom
        {
            start.point.line = min(start.point.line - delta, bottommost_line);

            if start.point.line >= range_bottom && end.point.line < range_bottom
            {
                return None;
            }

            if start.point.line < range_top && range_top != 0 {
                if self.ty != SelectionType::Block {
                    start.point.column = Column(0);
                    start.side = Side::Left;
                }
                start.point.line = range_top;
            }
        }

        if (end.point.line >= range_top || range_top == 0)
            && end.point.line < range_bottom
        {
            end.point.line = min(end.point.line - delta, bottommost_line);

            if end.point.line < start.point.line {
                return None;
            }

            if end.point.line >= range_bottom {
                if self.ty != SelectionType::Block {
                    end.point.column = dimensions.last_column();
                    end.side = Side::Right;
                }
                end.point.line = range_bottom - 1;
            }
        }

        Some(self)
    }

    pub fn is_empty(&self) -> bool {
        match self.ty {
            SelectionType::Simple => {
                let (mut start, mut end) = (self.region.start, self.region.end);
                if start.point > end.point {
                    mem::swap(&mut start, &mut end);
                }

                start == end
                    || (start.side == Side::Right
                    && end.side == Side::Left
                    && (start.point.line == end.point.line)
                    && start.point.column + 1 == end.point.column)
            }
            SelectionType::Block => {
                let (start, end) = (self.region.start, self.region.end);

                (start.point.column == end.point.column
                    && start.side == end.side)
                    || (start.point.column + 1 == end.point.column
                    && start.side == Side::Right
                    && end.side == Side::Left)
                    || (end.point.column + 1 == start.point.column
                    && start.side == Side::Left
                    && end.side == Side::Right)
            }
            SelectionType::Semantic | SelectionType::Lines => false,
        }
    }

    pub fn intersects_range<R: RangeBounds<Line>>(&self, range: R) -> bool {
        let mut start = self.region.start.point.line;
        let mut end = self.region.end.point.line;

        if start > end {
            mem::swap(&mut start, &mut end);
        }

        let range_top = match range.start_bound() {
            Bound::Included(&range_start) => range_start,
            Bound::Excluded(&range_start) => range_start + 1,
            Bound::Unbounded => Line(i32::MIN),
        };

        let range_bottom = match range.end_bound() {
            Bound::Included(&range_end) => range_end,
            Bound::Excluded(&range_end) => range_end - 1,
            Bound::Unbounded => Line(i32::MAX),
        };

        range_bottom >= start && range_top <= end
    }

    pub fn include_all(&mut self) {
        let (start, end) = (self.region.start.point, self.region.end.point);
        let (start_side, end_side) = match self.ty {
            SelectionType::Block
            if start.column > end.column
                || (start.column == end.column
                && start.line > end.line) =>
                {
                    (Side::Right, Side::Left)
                }
            SelectionType::Block => (Side::Left, Side::Right),
            _ if start > end => (Side::Right, Side::Left),
            _ => (Side::Left, Side::Right),
        };

        self.region.start.side = start_side;
        self.region.end.side = end_side;
    }

    pub fn to_range<T>(&self, term: &Term<T>) -> Option<SelectionRange> {
        let grid = term.grid();
        let columns = grid.columns();

        let mut start = self.region.start;
        let mut end = self.region.end;

        if start.point > end.point {
            mem::swap(&mut start, &mut end);
        }

        if end.point.line < term.topmost_line() {
            return None;
        }
        start.point = start.point.grid_clamp(term, Boundary::Grid);
        end.point = end.point.grid_clamp(term, Boundary::Grid);

        match self.ty {
            SelectionType::Simple => self.range_simple(start, end, columns),
            SelectionType::Block => self.range_block(start, end),
            SelectionType::Semantic => {
                Some(Self::range_semantic(term, start.point, end.point))
            }
            SelectionType::Lines => {
                Some(Self::range_lines(term, start.point, end.point))
            }
        }
    }

    fn range_semantic<T>(
        term: &Term<T>,
        mut start: Point,
        mut end: Point,
    ) -> SelectionRange {
        if start == end {
            if let Some(matching) = term.bracket_search(start) {
                if (matching.line == start.line
                    && matching.column < start.column)
                    || (matching.line < start.line)
                {
                    start = matching;
                } else {
                    end = matching;
                }

                return SelectionRange {
                    start,
                    end,
                    is_block: false,
                };
            }
        }

        let start = term.semantic_search_left(start);
        let end = term.semantic_search_right(end);

        SelectionRange {
            start,
            end,
            is_block: false,
        }
    }

    fn range_lines<T>(
        term: &Term<T>,
        start: Point,
        end: Point,
    ) -> SelectionRange {
        let start = term.line_search_left(start);
        let end = term.line_search_right(end);

        SelectionRange {
            start,
            end,
            is_block: false,
        }
    }

    fn range_simple(
        &self,
        mut start: Anchor,
        mut end: Anchor,
        columns: usize,
    ) -> Option<SelectionRange> {
        if self.is_empty() {
            return None;
        }

        if end.side == Side::Left && start.point != end.point {
            if end.point.column == 0 {
                end.point.column = Column(columns - 1);
                end.point.line -= 1;
            } else {
                end.point.column -= 1;
            }
        }

        if start.side == Side::Right && start.point != end.point {
            start.point.column += 1;

            if start.point.column == columns {
                start.point.column = Column(0);
                start.point.line += 1;
            }
        }

        Some(SelectionRange {
            start: start.point,
            end: end.point,
            is_block: false,
        })
    }

    fn range_block(
        &self,
        mut start: Anchor,
        mut end: Anchor,
    ) -> Option<SelectionRange> {
        if self.is_empty() {
            return None;
        }

        if start.point.column > end.point.column {
            mem::swap(&mut start.side, &mut end.side);
            mem::swap(&mut start.point.column, &mut end.point.column);
        }

        if end.side == Side::Left
            && start.point != end.point
            && end.point.column.0 > 0
        {
            end.point.column -= 1;
        }

        if start.side == Side::Right && start.point != end.point {
            start.point.column += 1;
        }

        Some(SelectionRange {
            start: start.point,
            end: end.point,
            is_block: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        index::{Column, Point, Side},
        term::{test::TermSize, Config, Term},
    };

    fn term(height: usize, width: usize) -> Term<()> {
        let size = TermSize::new(width, height);
        Term::new(Config::default(), &size, ())
    }

    #[test]
    fn single_cell_left_to_right() {
        let location = Point::new(Line(0), Column(0));
        let mut selection =
            Selection::new(SelectionType::Simple, location, Side::Left);
        selection.update(location, Side::Right);

        assert_eq!(
            selection.to_range(&term(1, 2)).unwrap(),
            SelectionRange {
                start: location,
                end: location,
                is_block: false,
            }
        );
    }

    #[test]
    fn single_cell_right_to_left() {
        let location = Point::new(Line(0), Column(0));
        let mut selection =
            Selection::new(SelectionType::Simple, location, Side::Right);
        selection.update(location, Side::Left);

        assert_eq!(
            selection.to_range(&term(1, 2)).unwrap(),
            SelectionRange {
                start: location,
                end: location,
                is_block: false,
            }
        );
    }

    #[test]
    fn between_adjacent_cells_left_to_right() {
        let mut selection = Selection::new(
            SelectionType::Simple,
            Point::new(Line(0), Column(0)),
            Side::Right,
        );
        selection.update(Point::new(Line(0), Column(1)), Side::Left);

        assert_eq!(selection.to_range(&term(1, 2)), None);
    }

    #[test]
    fn between_adjacent_cells_right_to_left() {
        let mut selection = Selection::new(
            SelectionType::Simple,
            Point::new(Line(0), Column(1)),
            Side::Left,
        );
        selection.update(Point::new(Line(0), Column(0)), Side::Right);

        assert_eq!(selection.to_range(&term(1, 2)), None);
    }

    #[rustfmt::skip]
    #[test]
    fn across_adjacent_lines_upward_final_cell_exclusive() {
        let mut selection =
            Selection::new(SelectionType::Simple, Point::new(Line(0), Column(1)), Side::Right);
        selection.update(Point::new(Line(1), Column(1)), Side::Right);

        assert_eq!(selection.to_range(&term(2, 5)).unwrap(), SelectionRange {
            start: Point::new(Line(0), Column(2)),
            end: Point::new(Line(1), Column(1)),
            is_block: false,
        });
    }

    #[rustfmt::skip]
    #[test]
    fn selection_bigger_then_smaller() {
        let mut selection =
            Selection::new(SelectionType::Simple, Point::new(Line(1), Column(1)), Side::Right);
        selection.update(Point::new(Line(0), Column(1)), Side::Right);
        selection.update(Point::new(Line(0), Column(0)), Side::Right);

        assert_eq!(selection.to_range(&term(2, 5)).unwrap(), SelectionRange {
            start: Point::new(Line(0), Column(1)),
            end: Point::new(Line(1), Column(1)),
            is_block: false,
        });
    }

    #[test]
    fn line_selection() {
        let size = (10, 5);
        let mut selection = Selection::new(
            SelectionType::Lines,
            Point::new(Line(9), Column(1)),
            Side::Left,
        );
        selection.update(Point::new(Line(4), Column(1)), Side::Right);
        selection = selection
            .rotate(&size, &(Line(0)..Line(size.0 as i32)), 4)
            .unwrap();

        assert_eq!(
            selection.to_range(&term(size.0, size.1)).unwrap(),
            SelectionRange {
                start: Point::new(Line(0), Column(0)),
                end: Point::new(Line(5), Column(4)),
                is_block: false,
            }
        );
    }

    #[test]
    fn semantic_selection() {
        let size = (10, 5);
        let mut selection = Selection::new(
            SelectionType::Semantic,
            Point::new(Line(9), Column(3)),
            Side::Left,
        );
        selection.update(Point::new(Line(4), Column(1)), Side::Right);
        selection = selection
            .rotate(&size, &(Line(0)..Line(size.0 as i32)), 4)
            .unwrap();

        assert_eq!(
            selection.to_range(&term(size.0, size.1)).unwrap(),
            SelectionRange {
                start: Point::new(Line(0), Column(1)),
                end: Point::new(Line(5), Column(3)),
                is_block: false,
            }
        );
    }

    #[test]
    fn simple_selection() {
        let size = (10, 5);
        let mut selection = Selection::new(
            SelectionType::Simple,
            Point::new(Line(9), Column(3)),
            Side::Right,
        );
        selection.update(Point::new(Line(4), Column(1)), Side::Right);
        selection = selection
            .rotate(&size, &(Line(0)..Line(size.0 as i32)), 4)
            .unwrap();

        assert_eq!(
            selection.to_range(&term(size.0, size.1)).unwrap(),
            SelectionRange {
                start: Point::new(Line(0), Column(2)),
                end: Point::new(Line(5), Column(3)),
                is_block: false,
            }
        );
    }

    #[test]
    fn block_selection() {
        let size = (10, 5);
        let mut selection = Selection::new(
            SelectionType::Block,
            Point::new(Line(9), Column(3)),
            Side::Right,
        );
        selection.update(Point::new(Line(4), Column(1)), Side::Right);
        selection = selection
            .rotate(&size, &(Line(0)..Line(size.0 as i32)), 4)
            .unwrap();

        assert_eq!(
            selection.to_range(&term(size.0, size.1)).unwrap(),
            SelectionRange {
                start: Point::new(Line(0), Column(2)),
                end: Point::new(Line(5), Column(3)),
                is_block: true,
            }
        );
    }

    #[test]
    fn simple_is_empty() {
        let mut selection = Selection::new(
            SelectionType::Simple,
            Point::new(Line(1), Column(0)),
            Side::Right,
        );
        assert!(selection.is_empty());
        selection.update(Point::new(Line(1), Column(1)), Side::Left);
        assert!(selection.is_empty());
        selection.update(Point::new(Line(0), Column(0)), Side::Right);
        assert!(!selection.is_empty());
    }

    #[test]
    fn block_is_empty() {
        let mut selection = Selection::new(
            SelectionType::Block,
            Point::new(Line(1), Column(0)),
            Side::Right,
        );
        assert!(selection.is_empty());
        selection.update(Point::new(Line(1), Column(1)), Side::Left);
        assert!(selection.is_empty());
        selection.update(Point::new(Line(1), Column(1)), Side::Right);
        assert!(!selection.is_empty());
        selection.update(Point::new(Line(0), Column(0)), Side::Right);
        assert!(selection.is_empty());
        selection.update(Point::new(Line(0), Column(1)), Side::Left);
        assert!(selection.is_empty());
        selection.update(Point::new(Line(0), Column(1)), Side::Right);
        assert!(!selection.is_empty());
    }

    #[test]
    fn rotate_in_region_up() {
        let size = (10, 5);
        let mut selection = Selection::new(
            SelectionType::Simple,
            Point::new(Line(7), Column(3)),
            Side::Right,
        );
        selection.update(Point::new(Line(4), Column(1)), Side::Right);
        selection = selection
            .rotate(&size, &(Line(1)..Line(size.0 as i32 - 1)), 4)
            .unwrap();

        assert_eq!(
            selection.to_range(&term(size.0, size.1)).unwrap(),
            SelectionRange {
                start: Point::new(Line(1), Column(0)),
                end: Point::new(Line(3), Column(3)),
                is_block: false,
            }
        );
    }

    #[test]
    fn rotate_in_region_down() {
        let size = (10, 5);
        let mut selection = Selection::new(
            SelectionType::Simple,
            Point::new(Line(4), Column(3)),
            Side::Right,
        );
        selection.update(Point::new(Line(1), Column(1)), Side::Left);
        selection = selection
            .rotate(&size, &(Line(1)..Line(size.0 as i32 - 1)), -5)
            .unwrap();

        assert_eq!(
            selection.to_range(&term(size.0, size.1)).unwrap(),
            SelectionRange {
                start: Point::new(Line(6), Column(1)),
                end: Point::new(Line(8), size.last_column()),
                is_block: false,
            }
        );
    }

    #[test]
    fn rotate_in_region_up_block() {
        let size = (10, 5);
        let mut selection = Selection::new(
            SelectionType::Block,
            Point::new(Line(7), Column(3)),
            Side::Right,
        );
        selection.update(Point::new(Line(4), Column(1)), Side::Right);
        selection = selection
            .rotate(&size, &(Line(1)..Line(size.0 as i32 - 1)), 4)
            .unwrap();

        assert_eq!(
            selection.to_range(&term(size.0, size.1)).unwrap(),
            SelectionRange {
                start: Point::new(Line(1), Column(2)),
                end: Point::new(Line(3), Column(3)),
                is_block: true,
            }
        );
    }

    #[test]
    fn range_intersection() {
        let mut selection = Selection::new(
            SelectionType::Lines,
            Point::new(Line(3), Column(1)),
            Side::Left,
        );
        selection.update(Point::new(Line(6), Column(1)), Side::Right);

        assert!(selection.intersects_range(..));
        assert!(selection.intersects_range(Line(2)..));
        assert!(selection.intersects_range(Line(2)..=Line(4)));
        assert!(selection.intersects_range(Line(2)..=Line(7)));
        assert!(selection.intersects_range(Line(4)..=Line(5)));
        assert!(selection.intersects_range(Line(5)..Line(8)));

        assert!(!selection.intersects_range(..=Line(2)));
        assert!(!selection.intersects_range(Line(7)..=Line(8)));
    }
}
