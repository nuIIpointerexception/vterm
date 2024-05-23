use std::{
    cmp, mem,
    ops::{Index, IndexMut, Range},
    ptr, slice, str,
    sync::Arc,
};

use base64::{engine::general_purpose::STANDARD as Base64, Engine};
use bitflags::bitflags;
use log::{debug, trace};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use unicode_width::UnicodeWidthChar;

use crate::{
    event::{Event, EventListener},
    grid::{Dimensions, Grid, GridIterator, Scroll},
    index::{self, Boundary, Column, Direction, Line, Point, Side},
    selection::{Selection, SelectionRange, SelectionType},
    term::{
        cell::{Cell, Flags, LineLength},
        color::Colors,
    },
    vi_mode::{ViModeCursor, ViMotion},
    vte::ansi::{
        self, Attr, CharsetIndex, Color, CursorShape, CursorStyle, Handler,
        Hyperlink, KeyboardModes, KeyboardModesApplyBehavior, NamedColor,
        NamedMode, NamedPrivateMode, PrivateMode, Rgb, StandardCharset,
    },
};

pub mod cell;
pub mod color;
pub mod search;

pub const MIN_COLUMNS: usize = 2;

pub const MIN_SCREEN_LINES: usize = 1;

const TITLE_STACK_MAX_DEPTH: usize = 4096;

pub const SEMANTIC_ESCAPE_CHARS: &str = ",│`|:\"' ()[]{}<>\t";

const KEYBOARD_MODE_STACK_MAX_DEPTH: usize = TITLE_STACK_MAX_DEPTH;

const INITIAL_TABSTOPS: usize = 8;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct TermMode: u32 {
        const NONE                    = 0;
        const SHOW_CURSOR             = 0b0000_0000_0000_0000_0000_0001;
        const APP_CURSOR              = 0b0000_0000_0000_0000_0000_0010;
        const APP_KEYPAD              = 0b0000_0000_0000_0000_0000_0100;
        const MOUSE_REPORT_CLICK      = 0b0000_0000_0000_0000_0000_1000;
        const BRACKETED_PASTE         = 0b0000_0000_0000_0000_0001_0000;
        const SGR_MOUSE               = 0b0000_0000_0000_0000_0010_0000;
        const MOUSE_MOTION            = 0b0000_0000_0000_0000_0100_0000;
        const LINE_WRAP               = 0b0000_0000_0000_0000_1000_0000;
        const LINE_FEED_NEW_LINE      = 0b0000_0000_0000_0001_0000_0000;
        const ORIGIN                  = 0b0000_0000_0000_0010_0000_0000;
        const INSERT                  = 0b0000_0000_0000_0100_0000_0000;
        const FOCUS_IN_OUT            = 0b0000_0000_0000_1000_0000_0000;
        const ALT_SCREEN              = 0b0000_0000_0001_0000_0000_0000;
        const MOUSE_DRAG              = 0b0000_0000_0010_0000_0000_0000;
        const MOUSE_MODE              = 0b0000_0000_0010_0000_0100_1000;
        const UTF8_MOUSE              = 0b0000_0000_0100_0000_0000_0000;
        const ALTERNATE_SCROLL        = 0b0000_0000_1000_0000_0000_0000;
        const VI                      = 0b0000_0001_0000_0000_0000_0000;
        const URGENCY_HINTS           = 0b0000_0010_0000_0000_0000_0000;
        const DISAMBIGUATE_ESC_CODES  = 0b0000_0100_0000_0000_0000_0000;
        const REPORT_EVENT_TYPES      = 0b0000_1000_0000_0000_0000_0000;
        const REPORT_ALTERNATE_KEYS   = 0b0001_0000_0000_0000_0000_0000;
        const REPORT_ALL_KEYS_AS_ESC  = 0b0010_0000_0000_0000_0000_0000;
        const REPORT_ASSOCIATED_TEXT  = 0b0100_0000_0000_0000_0000_0000;
        const KITTY_KEYBOARD_PROTOCOL = Self::DISAMBIGUATE_ESC_CODES.bits()
                                      | Self::REPORT_EVENT_TYPES.bits()
                                      | Self::REPORT_ALTERNATE_KEYS.bits()
                                      | Self::REPORT_ALL_KEYS_AS_ESC.bits()
                                      | Self::REPORT_ASSOCIATED_TEXT.bits();
         const ANY                    = u32::MAX;
    }
}

impl From<KeyboardModes> for TermMode {
    fn from(value: KeyboardModes) -> Self {
        let mut mode = Self::empty();

        let disambiguate_esc_codes =
            value.contains(KeyboardModes::DISAMBIGUATE_ESC_CODES);
        mode.set(TermMode::DISAMBIGUATE_ESC_CODES, disambiguate_esc_codes);

        let report_event_types =
            value.contains(KeyboardModes::REPORT_EVENT_TYPES);
        mode.set(TermMode::REPORT_EVENT_TYPES, report_event_types);

        let report_alternate_keys =
            value.contains(KeyboardModes::REPORT_ALTERNATE_KEYS);
        mode.set(TermMode::REPORT_ALTERNATE_KEYS, report_alternate_keys);

        let report_all_keys_as_esc =
            value.contains(KeyboardModes::REPORT_ALL_KEYS_AS_ESC);
        mode.set(TermMode::REPORT_ALL_KEYS_AS_ESC, report_all_keys_as_esc);

        let report_associated_text =
            value.contains(KeyboardModes::REPORT_ASSOCIATED_TEXT);
        mode.set(TermMode::REPORT_ASSOCIATED_TEXT, report_associated_text);

        mode
    }
}

impl Default for TermMode {
    fn default() -> TermMode {
        TermMode::SHOW_CURSOR
            | TermMode::LINE_WRAP
            | TermMode::ALTERNATE_SCROLL
            | TermMode::URGENCY_HINTS
    }
}

#[inline]
pub fn point_to_viewport(
    display_offset: usize,
    point: Point,
) -> Option<Point<usize>> {
    let viewport_line = point.line.0 + display_offset as i32;
    usize::try_from(viewport_line)
        .ok()
        .map(|line| Point::new(line, point.column))
}

#[inline]
pub fn viewport_to_point(display_offset: usize, point: Point<usize>) -> Point {
    let line = Line(point.line as i32) - display_offset;
    Point::new(line, point.column)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LineDamageBounds {
    pub line: usize,

    pub left: usize,

    pub right: usize,
}

impl LineDamageBounds {
    #[inline]
    pub fn new(line: usize, left: usize, right: usize) -> Self {
        Self { line, left, right }
    }

    #[inline]
    pub fn undamaged(line: usize, num_cols: usize) -> Self {
        Self {
            line,
            left: num_cols,
            right: 0,
        }
    }

    #[inline]
    pub fn reset(&mut self, num_cols: usize) {
        *self = Self::undamaged(self.line, num_cols);
    }

    #[inline]
    pub fn expand(&mut self, left: usize, right: usize) {
        self.left = cmp::min(self.left, left);
        self.right = cmp::max(self.right, right);
    }

    #[inline]
    pub fn is_damaged(&self) -> bool {
        self.left <= self.right
    }
}

#[derive(Debug)]
pub enum TermDamage<'a> {
    Full,

    Partial(TermDamageIterator<'a>),
}

#[derive(Clone, Debug)]
pub struct TermDamageIterator<'a> {
    line_damage: slice::Iter<'a, LineDamageBounds>,
    display_offset: usize,
}

impl<'a> TermDamageIterator<'a> {
    pub fn new(
        line_damage: &'a [LineDamageBounds],
        display_offset: usize,
    ) -> Self {
        let num_lines = line_damage.len();
        let line_damage =
            &line_damage[..num_lines.saturating_sub(display_offset)];
        Self {
            display_offset,
            line_damage: line_damage.iter(),
        }
    }
}

impl<'a> Iterator for TermDamageIterator<'a> {
    type Item = LineDamageBounds;

    fn next(&mut self) -> Option<Self::Item> {
        self.line_damage.find_map(|line| {
            line.is_damaged().then_some(LineDamageBounds::new(
                line.line + self.display_offset,
                line.left,
                line.right,
            ))
        })
    }
}

struct TermDamageState {
    full: bool,

    lines: Vec<LineDamageBounds>,

    last_cursor: Point,
}

impl TermDamageState {
    fn new(num_cols: usize, num_lines: usize) -> Self {
        let lines = (0..num_lines)
            .map(|line| LineDamageBounds::undamaged(line, num_cols))
            .collect();

        Self {
            full: true,
            lines,
            last_cursor: Default::default(),
        }
    }

    #[inline]
    fn resize(&mut self, num_cols: usize, num_lines: usize) {
        self.last_cursor = Default::default();
        self.full = true;

        self.lines.clear();
        self.lines.reserve(num_lines);
        for line in 0..num_lines {
            self.lines.push(LineDamageBounds::undamaged(line, num_cols));
        }
    }

    #[inline]
    fn damage_point(&mut self, point: Point<usize>) {
        self.damage_line(point.line, point.column.0, point.column.0);
    }

    #[inline]
    fn damage_line(&mut self, line: usize, left: usize, right: usize) {
        self.lines[line].expand(left, right);
    }

    fn reset(&mut self, num_cols: usize) {
        self.full = false;
        self.lines.iter_mut().for_each(|line| line.reset(num_cols));
    }
}

pub struct Term<T> {
    pub is_focused: bool,

    pub vi_mode_cursor: ViModeCursor,

    pub selection: Option<Selection>,

    grid: Grid<Cell>,

    inactive_grid: Grid<Cell>,

    active_charset: CharsetIndex,

    tabs: TabStops,

    mode: TermMode,

    scroll_region: Range<Line>,

    colors: Colors,

    cursor_style: Option<CursorStyle>,

    event_proxy: T,

    title: Option<String>,

    title_stack: Vec<Option<String>>,

    keyboard_mode_stack: Vec<KeyboardModes>,

    inactive_keyboard_mode_stack: Vec<KeyboardModes>,

    damage: TermDamageState,

    config: Config,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub scrolling_history: usize,

    pub default_cursor_style: CursorStyle,

    pub vi_mode_cursor_style: Option<CursorStyle>,

    pub semantic_escape_chars: String,

    pub kitty_keyboard: bool,

    pub osc52: Osc52,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            scrolling_history: 10000,
            semantic_escape_chars: SEMANTIC_ESCAPE_CHARS.to_owned(),
            default_cursor_style: Default::default(),
            vi_mode_cursor_style: Default::default(),
            kitty_keyboard: Default::default(),
            osc52: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "lowercase")
)]
pub enum Osc52 {
    Disabled,
    #[default]
    OnlyCopy,
    OnlyPaste,
    CopyPaste,
}

impl<T> Term<T> {
    #[inline]
    pub fn scroll_display(&mut self, scroll: Scroll)
        where
            T: EventListener,
    {
        let old_display_offset = self.grid.display_offset();
        self.grid.scroll_display(scroll);
        self.event_proxy.send_event(Event::MouseCursorDirty);

        let viewport_start = -(self.grid.display_offset() as i32);
        let viewport_end = viewport_start + self.bottommost_line().0;
        let vi_cursor_line = &mut self.vi_mode_cursor.point.line.0;
        *vi_cursor_line =
            cmp::min(viewport_end, cmp::max(viewport_start, *vi_cursor_line));
        self.vi_mode_recompute_selection();

        if old_display_offset != self.grid().display_offset() {
            self.mark_fully_damaged();
        }
    }

    pub fn new<D: Dimensions>(
        config: Config,
        dimensions: &D,
        event_proxy: T,
    ) -> Term<T> {
        let num_cols = dimensions.columns();
        let num_lines = dimensions.screen_lines();

        let history_size = config.scrolling_history;
        let grid = Grid::new(num_lines, num_cols, history_size);
        let inactive_grid = Grid::new(num_lines, num_cols, 0);

        let tabs = TabStops::new(grid.columns());

        let scroll_region = Line(0)..Line(grid.screen_lines() as i32);

        let damage = TermDamageState::new(num_cols, num_lines);

        Term {
            inactive_grid,
            scroll_region,
            event_proxy,
            damage,
            config,
            grid,
            tabs,
            inactive_keyboard_mode_stack: Default::default(),
            keyboard_mode_stack: Default::default(),
            active_charset: Default::default(),
            vi_mode_cursor: Default::default(),
            cursor_style: Default::default(),
            colors: color::Colors::default(),
            title_stack: Default::default(),
            is_focused: Default::default(),
            selection: Default::default(),
            title: Default::default(),
            mode: Default::default(),
        }
    }

