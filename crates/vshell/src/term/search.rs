use std::{cmp::max, error::Error, mem, ops::RangeInclusive};

use log::{debug, warn};
use regex_automata::{
    Anchored,
    hybrid::dfa::{Builder, Cache, Config, DFA},
    Input,
    MatchKind, nfa::thompson::Config as ThompsonConfig, util::syntax::Config as SyntaxConfig,
};
pub use regex_automata::hybrid::BuildError;

use crate::{
    grid::{BidirectionalIterator, Dimensions, GridIterator, Indexed},
    index::{Boundary, Column, Direction, Point, Side},
    term::{
        cell::{Cell, Flags},
        Term,
    },
};

const BRACKET_PAIRS: [(char, char); 4] =
    [('(', ')'), ('[', ']'), ('{', '}'), ('<', '>')];

pub type Match = RangeInclusive<Point>;

#[derive(Clone, Debug)]
pub struct RegexSearch {
    left_fdfa: LazyDfa,
    left_rdfa: LazyDfa,
    right_rdfa: LazyDfa,
    right_fdfa: LazyDfa,
}

impl RegexSearch {
    pub fn new(search: &str) -> Result<RegexSearch, Box<BuildError>> {
        let has_uppercase = search.chars().any(|c| c.is_uppercase());
        let syntax_config =
            SyntaxConfig::new().case_insensitive(!has_uppercase);
        let config = Config::new()
            .minimum_cache_clear_count(Some(3))
            .minimum_bytes_per_state(Some(10));
        let max_size = config.get_cache_capacity();
        let thompson_config =
            ThompsonConfig::new().nfa_size_limit(Some(max_size));

        let left_rdfa = LazyDfa::new(
            search,
            config.clone(),
            syntax_config,
            thompson_config.clone(),
            Direction::Right,
            true,
        )?;
        let has_empty = left_rdfa.dfa.get_nfa().has_empty();
        let left_fdfa = LazyDfa::new(
            search,
            config.clone(),
            syntax_config,
            thompson_config.clone(),
            Direction::Left,
            has_empty,
        )?;

        let right_fdfa = LazyDfa::new(
            search,
            config.clone(),
            syntax_config,
            thompson_config.clone(),
            Direction::Right,
            has_empty,
        )?;
        let right_rdfa = LazyDfa::new(
            search,
            config,
            syntax_config,
            thompson_config,
            Direction::Left,
            true,
        )?;

        Ok(RegexSearch {
            left_fdfa,
            left_rdfa,
            right_fdfa,
            right_rdfa,
        })
    }
}

#[derive(Clone, Debug)]
struct LazyDfa {
    dfa: DFA,
    cache: Cache,
    direction: Direction,
    match_all: bool,
}

impl LazyDfa {
    fn new(
        search: &str,
        mut config: Config,
        syntax: SyntaxConfig,
        mut thompson: ThompsonConfig,
        direction: Direction,
        match_all: bool,
    ) -> Result<Self, Box<BuildError>> {
        thompson = match direction {
            Direction::Left => thompson.reverse(true),
            Direction::Right => thompson.reverse(false),
        };
        config = if match_all {
            config.match_kind(MatchKind::All)
        } else {
            config.match_kind(MatchKind::LeftmostFirst)
        };

        let dfa = Builder::new()
            .configure(config)
            .syntax(syntax)
            .thompson(thompson)
            .build(search)?;

        let cache = dfa.create_cache();

        Ok(Self {
            direction,
            cache,
            dfa,
            match_all,
        })
    }
}

impl<T> Term<T> {
    pub fn search_next(
        &self,
        regex: &mut RegexSearch,
        mut origin: Point,
        direction: Direction,
        side: Side,
        mut max_lines: Option<usize>,
    ) -> Option<Match> {
        origin = self.expand_wide(origin, direction);

        max_lines =
            max_lines.filter(|max_lines| max_lines + 1 < self.total_lines());

        match direction {
            Direction::Right => {
                self.next_match_right(regex, origin, side, max_lines)
            }
            Direction::Left => {
                self.next_match_left(regex, origin, side, max_lines)
            }
        }
    }

    fn next_match_right(
        &self,
        regex: &mut RegexSearch,
        origin: Point,
        side: Side,
        max_lines: Option<usize>,
    ) -> Option<Match> {
        let start = self.line_search_left(origin);
        let mut end = start;

        end = match max_lines {
            Some(max_lines) => {
                let line =
                    (start.line + max_lines).grid_clamp(self, Boundary::None);
                Point::new(line, self.last_column())
            }
            _ => end.sub(self, Boundary::None, 1),
        };

        let mut regex_iter =
            RegexIter::new(start, end, Direction::Right, self, regex)
                .peekable();

        let first_match = regex_iter.peek()?.clone();

        let regex_match = regex_iter
            .find(|regex_match| {
                let match_point = Self::match_side(regex_match, side);

                match_point.line < start.line
                    || match_point.line > origin.line
                    || (match_point.line == origin.line
                        && match_point.column >= origin.column)
            })
            .unwrap_or(first_match);

        Some(regex_match)
    }

    fn next_match_left(
        &self,
        regex: &mut RegexSearch,
        origin: Point,
        side: Side,
        max_lines: Option<usize>,
    ) -> Option<Match> {
        let start = self.line_search_right(origin);
        let mut end = start;

        end = match max_lines {
            Some(max_lines) => {
                let line =
                    (start.line - max_lines).grid_clamp(self, Boundary::None);
                Point::new(line, Column(0))
            }
            _ => end.add(self, Boundary::None, 1),
        };

        let mut regex_iter =
            RegexIter::new(start, end, Direction::Left, self, regex).peekable();

        let first_match = regex_iter.peek()?.clone();

        let regex_match = regex_iter
            .find(|regex_match| {
                let match_point = Self::match_side(regex_match, side);

                match_point.line > start.line
                    || match_point.line < origin.line
                    || (match_point.line == origin.line
                        && match_point.column <= origin.column)
            })
            .unwrap_or(first_match);

        Some(regex_match)
    }

    fn match_side(regex_match: &Match, side: Side) -> Point {
        match side {
            Side::Right => *regex_match.end(),
            Side::Left => *regex_match.start(),
        }
    }

    pub fn regex_search_left(
        &self,
        regex: &mut RegexSearch,
        start: Point,
        end: Point,
    ) -> Option<Match> {
        let match_start =
            self.regex_search(start, end, &mut regex.left_fdfa)?;
        let match_end =
            self.regex_search(match_start, start, &mut regex.left_rdfa)?;

        Some(match_start..=match_end)
    }

    pub fn regex_search_right(
        &self,
        regex: &mut RegexSearch,
        start: Point,
        end: Point,
    ) -> Option<Match> {
        let match_end = self.regex_search(start, end, &mut regex.right_fdfa)?;
        let match_start =
            self.regex_search(match_end, start, &mut regex.right_rdfa)?;

        Some(match_start..=match_end)
    }

    fn regex_search(
        &self,
        start: Point,
        end: Point,
        regex: &mut LazyDfa,
    ) -> Option<Point> {
        match self.regex_search_internal(start, end, regex) {
            Ok(regex_match) => regex_match,
            Err(err) => {
                warn!("Regex exceeded complexity limit");
                debug!("    {err}");
                None
            }
        }
    }