    #[must_use]
    pub fn damage(&mut self) -> TermDamage<'_> {
        if self.mode.contains(TermMode::INSERT) {
            self.mark_fully_damaged();
        }

        let previous_cursor =
            mem::replace(&mut self.damage.last_cursor, self.grid.cursor.point);

        if self.damage.full {
            return TermDamage::Full;
        }

        if self.damage.last_cursor != previous_cursor {
            let point = Point::new(
                previous_cursor.line.0 as usize,
                previous_cursor.column,
            );
            self.damage.damage_point(point);
        }

        self.damage_cursor();

        let display_offset = self.grid().display_offset();
        TermDamage::Partial(TermDamageIterator::new(
            &self.damage.lines,
            display_offset,
        ))
    }

    pub fn reset_damage(&mut self) {
        self.damage.reset(self.columns());
    }

    #[inline]
    fn mark_fully_damaged(&mut self) {
        self.damage.full = true;
    }

    pub fn set_options(&mut self, options: Config)
        where
            T: EventListener,
    {
        let old_config = mem::replace(&mut self.config, options);

        let title_event = match &self.title {
            Some(title) => Event::Title(title.clone()),
            None => Event::ResetTitle,
        };

        self.event_proxy.send_event(title_event);

        if self.mode.contains(TermMode::ALT_SCREEN) {
            self.inactive_grid
                .update_history(self.config.scrolling_history);
        } else {
            self.grid.update_history(self.config.scrolling_history);
        }

        if self.config.kitty_keyboard != old_config.kitty_keyboard {
            self.keyboard_mode_stack = Vec::new();
            self.inactive_keyboard_mode_stack = Vec::new();
            self.mode.remove(TermMode::KITTY_KEYBOARD_PROTOCOL);
        }

        self.mark_fully_damaged();
    }

    pub fn selection_to_string(&self) -> Option<String> {
        let selection_range =
            self.selection.as_ref().and_then(|s| s.to_range(self))?;
        let SelectionRange { start, end, .. } = selection_range;

        let mut res = String::new();

        match self.selection.as_ref() {
            Some(Selection {
                     ty: SelectionType::Block,
                     ..
                 }) => {
                for line in (start.line.0..end.line.0).map(Line::from) {
                    res += self
                        .line_to_string(
                            line,
                            start.column..end.column,
                            start.column.0 != 0,
                        )
                        .trim_end();
                    res += "\n";
                }

                res += self
                    .line_to_string(end.line, start.column..end.column, true)
                    .trim_end();
            }
            Some(Selection {
                     ty: SelectionType::Lines,
                     ..
                 }) => {
                res = self.bounds_to_string(start, end) + "\n";
            }
            _ => {
                res = self.bounds_to_string(start, end);
            }
        }

        Some(res)
    }

    pub fn bounds_to_string(&self, start: Point, end: Point) -> String {
        let mut res = String::new();

        for line in (start.line.0..=end.line.0).map(Line::from) {
            let start_col = if line == start.line {
                start.column
            } else {
                Column(0)
            };
            let end_col = if line == end.line {
                end.column
            } else {
                self.last_column()
            };

            res += &self.line_to_string(
                line,
                start_col..end_col,
                line == end.line,
            );
        }

        res.strip_suffix('\n').map(str::to_owned).unwrap_or(res)
    }

    fn line_to_string(
        &self,
        line: Line,
        mut cols: Range<Column>,
        include_wrapped_wide: bool,
    ) -> String {
        let mut text = String::new();

        let grid_line = &self.grid[line];
        let line_length = cmp::min(grid_line.line_length(), cols.end + 1);

        if grid_line[cols.start]
            .flags
            .contains(Flags::WIDE_CHAR_SPACER)
        {
            cols.start -= 1;
        }

        let mut tab_mode = false;
        for column in (cols.start.0..line_length.0).map(Column::from) {
            let cell = &grid_line[column];

            if tab_mode {
                if self.tabs[column] || cell.c != ' ' {
                    tab_mode = false;
                } else {
                    continue;
                }
            }

            if cell.c == '\t' {
                tab_mode = true;
            }

            if !cell.flags.intersects(
                Flags::WIDE_CHAR_SPACER | Flags::LEADING_WIDE_CHAR_SPACER,
            ) {
                text.push(cell.c);

                for c in cell.zerowidth().into_iter().flatten() {
                    text.push(*c);
                }
            }
        }

        if cols.end >= self.columns() - 1
            && (line_length.0 == 0
            || !self.grid[line][line_length - 1]
            .flags
            .contains(Flags::WRAPLINE))
        {
            text.push('\n');
        }

        if line_length == self.columns()
            && line_length.0 >= 2
            && grid_line[line_length - 1]
            .flags
            .contains(Flags::LEADING_WIDE_CHAR_SPACER)
            && include_wrapped_wide
        {
            text.push(self.grid[line - 1i32][Column(0)].c);
        }

        text
    }

    #[inline]
    pub fn renderable_content(&self) -> RenderableContent<'_>
        where
            T: EventListener,
    {
        RenderableContent::new(self)
    }

    pub fn grid(&self) -> &Grid<Cell> {
        &self.grid
    }

    pub fn grid_mut(&mut self) -> &mut Grid<Cell> {
        &mut self.grid
    }

    pub fn resize<S: Dimensions>(&mut self, size: S) {
        let old_cols = self.columns();
        let old_lines = self.screen_lines();

        let num_cols = size.columns();
        let num_lines = size.screen_lines();

        if old_cols == num_cols && old_lines == num_lines {
            debug!("Term::resize dimensions unchanged");
            return;
        }

        debug!(
            "New num_cols is {} and num_lines is {}",
            num_cols, num_lines
        );

        let history_size = self.history_size();
        let mut delta = num_lines as i32 - old_lines as i32;
        let min_delta =
            cmp::min(0, num_lines as i32 - self.grid.cursor.point.line.0 - 1);
        delta = cmp::min(cmp::max(delta, min_delta), history_size as i32);
        self.vi_mode_cursor.point.line += delta;

        let is_alt = self.mode.contains(TermMode::ALT_SCREEN);
        self.grid.resize(!is_alt, num_lines, num_cols);
        self.inactive_grid.resize(is_alt, num_lines, num_cols);

        if old_cols != num_cols {
            self.selection = None;

            self.tabs.resize(num_cols);
        } else if let Some(selection) = self.selection.take() {
            let max_lines = cmp::max(num_lines, old_lines) as i32;
            let range = Line(0)..Line(max_lines);
            self.selection = selection.rotate(self, &range, -delta);
        }

        let vi_point = self.vi_mode_cursor.point;
        let viewport_top = Line(-(self.grid.display_offset() as i32));
        let viewport_bottom = viewport_top + self.bottommost_line();
        self.vi_mode_cursor.point.line =
            cmp::max(cmp::min(vi_point.line, viewport_bottom), viewport_top);
        self.vi_mode_cursor.point.column =
            cmp::min(vi_point.column, self.last_column());

        self.scroll_region = Line(0)..Line(self.screen_lines() as i32);

        self.damage.resize(num_cols, num_lines);
    }

    #[inline]
    pub fn mode(&self) -> &TermMode {
        &self.mode
    }

    pub fn swap_alt(&mut self) {
        if !self.mode.contains(TermMode::ALT_SCREEN) {
            self.inactive_grid.cursor = self.grid.cursor.clone();

            self.grid.saved_cursor = self.grid.cursor.clone();

            self.inactive_grid.reset_region(..);
        }

        mem::swap(
            &mut self.keyboard_mode_stack,
            &mut self.inactive_keyboard_mode_stack,
        );
        let keyboard_mode = self
            .keyboard_mode_stack
            .last()
            .copied()
            .unwrap_or(KeyboardModes::NO_MODE)
            .into();
        self.set_keyboard_mode(
            keyboard_mode,
            KeyboardModesApplyBehavior::Replace,
        );

        mem::swap(&mut self.grid, &mut self.inactive_grid);
        self.mode ^= TermMode::ALT_SCREEN;
        self.selection = None;
        self.mark_fully_damaged();
    }

    #[inline]
    fn scroll_down_relative(&mut self, origin: Line, mut lines: usize) {
        trace!(
            "Scrolling down relative: origin={}, lines={}",
            origin,
            lines
        );

        lines = cmp::min(
            lines,
            (self.scroll_region.end - self.scroll_region.start).0 as usize,
        );
        lines = cmp::min(lines, (self.scroll_region.end - origin).0 as usize);

        let region = origin..self.scroll_region.end;

        self.selection = self
            .selection
            .take()
            .and_then(|s| s.rotate(self, &region, -(lines as i32)));

        let line = &mut self.vi_mode_cursor.point.line;
        if region.start <= *line && region.end > *line {
            *line = cmp::min(*line + lines, region.end - 1);
        }

        self.grid.scroll_down(&region, lines);
        self.mark_fully_damaged();
    }

    #[inline]
    fn scroll_up_relative(&mut self, origin: Line, mut lines: usize) {
        trace!("Scrolling up relative: origin={}, lines={}", origin, lines);

        lines = cmp::min(
            lines,
            (self.scroll_region.end - self.scroll_region.start).0 as usize,
        );

        let region = origin..self.scroll_region.end;

        self.selection = self
            .selection
            .take()
            .and_then(|s| s.rotate(self, &region, lines as i32));

        self.grid.scroll_up(&region, lines);

        let viewport_top = Line(-(self.grid.display_offset() as i32));
        let top = if region.start == 0 {
            viewport_top
        } else {
            region.start
        };
        let line = &mut self.vi_mode_cursor.point.line;
        if (top <= *line) && region.end > *line {
            *line = cmp::max(*line - lines, top);
        }
        self.mark_fully_damaged();
    }

    fn deccolm(&mut self)
        where
            T: EventListener,
    {
        self.set_scrolling_region(1, None);

        self.grid.reset_region(..);
        self.mark_fully_damaged();
    }

    #[inline]
    pub fn exit(&mut self)
        where
            T: EventListener,
    {
        self.event_proxy.send_event(Event::Exit);
    }

    #[inline]
    pub fn toggle_vi_mode(&mut self)
        where
            T: EventListener,
    {
        self.mode ^= TermMode::VI;

        if self.mode.contains(TermMode::VI) {
            let display_offset = self.grid.display_offset() as i32;
            if self.grid.cursor.point.line
                > self.bottommost_line() - display_offset
            {
                let point = Point::new(Line(-display_offset), Column(0));
                self.vi_mode_cursor = ViModeCursor::new(point);
            } else {
                self.vi_mode_cursor = ViModeCursor::new(self.grid.cursor.point);
            }
        }

        self.event_proxy.send_event(Event::CursorBlinkingChange);
    }

    #[inline]
    pub fn vi_motion(&mut self, motion: ViMotion)
        where
            T: EventListener,
    {
        if !self.mode.contains(TermMode::VI) {
            return;
        }

        self.vi_mode_cursor = self.vi_mode_cursor.motion(self, motion);
        self.vi_mode_recompute_selection();
    }

    #[inline]
    pub fn vi_goto_point(&mut self, point: Point)
        where
            T: EventListener,
    {
        self.scroll_to_point(point);

        self.vi_mode_cursor.point = point;

        self.vi_mode_recompute_selection();
    }

    #[inline]
    fn vi_mode_recompute_selection(&mut self) {
        if !self.mode.contains(TermMode::VI) {
            return;
        }

        if let Some(selection) =
            self.selection.as_mut().filter(|s| !s.is_empty())
        {
            selection.update(self.vi_mode_cursor.point, Side::Left);
            selection.include_all();
        }
    }

    pub fn scroll_to_point(&mut self, point: Point)
        where
            T: EventListener,
    {
        let display_offset = self.grid.display_offset() as i32;
        let screen_lines = self.grid.screen_lines() as i32;

        if point.line < -display_offset {
            let lines = point.line + display_offset;
            self.scroll_display(Scroll::Delta(-lines.0));
        } else if point.line >= (screen_lines - display_offset) {
            let lines = point.line + display_offset - screen_lines + 1i32;
            self.scroll_display(Scroll::Delta(-lines.0));
        }
    }

    pub fn expand_wide(&self, mut point: Point, direction: Direction) -> Point {
        let flags = self.grid[point.line][point.column].flags;

        match direction {
            Direction::Right
            if flags.contains(Flags::LEADING_WIDE_CHAR_SPACER) =>
                {
                    point.column = Column(1);
                    point.line += 1;
                }
            Direction::Right if flags.contains(Flags::WIDE_CHAR) => {
                point.column = cmp::min(point.column + 1, self.last_column());
            }
            Direction::Left
            if flags
                .intersects(Flags::WIDE_CHAR | Flags::WIDE_CHAR_SPACER) =>
                {
                    if flags.contains(Flags::WIDE_CHAR_SPACER) {
                        point.column -= 1;
                    }

                    let prev = point.sub(self, Boundary::Grid, 1);
                    if self.grid[prev]
                        .flags
                        .contains(Flags::LEADING_WIDE_CHAR_SPACER)
                    {
                        point = prev;
                    }
                }
            _ => (),
        }

        point
    }

    #[inline]
    pub fn semantic_escape_chars(&self) -> &str {
        &self.config.semantic_escape_chars
    }

    #[inline]
    pub fn cursor_style(&self) -> CursorStyle {
        let cursor_style = self
            .cursor_style
            .unwrap_or(self.config.default_cursor_style);

        if self.mode.contains(TermMode::VI) {
            self.config.vi_mode_cursor_style.unwrap_or(cursor_style)
        } else {
            cursor_style
        }
    }

    pub fn colors(&self) -> &Colors {
        &self.colors
    }

    #[inline]
    fn wrapline(&mut self)
        where
            T: EventListener,
    {
        if !self.mode.contains(TermMode::LINE_WRAP) {
            return;
        }

        trace!("Wrapping input");

        self.grid.cursor_cell().flags.insert(Flags::WRAPLINE);

        if self.grid.cursor.point.line + 1 >= self.scroll_region.end {
            self.linefeed();
        } else {
            self.damage_cursor();
            self.grid.cursor.point.line += 1;
        }

        self.grid.cursor.point.column = Column(0);
        self.grid.cursor.input_needs_wrap = false;
        self.damage_cursor();
    }

    #[inline(always)]
    fn write_at_cursor(&mut self, c: char) {
        let c = self.grid.cursor.charsets[self.active_charset].map(c);
        let fg = self.grid.cursor.template.fg;
        let bg = self.grid.cursor.template.bg;
        let flags = self.grid.cursor.template.flags;
        let extra = self.grid.cursor.template.extra.clone();

        let mut cursor_cell = self.grid.cursor_cell();

        if cursor_cell
            .flags
            .intersects(Flags::WIDE_CHAR | Flags::WIDE_CHAR_SPACER)
        {
            let wide = cursor_cell.flags.contains(Flags::WIDE_CHAR);
            let point = self.grid.cursor.point;
            if wide && point.column < self.last_column() {
                self.grid[point.line][point.column + 1]
                    .flags
                    .remove(Flags::WIDE_CHAR_SPACER);
            } else if point.column > 0 {
                self.grid[point.line][point.column - 1].clear_wide();
            }

            if point.column <= 1 && point.line != self.topmost_line() {
                let column = self.last_column();
                self.grid[point.line - 1i32][column]
                    .flags
                    .remove(Flags::LEADING_WIDE_CHAR_SPACER);
            }

            cursor_cell = self.grid.cursor_cell();
        }

        cursor_cell.c = c;
        cursor_cell.fg = fg;
        cursor_cell.bg = bg;
        cursor_cell.flags = flags;
        cursor_cell.extra = extra;
    }

    #[inline]
    fn damage_cursor(&mut self) {
        let point = Point::new(
            self.grid.cursor.point.line.0 as usize,
            self.grid.cursor.point.column,
        );
        self.damage.damage_point(point);
    }

    #[inline]
    fn set_keyboard_mode(
        &mut self,
        mode: TermMode,
        apply: KeyboardModesApplyBehavior,
    ) {
        let active_mode = self.mode & TermMode::KITTY_KEYBOARD_PROTOCOL;
        self.mode &= !TermMode::KITTY_KEYBOARD_PROTOCOL;
        let new_mode = match apply {
            KeyboardModesApplyBehavior::Replace => mode,
            KeyboardModesApplyBehavior::Union => active_mode.union(mode),
            KeyboardModesApplyBehavior::Difference => {
                active_mode.difference(mode)
            }
        };
        trace!("Setting keyboard mode to {new_mode:?}");
        self.mode |= new_mode;
    }
}