    fn regex_search_internal(
        &self,
        start: Point,
        end: Point,
        regex: &mut LazyDfa,
    ) -> Result<Option<Point>, Box<dyn Error>> {
        let topmost_line = self.topmost_line();
        let screen_lines = self.screen_lines() as i32;
        let last_column = self.last_column();

        let next = match regex.direction {
            Direction::Right => GridIterator::next,
            Direction::Left => GridIterator::prev,
        };

        let regex_anchored = if regex.match_all {
            Anchored::Yes
        } else {
            Anchored::No
        };
        let input = Input::new(&[]).anchored(regex_anchored);
        let mut state = regex
            .dfa
            .start_state_forward(&mut regex.cache, &input)
            .unwrap();

        let mut iter = self.grid.iter_from(start);
        let mut last_wrapped = false;
        let mut regex_match = None;
        let mut done = false;

        let mut cell = iter.cell();
        self.skip_fullwidth(&mut iter, &mut cell, regex.direction);
        let mut c = cell.c;

        let mut point = iter.point();
        let mut last_point = point;
        let mut consumed_bytes = 0;

        macro_rules! reset_state {
            () => {{
                state =
                    regex.dfa.start_state_forward(&mut regex.cache, &input)?;
                consumed_bytes = 0;
                regex_match = None;
            }};
        }

        'outer: loop {
            let mut buf = [0; 4];
            let utf8_len = c.encode_utf8(&mut buf).len();

            for i in 0..utf8_len {
                let byte = match regex.direction {
                    Direction::Right => buf[i],
                    Direction::Left => buf[utf8_len - i - 1],
                };

                state = regex.dfa.next_state(&mut regex.cache, state, byte)?;
                consumed_bytes += 1;

                if i == 0 && state.is_match() {
                    regex_match = Some(last_point);
                } else if state.is_dead() {
                    if consumed_bytes == 2 {
                        reset_state!();

                        if i == 0 {
                            continue 'outer;
                        }
                    } else {
                        break 'outer;
                    }
                }
            }

            if point == end || done {
                state = regex.dfa.next_eoi_state(&mut regex.cache, state)?;
                if state.is_match() {
                    regex_match = Some(point);
                } else if state.is_dead() && consumed_bytes == 1 {
                    regex_match = None;
                }

                break;
            }

            let mut cell = match next(&mut iter) {
                Some(Indexed { cell, .. }) => cell,
                None => {
                    let line = topmost_line - point.line + screen_lines - 1;
                    let start = Point::new(line, last_column - point.column);
                    iter = self.grid.iter_from(start);
                    iter.cell()
                }
            };

            done = iter.point() == end;

            self.skip_fullwidth(&mut iter, &mut cell, regex.direction);

            let wrapped = cell.flags.contains(Flags::WRAPLINE);
            c = cell.c;

            last_point = mem::replace(&mut point, iter.point());

            if (last_point.column == last_column
                && point.column == Column(0)
                && !last_wrapped)
                || (last_point.column == Column(0)
                    && point.column == last_column
                    && !wrapped)
            {
                state = regex.dfa.next_eoi_state(&mut regex.cache, state)?;
                if state.is_match() {
                    regex_match = Some(last_point);
                }

                match regex_match {
                    Some(_)
                        if (!state.is_dead() || consumed_bytes > 1)
                            && consumed_bytes != 0 =>
                    {
                        break;
                    }
                    _ => reset_state!(),
                }
            }

            last_wrapped = wrapped;
        }

        Ok(regex_match)
    }

    fn skip_fullwidth<'a>(
        &self,
        iter: &'a mut GridIterator<'_, Cell>,
        cell: &mut &'a Cell,
        direction: Direction,
    ) {
        match direction {
            Direction::Right
                if cell.flags.contains(Flags::WIDE_CHAR)
                    && iter.point().column < self.last_column() =>
            {
                iter.next();
            }
            Direction::Right
                if cell.flags.contains(Flags::LEADING_WIDE_CHAR_SPACER) =>
            {
                if let Some(Indexed { cell: new_cell, .. }) = iter.next() {
                    *cell = new_cell;
                }
                iter.next();
            }
            Direction::Left if cell.flags.contains(Flags::WIDE_CHAR_SPACER) => {
                if let Some(Indexed { cell: new_cell, .. }) = iter.prev() {
                    *cell = new_cell;
                }

                let prev = iter.point().sub(self, Boundary::Grid, 1);
                if self.grid[prev]
                    .flags
                    .contains(Flags::LEADING_WIDE_CHAR_SPACER)
                {
                    iter.prev();
                }
            }
            _ => (),
        }
    }

    pub fn bracket_search(&self, point: Point) -> Option<Point> {
        let start_char = self.grid[point].c;

        let (forward, end_char) =
            BRACKET_PAIRS.iter().find_map(|(open, close)| {
                if open == &start_char {
                    Some((true, *close))
                } else if close == &start_char {
                    Some((false, *open))
                } else {
                    None
                }
            })?;

        let mut iter = self.grid.iter_from(point);

        let mut skip_pairs = 0;

        loop {
            let cell = if forward { iter.next() } else { iter.prev() };

            let cell = match cell {
                Some(cell) => cell,
                None => break,
            };

            if cell.c == end_char && skip_pairs == 0 {
                return Some(cell.point);
            } else if cell.c == start_char {
                skip_pairs += 1;
            } else if cell.c == end_char {
                skip_pairs -= 1;
            }
        }

        None
    }

    #[must_use]
    pub fn semantic_search_left(&self, point: Point) -> Point {
        match self.inline_search_left(point, self.semantic_escape_chars()) {
            Ok(point) => self
                .grid
                .iter_from(point)
                .next()
                .map_or(point, |cell| cell.point),
            Err(point) => point,
        }
    }

    #[must_use]
    pub fn semantic_search_right(&self, point: Point) -> Point {
        match self.inline_search_right(point, self.semantic_escape_chars()) {
            Ok(point) => self
                .grid
                .iter_from(point)
                .prev()
                .map_or(point, |cell| cell.point),
            Err(point) => point,
        }
    }

    pub fn inline_search_left(
        &self,
        mut point: Point,
        needles: &str,
    ) -> Result<Point, Point> {
        point.line = max(point.line, self.topmost_line());

        let mut iter = self.grid.iter_from(point);
        let last_column = self.columns() - 1;

        let wide = Flags::WIDE_CHAR
            | Flags::WIDE_CHAR_SPACER
            | Flags::LEADING_WIDE_CHAR_SPACER;
        while let Some(cell) = iter.prev() {
            if cell.point.column == last_column
                && !cell.flags.contains(Flags::WRAPLINE)
            {
                break;
            }

            point = cell.point;

            if !cell.flags.intersects(wide) && needles.contains(cell.c) {
                return Ok(point);
            }
        }

        Err(point)
    }

    pub fn inline_search_right(
        &self,
        mut point: Point,
        needles: &str,
    ) -> Result<Point, Point> {
        point.line = max(point.line, self.topmost_line());

        let wide = Flags::WIDE_CHAR
            | Flags::WIDE_CHAR_SPACER
            | Flags::LEADING_WIDE_CHAR_SPACER;
        let last_column = self.columns() - 1;

        if point.column == last_column
            && !self.grid[point].flags.contains(Flags::WRAPLINE)
        {
            return Err(point);
        }

        for cell in self.grid.iter_from(point) {
            point = cell.point;

            if !cell.flags.intersects(wide) && needles.contains(cell.c) {
                return Ok(point);
            }

            if point.column == last_column
                && !cell.flags.contains(Flags::WRAPLINE)
            {
                break;
            }
        }

        Err(point)
    }

    pub fn line_search_left(&self, mut point: Point) -> Point {
        while point.line > self.topmost_line()
            && self.grid[point.line - 1i32][self.last_column()]
                .flags
                .contains(Flags::WRAPLINE)
        {
            point.line -= 1;
        }

        point.column = Column(0);

        point
    }

    pub fn line_search_right(&self, mut point: Point) -> Point {
        while point.line + 1 < self.screen_lines()
            && self.grid[point.line][self.last_column()]
                .flags
                .contains(Flags::WRAPLINE)
        {
            point.line += 1;
        }

        point.column = self.last_column();

        point
    }
}

pub struct RegexIter<'a, T> {
    point: Point,
    end: Point,
    direction: Direction,
    regex: &'a mut RegexSearch,
    term: &'a Term<T>,
    done: bool,
}

impl<'a, T> RegexIter<'a, T> {
    pub fn new(
        start: Point,
        end: Point,
        direction: Direction,
        term: &'a Term<T>,
        regex: &'a mut RegexSearch,
    ) -> Self {
        Self {
            point: start,
            done: false,
            end,
            direction,
            term,
            regex,
        }
    }

    fn skip(&mut self) {
        self.point = self.term.expand_wide(self.point, self.direction);

        self.point = match self.direction {
            Direction::Right => self.point.add(self.term, Boundary::None, 1),
            Direction::Left => self.point.sub(self.term, Boundary::None, 1),
        };
    }

    fn next_match(&mut self) -> Option<Match> {
        match self.direction {
            Direction::Right => self
                .term
                .regex_search_right(self.regex, self.point, self.end),
            Direction::Left => self
                .term
                .regex_search_left(self.regex, self.point, self.end),
        }
    }
}

impl<'a, T> Iterator for RegexIter<'a, T> {
    type Item = Match;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        if self.point == self.end {
            self.done = true;
        }

        let regex_match = self.next_match()?;

        self.point = *regex_match.end();
        if self.point == self.end {
            self.done = true;
        } else {
            self.skip();
        }

        Some(regex_match)
    }
}