impl<T> Dimensions for Term<T> {
    #[inline]
    fn columns(&self) -> usize {
        self.grid.columns()
    }

    #[inline]
    fn screen_lines(&self) -> usize {
        self.grid.screen_lines()
    }

    #[inline]
    fn total_lines(&self) -> usize {
        self.grid.total_lines()
    }
}

impl<T: EventListener> Handler for Term<T> {
    #[inline(never)]
    fn input(&mut self, c: char) {
        let width = match c.width() {
            Some(width) => width,
            None => return,
        };

        if width == 0 {
            let mut column = self.grid.cursor.point.column;
            if !self.grid.cursor.input_needs_wrap {
                column.0 = column.saturating_sub(1);
            }

            let line = self.grid.cursor.point.line;
            if self.grid[line][column]
                .flags
                .contains(Flags::WIDE_CHAR_SPACER)
            {
                column.0 = column.saturating_sub(1);
            }

            self.grid[line][column].push_zerowidth(c);
            return;
        }

        if self.grid.cursor.input_needs_wrap {
            self.wrapline();
        }

        let columns = self.columns();
        if self.mode.contains(TermMode::INSERT)
            && self.grid.cursor.point.column + width < columns
        {
            let line = self.grid.cursor.point.line;
            let col = self.grid.cursor.point.column;
            let row = &mut self.grid[line][..];

            for col in (col.0..(columns - width)).rev() {
                row.swap(col + width, col);
            }
        }

        if width == 1 {
            self.write_at_cursor(c);
        } else {
            if self.grid.cursor.point.column + 1 >= columns {
                if self.mode.contains(TermMode::LINE_WRAP) {
                    self.grid
                        .cursor
                        .template
                        .flags
                        .insert(Flags::LEADING_WIDE_CHAR_SPACER);
                    self.write_at_cursor(' ');
                    self.grid
                        .cursor
                        .template
                        .flags
                        .remove(Flags::LEADING_WIDE_CHAR_SPACER);
                    self.wrapline();
                } else {
                    self.grid.cursor.input_needs_wrap = true;
                    return;
                }
            }

            self.grid.cursor.template.flags.insert(Flags::WIDE_CHAR);
            self.write_at_cursor(c);
            self.grid.cursor.template.flags.remove(Flags::WIDE_CHAR);

            self.grid.cursor.point.column += 1;
            self.grid
                .cursor
                .template
                .flags
                .insert(Flags::WIDE_CHAR_SPACER);
            self.write_at_cursor(' ');
            self.grid
                .cursor
                .template
                .flags
                .remove(Flags::WIDE_CHAR_SPACER);
        }

        if self.grid.cursor.point.column + 1 < columns {
            self.grid.cursor.point.column += 1;
        } else {
            self.grid.cursor.input_needs_wrap = true;
        }
    }

    #[inline]
    fn decaln(&mut self) {
        trace!("Decalnning");

        for line in (0..self.screen_lines()).map(Line::from) {
            for column in 0..self.columns() {
                let cell = &mut self.grid[line][Column(column)];
                *cell = Cell::default();
                cell.c = 'E';
            }
        }

        self.mark_fully_damaged();
    }

    #[inline]
    fn goto(&mut self, line: i32, col: usize) {
        let line = Line(line);
        let col = Column(col);

        trace!("Going to: line={}, col={}", line, col);
        let (y_offset, max_y) = if self.mode.contains(TermMode::ORIGIN) {
            (self.scroll_region.start, self.scroll_region.end - 1)
        } else {
            (Line(0), self.bottommost_line())
        };

        self.damage_cursor();
        self.grid.cursor.point.line =
            cmp::max(cmp::min(line + y_offset, max_y), Line(0));
        self.grid.cursor.point.column = cmp::min(col, self.last_column());
        self.damage_cursor();
        self.grid.cursor.input_needs_wrap = false;
    }

    #[inline]
    fn goto_line(&mut self, line: i32) {
        trace!("Going to line: {}", line);
        self.goto(line, self.grid.cursor.point.column.0)
    }

    #[inline]
    fn goto_col(&mut self, col: usize) {
        trace!("Going to column: {}", col);
        self.goto(self.grid.cursor.point.line.0, col)
    }

    #[inline]
    fn insert_blank(&mut self, count: usize) {
        let cursor = &self.grid.cursor;
        let bg = cursor.template.bg;

        let count = cmp::min(count, self.columns() - cursor.point.column.0);

        let source = cursor.point.column;
        let destination = cursor.point.column.0 + count;
        let num_cells = self.columns() - destination;

        let line = cursor.point.line;
        self.damage
            .damage_line(line.0 as usize, 0, self.columns() - 1);

        let row = &mut self.grid[line][..];

        for offset in (0..num_cells).rev() {
            row.swap(destination + offset, source.0 + offset);
        }

        for cell in &mut row[source.0..destination] {
            *cell = bg.into();
        }
    }

    #[inline]
    fn move_up(&mut self, lines: usize) {
        trace!("Moving up: {}", lines);

        let line = self.grid.cursor.point.line - lines;
        let column = self.grid.cursor.point.column;
        self.goto(line.0, column.0)
    }

    #[inline]
    fn move_down(&mut self, lines: usize) {
        trace!("Moving down: {}", lines);

        let line = self.grid.cursor.point.line + lines;
        let column = self.grid.cursor.point.column;
        self.goto(line.0, column.0)
    }

    #[inline]
    fn move_forward(&mut self, cols: usize) {
        trace!("Moving forward: {}", cols);
        let last_column =
            cmp::min(self.grid.cursor.point.column + cols, self.last_column());

        let cursor_line = self.grid.cursor.point.line.0 as usize;
        self.damage.damage_line(
            cursor_line,
            self.grid.cursor.point.column.0,
            last_column.0,
        );

        self.grid.cursor.point.column = last_column;
        self.grid.cursor.input_needs_wrap = false;
    }

    #[inline]
    fn move_backward(&mut self, cols: usize) {
        trace!("Moving backward: {}", cols);
        let column = self.grid.cursor.point.column.saturating_sub(cols);

        let cursor_line = self.grid.cursor.point.line.0 as usize;
        self.damage.damage_line(
            cursor_line,
            column,
            self.grid.cursor.point.column.0,
        );

        self.grid.cursor.point.column = Column(column);
        self.grid.cursor.input_needs_wrap = false;
    }

    #[inline]
    fn identify_terminal(&mut self, intermediate: Option<char>) {
        match intermediate {
            None => {
                trace!("Reporting primary device attributes");
                let text = String::from("\x1b[?6c");
                self.event_proxy.send_event(Event::PtyWrite(text));
            }
            Some('>') => {
                trace!("Reporting secondary device attributes");
                let version = version_number(env!("CARGO_PKG_VERSION"));
                let text = format!("\x1b[>0;{version};1c");
                self.event_proxy.send_event(Event::PtyWrite(text));
            }
            _ => debug!("Unsupported device attributes intermediate"),
        }
    }

    #[inline]
    fn report_keyboard_mode(&mut self) {
        if !self.config.kitty_keyboard {
            return;
        }

        trace!("Reporting active keyboard mode");
        let current_mode = self
            .keyboard_mode_stack
            .last()
            .unwrap_or(&KeyboardModes::NO_MODE)
            .bits();
        let text = format!("\x1b[?{current_mode}u");
        self.event_proxy.send_event(Event::PtyWrite(text));
    }

    #[inline]
    fn push_keyboard_mode(&mut self, mode: KeyboardModes) {
        if !self.config.kitty_keyboard {
            return;
        }

        trace!("Pushing `{mode:?}` keyboard mode into the stack");

        if self.keyboard_mode_stack.len() >= KEYBOARD_MODE_STACK_MAX_DEPTH {
            let removed = self.title_stack.remove(0);
            trace!(
                "Removing '{:?}' from bottom of keyboard mode stack that exceeds its maximum depth",
                removed
            );
        }

        self.keyboard_mode_stack.push(mode);
        self.set_keyboard_mode(
            mode.into(),
            KeyboardModesApplyBehavior::Replace,
        );
    }

    #[inline]
    fn pop_keyboard_modes(&mut self, to_pop: u16) {
        if !self.config.kitty_keyboard {
            return;
        }

        trace!("Attempting to pop {to_pop} keyboard modes from the stack");
        let new_len = self
            .keyboard_mode_stack
            .len()
            .saturating_sub(to_pop as usize);
        self.keyboard_mode_stack.truncate(new_len);

        let mode = self
            .keyboard_mode_stack
            .last()
            .copied()
            .unwrap_or(KeyboardModes::NO_MODE);
        self.set_keyboard_mode(
            mode.into(),
            KeyboardModesApplyBehavior::Replace,
        );
    }

    #[inline]
    fn set_keyboard_mode(
        &mut self,
        mode: KeyboardModes,
        apply: KeyboardModesApplyBehavior,
    ) {
        if !self.config.kitty_keyboard {
            return;
        }

        self.set_keyboard_mode(mode.into(), apply);
    }

    #[inline]
    fn device_status(&mut self, arg: usize) {
        trace!("Reporting device status: {}", arg);
        match arg {
            5 => {
                let text = String::from("\x1b[0n");
                self.event_proxy.send_event(Event::PtyWrite(text));
            }
            6 => {
                let pos = self.grid.cursor.point;
                let text = format!("\x1b[{};{}R", pos.line + 1, pos.column + 1);
                self.event_proxy.send_event(Event::PtyWrite(text));
            }
            _ => debug!("unknown device status query: {}", arg),
        };
    }

    #[inline]
    fn move_down_and_cr(&mut self, lines: usize) {
        trace!("Moving down and cr: {}", lines);

        let line = self.grid.cursor.point.line + lines;
        self.goto(line.0, 0)
    }

    #[inline]
    fn move_up_and_cr(&mut self, lines: usize) {
        trace!("Moving up and cr: {}", lines);

        let line = self.grid.cursor.point.line - lines;
        self.goto(line.0, 0)
    }

    #[inline]
    fn put_tab(&mut self, mut count: u16) {
        if self.grid.cursor.input_needs_wrap {
            self.wrapline();
            return;
        }

        while self.grid.cursor.point.column < self.columns() && count != 0 {
            count -= 1;

            let c = self.grid.cursor.charsets[self.active_charset].map('\t');
            let cell = self.grid.cursor_cell();
            if cell.c == ' ' {
                cell.c = c;
            }

            loop {
                if (self.grid.cursor.point.column + 1) == self.columns() {
                    break;
                }

                self.grid.cursor.point.column += 1;

                if self.tabs[self.grid.cursor.point.column] {
                    break;
                }
            }
        }
    }

    #[inline]
    fn backspace(&mut self) {
        trace!("Backspace");

        if self.grid.cursor.point.column > Column(0) {
            let line = self.grid.cursor.point.line.0 as usize;
            let column = self.grid.cursor.point.column.0;
            self.grid.cursor.point.column -= 1;
            self.grid.cursor.input_needs_wrap = false;
            self.damage.damage_line(line, column - 1, column);
        }
    }

    #[inline]
    fn carriage_return(&mut self) {
        trace!("Carriage return");
        let new_col = 0;
        let line = self.grid.cursor.point.line.0 as usize;
        self.damage
            .damage_line(line, new_col, self.grid.cursor.point.column.0);
        self.grid.cursor.point.column = Column(new_col);
        self.grid.cursor.input_needs_wrap = false;
    }

    #[inline]
    fn linefeed(&mut self) {
        trace!("Linefeed");
        let next = self.grid.cursor.point.line + 1;
        if next == self.scroll_region.end {
            self.scroll_up(1);
        } else if next < self.screen_lines() {
            self.damage_cursor();
            self.grid.cursor.point.line += 1;
            self.damage_cursor();
        }
    }

    #[inline]
    fn bell(&mut self) {
        trace!("Bell");
        self.event_proxy.send_event(Event::Bell);
    }

    #[inline]
    fn substitute(&mut self) {
        trace!("[unimplemented] Substitute");
    }

    #[inline]
    fn newline(&mut self) {
        self.linefeed();

        if self.mode.contains(TermMode::LINE_FEED_NEW_LINE) {
            self.carriage_return();
        }
    }

    #[inline]
    fn set_horizontal_tabstop(&mut self) {
        trace!("Setting horizontal tabstop");
        self.tabs[self.grid.cursor.point.column] = true;
    }

    #[inline]
    fn scroll_up(&mut self, lines: usize) {
        let origin = self.scroll_region.start;
        self.scroll_up_relative(origin, lines);
    }

    #[inline]
    fn scroll_down(&mut self, lines: usize) {
        let origin = self.scroll_region.start;
        self.scroll_down_relative(origin, lines);
    }

    #[inline]
    fn insert_blank_lines(&mut self, lines: usize) {
        trace!("Inserting blank {} lines", lines);

        let origin = self.grid.cursor.point.line;
        if self.scroll_region.contains(&origin) {
            self.scroll_down_relative(origin, lines);
        }
    }

    #[inline]
    fn delete_lines(&mut self, lines: usize) {
        let origin = self.grid.cursor.point.line;
        let lines = cmp::min(self.screen_lines() - origin.0 as usize, lines);

        trace!("Deleting {} lines", lines);

        if lines > 0 && self.scroll_region.contains(&origin) {
            self.scroll_up_relative(origin, lines);
        }
    }

    #[inline]
    fn erase_chars(&mut self, count: usize) {
        let cursor = &self.grid.cursor;

        trace!(
            "Erasing chars: count={}, col={}",
            count,
            cursor.point.column
        );

        let start = cursor.point.column;
        let end = cmp::min(start + count, Column(self.columns()));

        let bg = self.grid.cursor.template.bg;
        let line = cursor.point.line;
        self.damage.damage_line(line.0 as usize, start.0, end.0);
        let row = &mut self.grid[line];
        for cell in &mut row[start..end] {
            *cell = bg.into();
        }
    }

    #[inline]
    fn delete_chars(&mut self, count: usize) {
        let columns = self.columns();
        let cursor = &self.grid.cursor;
        let bg = cursor.template.bg;

        let count = cmp::min(count, columns);

        let start = cursor.point.column.0;
        let end = cmp::min(start + count, columns - 1);
        let num_cells = columns - end;

        let line = cursor.point.line;
        self.damage
            .damage_line(line.0 as usize, 0, self.columns() - 1);
        let row = &mut self.grid[line][..];

        for offset in 0..num_cells {
            row.swap(start + offset, end + offset);
        }

        let end = columns - count;
        for cell in &mut row[end..] {
            *cell = bg.into();
        }
    }

    #[inline]
    fn move_backward_tabs(&mut self, count: u16) {
        trace!("Moving backward {} tabs", count);
        self.damage_cursor();

        let old_col = self.grid.cursor.point.column.0;
        for _ in 0..count {
            let mut col = self.grid.cursor.point.column;
            for i in (0..(col.0)).rev() {
                if self.tabs[index::Column(i)] {
                    col = index::Column(i);
                    break;
                }
            }
            self.grid.cursor.point.column = col;
        }

        let line = self.grid.cursor.point.line.0 as usize;
        self.damage
            .damage_line(line, self.grid.cursor.point.column.0, old_col);
    }

    #[inline]
    fn move_forward_tabs(&mut self, count: u16) {
        trace!("[unimplemented] Moving forward {} tabs", count);
    }

    #[inline]
    fn save_cursor_position(&mut self) {
        trace!("Saving cursor position");

        self.grid.saved_cursor = self.grid.cursor.clone();
    }

    #[inline]
    fn restore_cursor_position(&mut self) {
        trace!("Restoring cursor position");

        self.damage_cursor();
        self.grid.cursor = self.grid.saved_cursor.clone();
        self.damage_cursor();
    }

    #[inline]
    fn clear_line(&mut self, mode: ansi::LineClearMode) {
        trace!("Clearing line: {:?}", mode);

        let cursor = &self.grid.cursor;
        let bg = cursor.template.bg;
        let point = cursor.point;

        let (left, right) = match mode {
            ansi::LineClearMode::Right if cursor.input_needs_wrap => return,
            ansi::LineClearMode::Right => {
                (point.column, Column(self.columns()))
            }
            ansi::LineClearMode::Left => (Column(0), point.column + 1),
            ansi::LineClearMode::All => (Column(0), Column(self.columns())),
        };

        self.damage
            .damage_line(point.line.0 as usize, left.0, right.0 - 1);

        let row = &mut self.grid[point.line];
        for cell in &mut row[left..right] {
            *cell = bg.into();
        }

        let range = self.grid.cursor.point.line..=self.grid.cursor.point.line;
        self.selection =
            self.selection.take().filter(|s| !s.intersects_range(range));
    }

    #[inline]
    fn set_color(&mut self, index: usize, color: Rgb) {
        trace!("Setting color[{}] = {:?}", index, color);

        if index != NamedColor::Cursor as usize
            && self.colors[index] != Some(color)
        {
            self.mark_fully_damaged();
        }

        self.colors[index] = Some(color);
    }

    #[inline]
    fn dynamic_color_sequence(
        &mut self,
        prefix: String,
        index: usize,
        terminator: &str,
    ) {
        trace!(
            "Requested write of escape sequence for color code {}: color[{}]",
            prefix,
            index
        );

        let terminator = terminator.to_owned();
        self.event_proxy.send_event(Event::ColorRequest(
            index,
            Arc::new(move |color| {
                format!(
                    "\x1b]{};rgb:{1:02x}{1:02x}/{2:02x}{2:02x}/{3:02x}{3:02x}{4}",
                    prefix, color.r, color.g, color.b, terminator
                )
            }),
        ));
    }

    #[inline]
    fn reset_color(&mut self, index: usize) {
        trace!("Resetting color[{}]", index);

        if index != NamedColor::Cursor as usize && self.colors[index].is_some()
        {
            self.mark_fully_damaged();
        }

        self.colors[index] = None;
    }

    #[inline]
    fn clipboard_store(&mut self, clipboard: u8, base64: &[u8]) {
        if !matches!(self.config.osc52, Osc52::OnlyCopy | Osc52::CopyPaste) {
            debug!("Denied osc52 store");
            return;
        }

        let clipboard_type = match clipboard {
            b'c' => ClipboardType::Clipboard,
            b'p' | b's' => ClipboardType::Selection,
            _ => return,
        };

        if let Ok(bytes) = Base64.decode(base64) {
            if let Ok(text) = String::from_utf8(bytes) {
                self.event_proxy
                    .send_event(Event::ClipboardStore(clipboard_type, text));
            }
        }
    }

    #[inline]
    fn clipboard_load(&mut self, clipboard: u8, terminator: &str) {
        if !matches!(self.config.osc52, Osc52::OnlyPaste | Osc52::CopyPaste) {
            debug!("Denied osc52 load");
            return;
        }

        let clipboard_type = match clipboard {
            b'c' => ClipboardType::Clipboard,
            b'p' | b's' => ClipboardType::Selection,
            _ => return,
        };

        let terminator = terminator.to_owned();

        self.event_proxy.send_event(Event::ClipboardLoad(
            clipboard_type,
            Arc::new(move |text| {
                let base64 = Base64.encode(text);
                format!(
                    "\x1b]52;{};{}{}",
                    clipboard as char, base64, terminator
                )
            }),
        ));
    }

    #[inline]
    fn clear_screen(&mut self, mode: ansi::ClearMode) {
        trace!("Clearing screen: {:?}", mode);
        let bg = self.grid.cursor.template.bg;

        let screen_lines = self.screen_lines();

        match mode {
            ansi::ClearMode::Above => {
                let cursor = self.grid.cursor.point;

                if cursor.line > 1 {
                    self.grid.reset_region(..cursor.line);
                }

                let end = cmp::min(cursor.column + 1, Column(self.columns()));
                for cell in &mut self.grid[cursor.line][..end] {
                    *cell = bg.into();
                }

                let range = Line(0)..=cursor.line;
                self.selection = self
                    .selection
                    .take()
                    .filter(|s| !s.intersects_range(range));
            }
            ansi::ClearMode::Below => {
                let cursor = self.grid.cursor.point;
                for cell in &mut self.grid[cursor.line][cursor.column..] {
                    *cell = bg.into();
                }

                if (cursor.line.0 as usize) < screen_lines - 1 {
                    self.grid.reset_region((cursor.line + 1)..);
                }

                let range = cursor.line..Line(screen_lines as i32);
                self.selection = self
                    .selection
                    .take()
                    .filter(|s| !s.intersects_range(range));
            }
            ansi::ClearMode::All => {
                if self.mode.contains(TermMode::ALT_SCREEN) {
                    self.grid.reset_region(..);
                } else {
                    let old_offset = self.grid.display_offset();

                    self.grid.clear_viewport();

                    let lines =
                        self.grid.display_offset().saturating_sub(old_offset);

                    self.vi_mode_cursor.point.line =
                        (self.vi_mode_cursor.point.line - lines)
                            .grid_clamp(self, Boundary::Grid);
                }

                self.selection = None;
            }
            ansi::ClearMode::Saved if self.history_size() > 0 => {
                self.grid.clear_history();

                self.vi_mode_cursor.point.line = self
                    .vi_mode_cursor
                    .point
                    .line
                    .grid_clamp(self, Boundary::Cursor);

                self.selection = self
                    .selection
                    .take()
                    .filter(|s| !s.intersects_range(..Line(0)));
            }
            ansi::ClearMode::Saved => (),
        }

        self.mark_fully_damaged();
    }

    #[inline]
    fn clear_tabs(&mut self, mode: ansi::TabulationClearMode) {
        trace!("Clearing tabs: {:?}", mode);
        match mode {
            ansi::TabulationClearMode::Current => {
                self.tabs[self.grid.cursor.point.column] = false;
            }
            ansi::TabulationClearMode::All => {
                self.tabs.clear_all();
            }
        }
    }

    #[inline]
    fn reset_state(&mut self) {
        if self.mode.contains(TermMode::ALT_SCREEN) {
            mem::swap(&mut self.grid, &mut self.inactive_grid);
        }
        self.active_charset = Default::default();
        self.cursor_style = None;
        self.grid.reset();
        self.inactive_grid.reset();
        self.scroll_region = Line(0)..Line(self.screen_lines() as i32);
        self.tabs = TabStops::new(self.columns());
        self.title_stack = Vec::new();
        self.title = None;
        self.selection = None;
        self.vi_mode_cursor = Default::default();
        self.keyboard_mode_stack = Default::default();
        self.inactive_keyboard_mode_stack = Default::default();

        self.mode &= TermMode::VI;
        self.mode.insert(TermMode::default());

        self.event_proxy.send_event(Event::CursorBlinkingChange);
        self.mark_fully_damaged();
    }

    #[inline]
    fn reverse_index(&mut self) {
        trace!("Reversing index");
        if self.grid.cursor.point.line == self.scroll_region.start {
            self.scroll_down(1);
        } else {
            self.damage_cursor();
            self.grid.cursor.point.line =
                cmp::max(self.grid.cursor.point.line - 1, Line(0));
            self.damage_cursor();
        }
    }

    #[inline]
    fn set_hyperlink(&mut self, hyperlink: Option<Hyperlink>) {
        trace!("Setting hyperlink: {:?}", hyperlink);
        self.grid
            .cursor
            .template
            .set_hyperlink(hyperlink.map(|e| e.into()));
    }

    #[inline]
    fn terminal_attribute(&mut self, attr: Attr) {
        trace!("Setting attribute: {:?}", attr);
        let cursor = &mut self.grid.cursor;
        match attr {
            Attr::Foreground(color) => cursor.template.fg = color,
            Attr::Background(color) => cursor.template.bg = color,
            Attr::UnderlineColor(color) => {
                cursor.template.set_underline_color(color)
            }
            Attr::Reset => {
                cursor.template.fg = Color::Named(NamedColor::Foreground);
                cursor.template.bg = Color::Named(NamedColor::Background);
                cursor.template.flags = Flags::empty();
                cursor.template.set_underline_color(None);
            }
            Attr::Reverse => cursor.template.flags.insert(Flags::INVERSE),
            Attr::CancelReverse => cursor.template.flags.remove(Flags::INVERSE),
            Attr::Bold => cursor.template.flags.insert(Flags::BOLD),
            Attr::CancelBold => cursor.template.flags.remove(Flags::BOLD),
            Attr::Dim => cursor.template.flags.insert(Flags::DIM),
            Attr::CancelBoldDim => {
                cursor.template.flags.remove(Flags::BOLD | Flags::DIM)
            }
            Attr::Italic => cursor.template.flags.insert(Flags::ITALIC),
            Attr::CancelItalic => cursor.template.flags.remove(Flags::ITALIC),
            Attr::Underline => {
                cursor.template.flags.remove(Flags::ALL_UNDERLINES);
                cursor.template.flags.insert(Flags::UNDERLINE);
            }
            Attr::DoubleUnderline => {
                cursor.template.flags.remove(Flags::ALL_UNDERLINES);
                cursor.template.flags.insert(Flags::DOUBLE_UNDERLINE);
            }
            Attr::Undercurl => {
                cursor.template.flags.remove(Flags::ALL_UNDERLINES);
                cursor.template.flags.insert(Flags::UNDERCURL);
            }
            Attr::DottedUnderline => {
                cursor.template.flags.remove(Flags::ALL_UNDERLINES);
                cursor.template.flags.insert(Flags::DOTTED_UNDERLINE);
            }
            Attr::DashedUnderline => {
                cursor.template.flags.remove(Flags::ALL_UNDERLINES);
                cursor.template.flags.insert(Flags::DASHED_UNDERLINE);
            }
            Attr::CancelUnderline => {
                cursor.template.flags.remove(Flags::ALL_UNDERLINES)
            }
            Attr::Hidden => cursor.template.flags.insert(Flags::HIDDEN),
            Attr::CancelHidden => cursor.template.flags.remove(Flags::HIDDEN),
            Attr::Strike => cursor.template.flags.insert(Flags::STRIKEOUT),
            Attr::CancelStrike => {
                cursor.template.flags.remove(Flags::STRIKEOUT)
            }
            _ => {
                debug!("Term got unhandled attr: {:?}", attr);
            }
        }
    }

    #[inline]
    fn set_private_mode(&mut self, mode: PrivateMode) {
        let mode = match mode {
            PrivateMode::Named(mode) => mode,
            PrivateMode::Unknown(mode) => {
                debug!("Ignoring unknown mode {} in set_private_mode", mode);
                return;
            }
        };

        trace!("Setting private mode: {:?}", mode);
        match mode {
            NamedPrivateMode::UrgencyHints => {
                self.mode.insert(TermMode::URGENCY_HINTS)
            }
            NamedPrivateMode::SwapScreenAndSetRestoreCursor => {
                if !self.mode.contains(TermMode::ALT_SCREEN) {
                    self.swap_alt();
                }
            }
            NamedPrivateMode::ShowCursor => {
                self.mode.insert(TermMode::SHOW_CURSOR)
            }
            NamedPrivateMode::CursorKeys => {
                self.mode.insert(TermMode::APP_CURSOR)
            }
            NamedPrivateMode::ReportMouseClicks => {
                self.mode.remove(TermMode::MOUSE_MODE);
                self.mode.insert(TermMode::MOUSE_REPORT_CLICK);
                self.event_proxy.send_event(Event::MouseCursorDirty);
            }
            NamedPrivateMode::ReportCellMouseMotion => {
                self.mode.remove(TermMode::MOUSE_MODE);
                self.mode.insert(TermMode::MOUSE_DRAG);
                self.event_proxy.send_event(Event::MouseCursorDirty);
            }
            NamedPrivateMode::ReportAllMouseMotion => {
                self.mode.remove(TermMode::MOUSE_MODE);
                self.mode.insert(TermMode::MOUSE_MOTION);
                self.event_proxy.send_event(Event::MouseCursorDirty);
            }
            NamedPrivateMode::ReportFocusInOut => {
                self.mode.insert(TermMode::FOCUS_IN_OUT)
            }
            NamedPrivateMode::BracketedPaste => {
                self.mode.insert(TermMode::BRACKETED_PASTE)
            }
            NamedPrivateMode::SgrMouse => {
                self.mode.remove(TermMode::UTF8_MOUSE);
                self.mode.insert(TermMode::SGR_MOUSE);
            }
            NamedPrivateMode::Utf8Mouse => {
                self.mode.remove(TermMode::SGR_MOUSE);
                self.mode.insert(TermMode::UTF8_MOUSE);
            }
            NamedPrivateMode::AlternateScroll => {
                self.mode.insert(TermMode::ALTERNATE_SCROLL)
            }
            NamedPrivateMode::LineWrap => self.mode.insert(TermMode::LINE_WRAP),
            NamedPrivateMode::Origin => self.mode.insert(TermMode::ORIGIN),
            NamedPrivateMode::ColumnMode => self.deccolm(),
            NamedPrivateMode::BlinkingCursor => {
                let style = self
                    .cursor_style
                    .get_or_insert(self.config.default_cursor_style);
                style.blinking = true;
                self.event_proxy.send_event(Event::CursorBlinkingChange);
            }
            NamedPrivateMode::SyncUpdate => (),
        }
    }

    #[inline]
    fn unset_private_mode(&mut self, mode: PrivateMode) {
        let mode = match mode {
            PrivateMode::Named(mode) => mode,
            PrivateMode::Unknown(mode) => {
                debug!("Ignoring unknown mode {} in unset_private_mode", mode);
                return;
            }
        };

        trace!("Unsetting private mode: {:?}", mode);
        match mode {
            NamedPrivateMode::UrgencyHints => {
                self.mode.remove(TermMode::URGENCY_HINTS)
            }
            NamedPrivateMode::SwapScreenAndSetRestoreCursor => {
                if self.mode.contains(TermMode::ALT_SCREEN) {
                    self.swap_alt();
                }
            }
            NamedPrivateMode::ShowCursor => {
                self.mode.remove(TermMode::SHOW_CURSOR)
            }
            NamedPrivateMode::CursorKeys => {
                self.mode.remove(TermMode::APP_CURSOR)
            }
            NamedPrivateMode::ReportMouseClicks => {
                self.mode.remove(TermMode::MOUSE_REPORT_CLICK);
                self.event_proxy.send_event(Event::MouseCursorDirty);
            }
            NamedPrivateMode::ReportCellMouseMotion => {
                self.mode.remove(TermMode::MOUSE_DRAG);
                self.event_proxy.send_event(Event::MouseCursorDirty);
            }
            NamedPrivateMode::ReportAllMouseMotion => {
                self.mode.remove(TermMode::MOUSE_MOTION);
                self.event_proxy.send_event(Event::MouseCursorDirty);
            }
            NamedPrivateMode::ReportFocusInOut => {
                self.mode.remove(TermMode::FOCUS_IN_OUT)
            }
            NamedPrivateMode::BracketedPaste => {
                self.mode.remove(TermMode::BRACKETED_PASTE)
            }
            NamedPrivateMode::SgrMouse => self.mode.remove(TermMode::SGR_MOUSE),
            NamedPrivateMode::Utf8Mouse => {
                self.mode.remove(TermMode::UTF8_MOUSE)
            }
            NamedPrivateMode::AlternateScroll => {
                self.mode.remove(TermMode::ALTERNATE_SCROLL)
            }
            NamedPrivateMode::LineWrap => self.mode.remove(TermMode::LINE_WRAP),
            NamedPrivateMode::Origin => self.mode.remove(TermMode::ORIGIN),
            NamedPrivateMode::ColumnMode => self.deccolm(),
            NamedPrivateMode::BlinkingCursor => {
                let style = self
                    .cursor_style
                    .get_or_insert(self.config.default_cursor_style);
                style.blinking = false;
                self.event_proxy.send_event(Event::CursorBlinkingChange);
            }
            NamedPrivateMode::SyncUpdate => (),
        }
    }

    #[inline]
    fn report_private_mode(&mut self, mode: PrivateMode) {
        trace!("Reporting private mode {mode:?}");
        let state = match mode {
            PrivateMode::Named(mode) => match mode {
                NamedPrivateMode::CursorKeys => {
                    self.mode.contains(TermMode::APP_CURSOR).into()
                }
                NamedPrivateMode::Origin => {
                    self.mode.contains(TermMode::ORIGIN).into()
                }
                NamedPrivateMode::LineWrap => {
                    self.mode.contains(TermMode::LINE_WRAP).into()
                }
                NamedPrivateMode::BlinkingCursor => {
                    let style = self
                        .cursor_style
                        .get_or_insert(self.config.default_cursor_style);
                    style.blinking.into()
                }
                NamedPrivateMode::ShowCursor => {
                    self.mode.contains(TermMode::SHOW_CURSOR).into()
                }
                NamedPrivateMode::ReportMouseClicks => {
                    self.mode.contains(TermMode::MOUSE_REPORT_CLICK).into()
                }
                NamedPrivateMode::ReportCellMouseMotion => {
                    self.mode.contains(TermMode::MOUSE_DRAG).into()
                }
                NamedPrivateMode::ReportAllMouseMotion => {
                    self.mode.contains(TermMode::MOUSE_MOTION).into()
                }
                NamedPrivateMode::ReportFocusInOut => {
                    self.mode.contains(TermMode::FOCUS_IN_OUT).into()
                }
                NamedPrivateMode::Utf8Mouse => {
                    self.mode.contains(TermMode::UTF8_MOUSE).into()
                }
                NamedPrivateMode::SgrMouse => {
                    self.mode.contains(TermMode::SGR_MOUSE).into()
                }
                NamedPrivateMode::AlternateScroll => {
                    self.mode.contains(TermMode::ALTERNATE_SCROLL).into()
                }
                NamedPrivateMode::UrgencyHints => {
                    self.mode.contains(TermMode::URGENCY_HINTS).into()
                }
                NamedPrivateMode::SwapScreenAndSetRestoreCursor => {
                    self.mode.contains(TermMode::ALT_SCREEN).into()
                }
                NamedPrivateMode::BracketedPaste => {
                    self.mode.contains(TermMode::BRACKETED_PASTE).into()
                }
                NamedPrivateMode::SyncUpdate => ModeState::Reset,
                NamedPrivateMode::ColumnMode => ModeState::NotSupported,
            },
            PrivateMode::Unknown(_) => ModeState::NotSupported,
        };

        self.event_proxy.send_event(Event::PtyWrite(format!(
            "\x1b[?{};{}$y",
            mode.raw(),
            state as u8,
        )));
    }

    #[inline]
    fn set_mode(&mut self, mode: ansi::Mode) {
        let mode = match mode {
            ansi::Mode::Named(mode) => mode,
            ansi::Mode::Unknown(mode) => {
                debug!("Ignoring unknown mode {} in set_mode", mode);
                return;
            }
        };

        trace!("Setting public mode: {:?}", mode);
        match mode {
            NamedMode::Insert => self.mode.insert(TermMode::INSERT),
            NamedMode::LineFeedNewLine => {
                self.mode.insert(TermMode::LINE_FEED_NEW_LINE)
            }
        }
    }

    #[inline]
    fn unset_mode(&mut self, mode: ansi::Mode) {
        let mode = match mode {
            ansi::Mode::Named(mode) => mode,
            ansi::Mode::Unknown(mode) => {
                debug!("Ignorning unknown mode {} in unset_mode", mode);
                return;
            }
        };

        trace!("Setting public mode: {:?}", mode);
        match mode {
            NamedMode::Insert => {
                self.mode.remove(TermMode::INSERT);
                self.mark_fully_damaged();
            }
            NamedMode::LineFeedNewLine => {
                self.mode.remove(TermMode::LINE_FEED_NEW_LINE)
            }
        }
    }

    #[inline]
    fn report_mode(&mut self, mode: ansi::Mode) {
        trace!("Reporting mode {mode:?}");
        let state = match mode {
            ansi::Mode::Named(mode) => match mode {
                NamedMode::Insert => {
                    self.mode.contains(TermMode::INSERT).into()
                }
                NamedMode::LineFeedNewLine => {
                    self.mode.contains(TermMode::LINE_FEED_NEW_LINE).into()
                }
            },
            ansi::Mode::Unknown(_) => ModeState::NotSupported,
        };

        self.event_proxy.send_event(Event::PtyWrite(format!(
            "\x1b[{};{}$y",
            mode.raw(),
            state as u8,
        )));
    }

    #[inline]
    fn set_scrolling_region(&mut self, top: usize, bottom: Option<usize>) {
        let bottom = bottom.unwrap_or_else(|| self.screen_lines());

        if top >= bottom {
            debug!("Invalid scrolling region: ({};{})", top, bottom);
            return;
        }

        let start = Line(top as i32 - 1);
        let end = Line(bottom as i32);

        trace!("Setting scrolling region: ({};{})", start, end);

        let screen_lines = Line(self.screen_lines() as i32);
        self.scroll_region.start = cmp::min(start, screen_lines);
        self.scroll_region.end = cmp::min(end, screen_lines);
        self.goto(0, 0);
    }

    #[inline]
    fn set_keypad_application_mode(&mut self) {
        trace!("Setting keypad application mode");
        self.mode.insert(TermMode::APP_KEYPAD);
    }

    #[inline]
    fn unset_keypad_application_mode(&mut self) {
        trace!("Unsetting keypad application mode");
        self.mode.remove(TermMode::APP_KEYPAD);
    }

    #[inline]
    fn configure_charset(
        &mut self,
        index: CharsetIndex,
        charset: StandardCharset,
    ) {
        trace!("Configuring charset {:?} as {:?}", index, charset);
        self.grid.cursor.charsets[index] = charset;
    }

    #[inline]
    fn set_active_charset(&mut self, index: CharsetIndex) {
        trace!("Setting active charset {:?}", index);
        self.active_charset = index;
    }

    #[inline]
    fn set_cursor_style(&mut self, style: Option<CursorStyle>) {
        trace!("Setting cursor style {:?}", style);
        self.cursor_style = style;

        self.event_proxy.send_event(Event::CursorBlinkingChange);
    }

    #[inline]
    fn set_cursor_shape(&mut self, shape: CursorShape) {
        trace!("Setting cursor shape {:?}", shape);

        let style = self
            .cursor_style
            .get_or_insert(self.config.default_cursor_style);
        style.shape = shape;
    }

    #[inline]
    fn set_title(&mut self, title: Option<String>) {
        trace!("Setting title to '{:?}'", title);

        self.title = title.clone();

        let title_event = match title {
            Some(title) => Event::Title(title),
            None => Event::ResetTitle,
        };

        self.event_proxy.send_event(title_event);
    }

    #[inline]
    fn push_title(&mut self) {
        trace!("Pushing '{:?}' onto title stack", self.title);

        if self.title_stack.len() >= TITLE_STACK_MAX_DEPTH {
            let removed = self.title_stack.remove(0);
            trace!(
                "Removing '{:?}' from bottom of title stack that exceeds its maximum depth",
                removed
            );
        }

        self.title_stack.push(self.title.clone());
    }

    #[inline]
    fn pop_title(&mut self) {
        trace!("Attempting to pop title from stack...");

        if let Some(popped) = self.title_stack.pop() {
            trace!("Title '{:?}' popped from stack", popped);
            self.set_title(popped);
        }
    }

    #[inline]
    fn text_area_size_pixels(&mut self) {
        self.event_proxy
            .send_event(Event::TextAreaSizeRequest(Arc::new(
                move |window_size| {
                    let height =
                        window_size.num_lines * window_size.cell_height;
                    let width = window_size.num_cols * window_size.cell_width;
                    format!("\x1b[4;{height};{width}t")
                },
            )));
    }

    #[inline]
    fn text_area_size_chars(&mut self) {
        let text =
            format!("\x1b[8;{};{}t", self.screen_lines(), self.columns());
        self.event_proxy.send_event(Event::PtyWrite(text));
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
enum ModeState {
    NotSupported = 0,
    Set = 1,
    Reset = 2,
}

impl From<bool> for ModeState {
    fn from(value: bool) -> Self {
        if value {
            Self::Set
        } else {
            Self::Reset
        }
    }
}

fn version_number(mut version: &str) -> usize {
    if let Some(separator) = version.rfind('-') {
        version = &version[..separator];
    }

    let mut version_number = 0;

    let semver_versions = version.split('.');
    for (i, semver_version) in semver_versions.rev().enumerate() {
        let semver_number = semver_version.parse::<usize>().unwrap_or(0);
        version_number += usize::pow(100, i as u32) * semver_number;
    }

    version_number
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardType {
    Clipboard,
    Selection,
}

struct TabStops {
    tabs: Vec<bool>,
}

impl TabStops {
    #[inline]
    fn new(columns: usize) -> TabStops {
        TabStops {
            tabs: (0..columns).map(|i| i % INITIAL_TABSTOPS == 0).collect(),
        }
    }

    #[inline]
    fn clear_all(&mut self) {
        unsafe {
            ptr::write_bytes(self.tabs.as_mut_ptr(), 0, self.tabs.len());
        }
    }

    #[inline]
    fn resize(&mut self, columns: usize) {
        let mut index = self.tabs.len();
        self.tabs.resize_with(columns, || {
            let is_tabstop = index % INITIAL_TABSTOPS == 0;
            index += 1;
            is_tabstop
        });
    }
}

impl Index<Column> for TabStops {
    type Output = bool;

    fn index(&self, index: Column) -> &bool {
        &self.tabs[index.0]
    }
}

impl IndexMut<Column> for TabStops {
    fn index_mut(&mut self, index: Column) -> &mut bool {
        self.tabs.index_mut(index.0)
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct RenderableCursor {
    pub shape: CursorShape,
    pub point: Point,
}

impl RenderableCursor {
    fn new<T>(term: &Term<T>) -> Self {
        let vi_mode = term.mode().contains(TermMode::VI);
        let mut point = if vi_mode {
            term.vi_mode_cursor.point
        } else {
            term.grid.cursor.point
        };
        if term.grid[point].flags.contains(Flags::WIDE_CHAR_SPACER) {
            point.column -= 1;
        }

        let shape = if !vi_mode && !term.mode().contains(TermMode::SHOW_CURSOR)
        {
            CursorShape::Hidden
        } else {
            term.cursor_style().shape
        };

        Self { shape, point }
    }
}

pub struct RenderableContent<'a> {
    pub display_iter: GridIterator<'a, Cell>,
    pub selection: Option<SelectionRange>,
    pub cursor: RenderableCursor,
    pub display_offset: usize,
    pub colors: &'a color::Colors,
    pub mode: TermMode,
}

impl<'a> RenderableContent<'a> {
    fn new<T>(term: &'a Term<T>) -> Self {
        Self {
            display_iter: term.grid().display_iter(),
            display_offset: term.grid().display_offset(),
            cursor: RenderableCursor::new(term),
            selection: term.selection.as_ref().and_then(|s| s.to_range(term)),
            colors: &term.colors,
            mode: *term.mode(),
        }
    }
}

pub mod test {
    #[cfg(feature = "serde")]
    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::event::VoidListener;

    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct TermSize {
        pub columns: usize,
        pub screen_lines: usize,
    }

    impl TermSize {
        pub fn new(columns: usize, screen_lines: usize) -> Self {
            Self {
                columns,
                screen_lines,
            }
        }
    }

    impl Dimensions for TermSize {
        fn total_lines(&self) -> usize {
            self.screen_lines()
        }

        fn screen_lines(&self) -> usize {
            self.screen_lines
        }

        fn columns(&self) -> usize {
            self.columns
        }
    }

    pub fn mock_term(content: &str) -> Term<VoidListener> {
        let lines: Vec<&str> = content.split('\n').collect();
        let num_cols = lines
            .iter()
            .map(|line| {
                line.chars()
                    .filter(|c| *c != '\r')
                    .map(|c| c.width().unwrap())
                    .sum()
            })
            .max()
            .unwrap_or(0);

        let size = TermSize::new(num_cols, lines.len());
        let mut term = Term::new(Config::default(), &size, VoidListener);

        for (line, text) in lines.iter().enumerate() {
            let line = Line(line as i32);
            if !text.ends_with('\r') && line + 1 != lines.len() {
                term.grid[line][Column(num_cols - 1)]
                    .flags
                    .insert(Flags::WRAPLINE);
            }

            let mut index = 0;
            for c in text.chars().take_while(|c| *c != '\r') {
                term.grid[line][Column(index)].c = c;

                let width = c.width().unwrap();
                if width == 2 {
                    term.grid[line][Column(index)]
                        .flags
                        .insert(Flags::WIDE_CHAR);
                    term.grid[line][Column(index + 1)]
                        .flags
                        .insert(Flags::WIDE_CHAR_SPACER);
                }

                index += width;
            }
        }

        term
    }
}

#[cfg(test)]
mod tests {
    use std::mem;

    use super::*;
    use crate::{
        event::VoidListener,
        grid::{Grid, Scroll},
        index::{Column, Point, Side},
        selection::{Selection, SelectionType},
        term::{
            cell::{Cell, Flags},
            test::TermSize,
        },
        vte::ansi::{self, CharsetIndex, Handler, StandardCharset},
    };

    #[test]
    fn scroll_display_page_up() {
        let size = TermSize::new(5, 10);
        let mut term = Term::new(Config::default(), &size, VoidListener);

        for _ in 0..20 {
            term.newline();
        }

        term.scroll_display(Scroll::PageUp);
        assert_eq!(term.vi_mode_cursor.point, Point::new(Line(-1), Column(0)));
        assert_eq!(term.grid.display_offset(), 10);

        term.scroll_display(Scroll::PageUp);
        assert_eq!(term.vi_mode_cursor.point, Point::new(Line(-2), Column(0)));
        assert_eq!(term.grid.display_offset(), 11);

        term.scroll_display(Scroll::PageUp);
        assert_eq!(term.vi_mode_cursor.point, Point::new(Line(-2), Column(0)));
        assert_eq!(term.grid.display_offset(), 11);
    }

    #[test]
    fn scroll_display_page_down() {
        let size = TermSize::new(5, 10);
        let mut term = Term::new(Config::default(), &size, VoidListener);

        for _ in 0..20 {
            term.newline();
        }

        term.grid_mut().scroll_display(Scroll::Top);
        term.vi_mode_cursor =
            ViModeCursor::new(Point::new(Line(-11), Column(0)));

        term.scroll_display(Scroll::PageDown);
        assert_eq!(term.vi_mode_cursor.point, Point::new(Line(-1), Column(0)));
        assert_eq!(term.grid.display_offset(), 1);

        term.scroll_display(Scroll::PageDown);
        assert_eq!(term.vi_mode_cursor.point, Point::new(Line(0), Column(0)));
        assert_eq!(term.grid.display_offset(), 0);

        term.scroll_display(Scroll::PageDown);
        assert_eq!(term.vi_mode_cursor.point, Point::new(Line(0), Column(0)));
        assert_eq!(term.grid.display_offset(), 0);
    }

    #[test]
    fn simple_selection_works() {
        let size = TermSize::new(5, 5);
        let mut term = Term::new(Config::default(), &size, VoidListener);
        let grid = term.grid_mut();
        for i in 0..4 {
            if i == 1 {
                continue;
            }

            grid[Line(i)][Column(0)].c = '"';

            for j in 1..4 {
                grid[Line(i)][Column(j)].c = 'a';
            }

            grid[Line(i)][Column(4)].c = '"';
        }
        grid[Line(2)][Column(0)].c = ' ';
        grid[Line(2)][Column(4)].c = ' ';
        grid[Line(2)][Column(4)].flags.insert(Flags::WRAPLINE);
        grid[Line(3)][Column(0)].c = ' ';

        term.selection = Some(Selection::new(
            SelectionType::Simple,
            Point {
                line: Line(0),
                column: Column(0),
            },
            Side::Left,
        ));
        if let Some(s) = term.selection.as_mut() {
            s.update(
                Point {
                    line: Line(2),
                    column: Column(4),
                },
                Side::Right,
            );
        }
        assert_eq!(
            term.selection_to_string(),
            Some(String::from("\"aaa\"\n\n aaa "))
        );

        term.selection = Some(Selection::new(
            SelectionType::Simple,
            Point {
                line: Line(2),
                column: Column(0),
            },
            Side::Left,
        ));
        if let Some(s) = term.selection.as_mut() {
            s.update(
                Point {
                    line: Line(3),
                    column: Column(4),
                },
                Side::Right,
            );
        }
        assert_eq!(
            term.selection_to_string(),
            Some(String::from(" aaa  aaa\""))
        );
    }

    #[test]
    fn semantic_selection_works() {
        let size = TermSize::new(5, 3);
        let mut term = Term::new(Config::default(), &size, VoidListener);
        let mut grid: Grid<Cell> = Grid::new(3, 5, 0);
        for i in 0..5 {
            for j in 0..2 {
                grid[Line(j)][Column(i)].c = 'a';
            }
        }
        grid[Line(0)][Column(0)].c = '"';
        grid[Line(0)][Column(3)].c = '"';
        grid[Line(1)][Column(2)].c = '"';
        grid[Line(0)][Column(4)].flags.insert(Flags::WRAPLINE);

        let mut escape_chars = String::from("\"");

        mem::swap(&mut term.grid, &mut grid);
        mem::swap(&mut term.config.semantic_escape_chars, &mut escape_chars);

        {
            term.selection = Some(Selection::new(
                SelectionType::Semantic,
                Point {
                    line: Line(0),
                    column: Column(1),
                },
                Side::Left,
            ));
            assert_eq!(term.selection_to_string(), Some(String::from("aa")));
        }

        {
            term.selection = Some(Selection::new(
                SelectionType::Semantic,
                Point {
                    line: Line(0),
                    column: Column(4),
                },
                Side::Left,
            ));
            assert_eq!(term.selection_to_string(), Some(String::from("aaa")));
        }

        {
            term.selection = Some(Selection::new(
                SelectionType::Semantic,
                Point {
                    line: Line(1),
                    column: Column(1),
                },
                Side::Left,
            ));
            assert_eq!(term.selection_to_string(), Some(String::from("aaa")));
        }
    }

    #[test]
    fn line_selection_works() {
        let size = TermSize::new(5, 1);
        let mut term = Term::new(Config::default(), &size, VoidListener);
        let mut grid: Grid<Cell> = Grid::new(1, 5, 0);
        for i in 0..5 {
            grid[Line(0)][Column(i)].c = 'a';
        }
        grid[Line(0)][Column(0)].c = '"';
        grid[Line(0)][Column(3)].c = '"';

        mem::swap(&mut term.grid, &mut grid);

        term.selection = Some(Selection::new(
            SelectionType::Lines,
            Point {
                line: Line(0),
                column: Column(3),
            },
            Side::Left,
        ));
        assert_eq!(term.selection_to_string(), Some(String::from("\"aa\"a\n")));
    }

    #[test]
    fn block_selection_works() {
        let size = TermSize::new(5, 5);
        let mut term = Term::new(Config::default(), &size, VoidListener);
        let grid = term.grid_mut();
        for i in 1..4 {
            grid[Line(i)][Column(0)].c = '"';

            for j in 1..4 {
                grid[Line(i)][Column(j)].c = 'a';
            }

            grid[Line(i)][Column(4)].c = '"';
        }
        grid[Line(2)][Column(2)].c = ' ';
        grid[Line(2)][Column(4)].flags.insert(Flags::WRAPLINE);
        grid[Line(3)][Column(4)].c = ' ';

        term.selection = Some(Selection::new(
            SelectionType::Block,
            Point {
                line: Line(0),
                column: Column(3),
            },
            Side::Left,
        ));

        if let Some(s) = term.selection.as_mut() {
            s.update(
                Point {
                    line: Line(3),
                    column: Column(3),
                },
                Side::Right,
            );
        }
        assert_eq!(term.selection_to_string(), Some(String::from("\na\na\na")));

        if let Some(s) = term.selection.as_mut() {
            s.update(
                Point {
                    line: Line(3),
                    column: Column(0),
                },
                Side::Left,
            );
        }
        assert_eq!(
            term.selection_to_string(),
            Some(String::from("\n\"aa\n\"a\n\"aa"))
        );

        if let Some(s) = term.selection.as_mut() {
            s.update(
                Point {
                    line: Line(3),
                    column: Column(4),
                },
                Side::Right,
            );
        }
        assert_eq!(
            term.selection_to_string(),
            Some(String::from("\na\"\na\"\na"))
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn grid_serde() {
        let grid: Grid<Cell> = Grid::new(24, 80, 0);
        let serialized = serde_json::to_string(&grid).expect("ser");
        let deserialized =
            serde_json::from_str::<Grid<Cell>>(&serialized).expect("de");

        assert_eq!(deserialized, grid);
    }

    #[test]
    fn input_line_drawing_character() {
        let size = TermSize::new(7, 17);
        let mut term = Term::new(Config::default(), &size, VoidListener);
        let cursor = Point::new(Line(0), Column(0));
        term.configure_charset(
            CharsetIndex::G0,
            StandardCharset::SpecialCharacterAndLineDrawing,
        );
        term.input('a');

        assert_eq!(term.grid()[cursor].c, '▒');
    }

    #[test]
    fn clearing_viewport_keeps_history_position() {
        let size = TermSize::new(10, 20);
        let mut term = Term::new(Config::default(), &size, VoidListener);

        for _ in 0..29 {
            term.newline();
        }

        term.scroll_display(Scroll::Top);

        assert_eq!(term.grid.display_offset(), 10);

        term.clear_screen(ansi::ClearMode::All);

        assert_eq!(term.grid.display_offset(), 10);
    }

    #[test]
    fn clearing_viewport_with_vi_mode_keeps_history_position() {
        let size = TermSize::new(10, 20);
        let mut term = Term::new(Config::default(), &size, VoidListener);

        for _ in 0..29 {
            term.newline();
        }

        term.toggle_vi_mode();

        term.scroll_display(Scroll::Top);
        term.vi_mode_cursor.point = Point::new(Line(-5), Column(3));

        assert_eq!(term.grid.display_offset(), 10);

        term.clear_screen(ansi::ClearMode::All);

        assert_eq!(term.grid.display_offset(), 10);
        assert_eq!(term.vi_mode_cursor.point, Point::new(Line(-5), Column(3)));
    }

    #[test]
    fn clearing_scrollback_resets_display_offset() {
        let size = TermSize::new(10, 20);
        let mut term = Term::new(Config::default(), &size, VoidListener);

        for _ in 0..29 {
            term.newline();
        }

        term.scroll_display(Scroll::Top);

        assert_eq!(term.grid.display_offset(), 10);

        term.clear_screen(ansi::ClearMode::Saved);

        assert_eq!(term.grid.display_offset(), 0);
    }

    #[test]
    fn clearing_scrollback_sets_vi_cursor_into_viewport() {
        let size = TermSize::new(10, 20);
        let mut term = Term::new(Config::default(), &size, VoidListener);

        for _ in 0..29 {
            term.newline();
        }

        term.toggle_vi_mode();

        term.scroll_display(Scroll::Top);
        term.vi_mode_cursor.point = Point::new(Line(-5), Column(3));

        assert_eq!(term.grid.display_offset(), 10);

        term.clear_screen(ansi::ClearMode::Saved);

        assert_eq!(term.grid.display_offset(), 0);
        assert_eq!(term.vi_mode_cursor.point, Point::new(Line(0), Column(3)));
    }

    #[test]
    fn clear_saved_lines() {
        let size = TermSize::new(7, 17);
        let mut term = Term::new(Config::default(), &size, VoidListener);

        term.grid.scroll_up(&(Line(0)..Line(1)), 1);

        term.clear_screen(ansi::ClearMode::Saved);

        let mut scrolled_grid = term.grid.clone();
        scrolled_grid.scroll_display(Scroll::Top);

        scrolled_grid.truncate();
        term.grid.truncate();

        assert_eq!(term.grid, scrolled_grid);
    }

    #[test]
    fn vi_cursor_keep_pos_on_scrollback_buffer() {
        let size = TermSize::new(5, 10);
        let mut term = Term::new(Config::default(), &size, VoidListener);

        for _ in 0..20 {
            term.newline();
        }

        term.toggle_vi_mode();

        term.scroll_display(Scroll::Top);
        term.vi_mode_cursor.point.line = Line(-11);

        term.linefeed();
        assert_eq!(term.vi_mode_cursor.point.line, Line(-12));
    }

    #[test]
    fn grow_lines_updates_active_cursor_pos() {
        let mut size = TermSize::new(100, 10);
        let mut term = Term::new(Config::default(), &size, VoidListener);

        for _ in 0..19 {
            term.newline();
        }
        assert_eq!(term.history_size(), 10);
        assert_eq!(term.grid.cursor.point, Point::new(Line(9), Column(0)));

        size.screen_lines = 30;
        term.resize(size);

        assert_eq!(term.history_size(), 0);
        assert_eq!(term.grid.cursor.point, Point::new(Line(19), Column(0)));
    }

    #[test]
    fn grow_lines_updates_inactive_cursor_pos() {
        let mut size = TermSize::new(100, 10);
        let mut term = Term::new(Config::default(), &size, VoidListener);

        for _ in 0..19 {
            term.newline();
        }
        assert_eq!(term.history_size(), 10);
        assert_eq!(term.grid.cursor.point, Point::new(Line(9), Column(0)));

        term.set_private_mode(
            NamedPrivateMode::SwapScreenAndSetRestoreCursor.into(),
        );

        size.screen_lines = 30;
        term.resize(size);

        term.unset_private_mode(
            NamedPrivateMode::SwapScreenAndSetRestoreCursor.into(),
        );

        assert_eq!(term.history_size(), 0);
        assert_eq!(term.grid.cursor.point, Point::new(Line(19), Column(0)));
    }

    #[test]
    fn shrink_lines_updates_active_cursor_pos() {
        let mut size = TermSize::new(100, 10);
        let mut term = Term::new(Config::default(), &size, VoidListener);

        for _ in 0..19 {
            term.newline();
        }
        assert_eq!(term.history_size(), 10);
        assert_eq!(term.grid.cursor.point, Point::new(Line(9), Column(0)));

        size.screen_lines = 5;
        term.resize(size);

        assert_eq!(term.history_size(), 15);
        assert_eq!(term.grid.cursor.point, Point::new(Line(4), Column(0)));
    }

    #[test]
    fn shrink_lines_updates_inactive_cursor_pos() {
        let mut size = TermSize::new(100, 10);
        let mut term = Term::new(Config::default(), &size, VoidListener);

        for _ in 0..19 {
            term.newline();
        }
        assert_eq!(term.history_size(), 10);
        assert_eq!(term.grid.cursor.point, Point::new(Line(9), Column(0)));

        term.set_private_mode(
            NamedPrivateMode::SwapScreenAndSetRestoreCursor.into(),
        );

        size.screen_lines = 5;
        term.resize(size);

        term.unset_private_mode(
            NamedPrivateMode::SwapScreenAndSetRestoreCursor.into(),
        );

        assert_eq!(term.history_size(), 15);
        assert_eq!(term.grid.cursor.point, Point::new(Line(4), Column(0)));
    }

    #[test]
    fn damage_public_usage() {
        let size = TermSize::new(10, 10);
        let mut term = Term::new(Config::default(), &size, VoidListener);
        term.reset_damage();


        let left = term.grid.cursor.point.column.0;
        term.input('d');
        term.input('a');
        term.input('m');
        term.input('a');
        term.input('g');
        term.input('e');
        let right = term.grid.cursor.point.column.0;

        let mut damaged_lines = match term.damage() {
            TermDamage::Full => {
                panic!("Expected partial damage, however got Full")
            }
            TermDamage::Partial(damaged_lines) => damaged_lines,
        };
        assert_eq!(
            damaged_lines.next(),
            Some(LineDamageBounds {
                line: 0,
                left,
                right,
            })
        );
        assert_eq!(damaged_lines.next(), None);
        term.reset_damage();

        for _ in 0..20 {
            term.newline();
        }

        match term.damage() {
            TermDamage::Full => (),
            TermDamage::Partial(_) => {
                panic!("Expected Full damage, however got Partial ")
            }
        };
        term.reset_damage();

        term.scroll_display(Scroll::Delta(10));
        term.reset_damage();

        for idx in 0..term.columns() {
            term.goto(idx as i32, idx);
        }
        let mut damaged_lines = match term.damage() {
            TermDamage::Full => {
                panic!("Expected partial damage, however got Full")
            }
            TermDamage::Partial(damaged_lines) => damaged_lines,
        };
        assert_eq!(damaged_lines.next(), None);

        term.scroll_display(Scroll::Delta(-2));
        term.reset_damage();

        term.goto(0, 0);
        term.goto(1, 0);
        term.goto(2, 0);
        let display_offset = term.grid().display_offset();
        let mut damaged_lines = match term.damage() {
            TermDamage::Full => {
                panic!("Expected partial damage, however got Full")
            }
            TermDamage::Partial(damaged_lines) => damaged_lines,
        };
        assert_eq!(
            damaged_lines.next(),
            Some(LineDamageBounds {
                line: display_offset,
                left: 0,
                right: 0,
            })
        );
        assert_eq!(
            damaged_lines.next(),
            Some(LineDamageBounds {
                line: display_offset + 1,
                left: 0,
                right: 0,
            })
        );
        assert_eq!(damaged_lines.next(), None);
    }

    #[test]
    fn damage_cursor_movements() {
        let size = TermSize::new(10, 10);
        let mut term = Term::new(Config::default(), &size, VoidListener);
        let num_cols = term.columns();
        term.reset_damage();

        term.goto(1, 1);


        assert_eq!(
            term.damage.lines[0],
            LineDamageBounds {
                line: 0,
                left: 0,
                right: 0,
            }
        );
        assert_eq!(
            term.damage.lines[1],
            LineDamageBounds {
                line: 1,
                left: 1,
                right: 1,
            }
        );
        term.damage.reset(num_cols);

        term.move_forward(3);
        assert_eq!(
            term.damage.lines[1],
            LineDamageBounds {
                line: 1,
                left: 1,
                right: 4,
            }
        );
        term.damage.reset(num_cols);

        term.move_backward(8);
        assert_eq!(
            term.damage.lines[1],
            LineDamageBounds {
                line: 1,
                left: 0,
                right: 4,
            }
        );
        term.goto(5, 5);
        term.damage.reset(num_cols);

        term.backspace();
        term.backspace();
        assert_eq!(
            term.damage.lines[5],
            LineDamageBounds {
                line: 5,
                left: 3,
                right: 5,
            }
        );
        term.damage.reset(num_cols);

        term.move_up(1);
        assert_eq!(
            term.damage.lines[5],
            LineDamageBounds {
                line: 5,
                left: 3,
                right: 3,
            }
        );
        assert_eq!(
            term.damage.lines[4],
            LineDamageBounds {
                line: 4,
                left: 3,
                right: 3,
            }
        );
        term.damage.reset(num_cols);

        term.move_down(1);
        term.move_down(1);
        assert_eq!(
            term.damage.lines[4],
            LineDamageBounds {
                line: 4,
                left: 3,
                right: 3,
            }
        );
        assert_eq!(
            term.damage.lines[5],
            LineDamageBounds {
                line: 5,
                left: 3,
                right: 3,
            }
        );
        assert_eq!(
            term.damage.lines[6],
            LineDamageBounds {
                line: 6,
                left: 3,
                right: 3,
            }
        );
        term.damage.reset(num_cols);

        term.wrapline();
        assert_eq!(
            term.damage.lines[6],
            LineDamageBounds {
                line: 6,
                left: 3,
                right: 3,
            }
        );
        assert_eq!(
            term.damage.lines[7],
            LineDamageBounds {
                line: 7,
                left: 0,
                right: 0,
            }
        );
        term.move_forward(3);
        term.move_up(1);
        term.damage.reset(num_cols);

        term.linefeed();
        assert_eq!(
            term.damage.lines[6],
            LineDamageBounds {
                line: 6,
                left: 3,
                right: 3,
            }
        );
        assert_eq!(
            term.damage.lines[7],
            LineDamageBounds {
                line: 7,
                left: 3,
                right: 3,
            }
        );
        term.damage.reset(num_cols);

        term.carriage_return();
        assert_eq!(
            term.damage.lines[7],
            LineDamageBounds {
                line: 7,
                left: 0,
                right: 3,
            }
        );
        term.damage.reset(num_cols);

        term.erase_chars(5);
        assert_eq!(
            term.damage.lines[7],
            LineDamageBounds {
                line: 7,
                left: 0,
                right: 5,
            }
        );
        term.damage.reset(num_cols);

        term.delete_chars(3);
        let right = term.columns() - 1;
        assert_eq!(
            term.damage.lines[7],
            LineDamageBounds {
                line: 7,
                left: 0,
                right,
            }
        );
        term.move_forward(term.columns());
        term.damage.reset(num_cols);

        term.move_backward_tabs(1);
        assert_eq!(
            term.damage.lines[7],
            LineDamageBounds {
                line: 7,
                left: 8,
                right,
            }
        );
        term.save_cursor_position();
        term.goto(1, 1);
        term.damage.reset(num_cols);

        term.restore_cursor_position();
        assert_eq!(
            term.damage.lines[1],
            LineDamageBounds {
                line: 1,
                left: 1,
                right: 1,
            }
        );
        assert_eq!(
            term.damage.lines[7],
            LineDamageBounds {
                line: 7,
                left: 8,
                right: 8,
            }
        );
        term.damage.reset(num_cols);

        term.clear_line(ansi::LineClearMode::All);
        assert_eq!(
            term.damage.lines[7],
            LineDamageBounds {
                line: 7,
                left: 0,
                right,
            }
        );
        term.damage.reset(num_cols);

        term.clear_line(ansi::LineClearMode::Left);
        assert_eq!(
            term.damage.lines[7],
            LineDamageBounds {
                line: 7,
                left: 0,
                right: 8,
            }
        );
        term.damage.reset(num_cols);

        term.clear_line(ansi::LineClearMode::Right);
        assert_eq!(
            term.damage.lines[7],
            LineDamageBounds {
                line: 7,
                left: 8,
                right,
            }
        );
        term.damage.reset(num_cols);

        term.reverse_index();
        assert_eq!(
            term.damage.lines[7],
            LineDamageBounds {
                line: 7,
                left: 8,
                right: 8,
            }
        );
        assert_eq!(
            term.damage.lines[6],
            LineDamageBounds {
                line: 6,
                left: 8,
                right: 8,
            }
        );
    }

    #[test]
    fn full_damage() {
        let size = TermSize::new(100, 10);
        let mut term = Term::new(Config::default(), &size, VoidListener);

        assert!(term.damage.full);
        for _ in 0..20 {
            term.newline();
        }
        term.reset_damage();

        term.clear_screen(ansi::ClearMode::Above);
        assert!(term.damage.full);
        term.reset_damage();

        term.scroll_display(Scroll::Top);
        assert!(term.damage.full);
        term.reset_damage();

        term.scroll_display(Scroll::Top);
        assert!(!term.damage.full);
        term.reset_damage();

        term.set_options(Config::default());
        assert!(term.damage.full);
        term.reset_damage();

        term.scroll_down_relative(Line(5), 2);
        assert!(term.damage.full);
        term.reset_damage();

        term.scroll_up_relative(Line(3), 2);
        assert!(term.damage.full);
        term.reset_damage();

        term.deccolm();
        assert!(term.damage.full);
        term.reset_damage();

        term.decaln();
        assert!(term.damage.full);
        term.reset_damage();

        term.set_mode(NamedMode::Insert.into());
        assert!(!term.damage.full);
        term.reset_damage();

        let color_index = 257;
        term.set_color(color_index, Rgb::default());
        assert!(term.damage.full);
        term.reset_damage();

        term.set_color(color_index, Rgb::default());
        assert!(!term.damage.full);

        term.reset_color(color_index);
        assert!(term.damage.full);
        term.reset_damage();

        term.set_color(NamedColor::Cursor as usize, Rgb::default());
        assert!(!term.damage.full);

        let _ = term.damage();
        assert!(term.damage.full);
        term.reset_damage();

        term.unset_mode(NamedMode::Insert.into());
        assert!(term.damage.full);
        term.reset_damage();

        term.swap_alt();
        assert!(term.damage.full);
        term.reset_damage();

        let size = TermSize::new(10, 10);
        term.resize(size);
        assert!(term.damage.full);
    }

    #[test]
    fn window_title() {
        let size = TermSize::new(7, 17);
        let mut term = Term::new(Config::default(), &size, VoidListener);

        assert_eq!(term.title, None);

        term.set_title(Some("Test".into()));
        assert_eq!(term.title, Some("Test".into()));

        term.push_title();
        term.set_title(Some("Next".into()));
        assert_eq!(term.title, Some("Next".into()));
        assert_eq!(term.title_stack.first().unwrap(), &Some("Test".into()));

        term.pop_title();
        assert_eq!(term.title, Some("Test".into()));
        assert!(term.title_stack.is_empty());

        for _ in 0..4097 {
            term.push_title();
        }
        assert_eq!(term.title_stack.len(), 4096);

        term.push_title();
        term.reset_state();
        assert_eq!(term.title, None);
        assert!(term.title_stack.is_empty());

        term.title = None;
        term.push_title();
        term.set_title(Some("Test".into()));
        term.pop_title();
        assert_eq!(term.title, None);

        term.title = Some("Test".into());
        term.set_title(None);
        assert_eq!(term.title, None);
    }

    #[test]
    fn parse_cargo_version() {
        assert!(version_number(env!("CARGO_PKG_VERSION")) >= 10_01);
        assert_eq!(version_number("0.0.1-dev"), 1);
        assert_eq!(version_number("0.1.2-dev"), 1_02);
        assert_eq!(version_number("1.2.3-dev"), 1_02_03);
        assert_eq!(version_number("999.99.99"), 9_99_99_99);
    }
}
