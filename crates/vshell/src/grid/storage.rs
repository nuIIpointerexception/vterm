use std::{
    cmp::max,
    mem,
    mem::MaybeUninit,
    ops::{Index, IndexMut},
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::Row;
use crate::index::Line;

const MAX_CACHE_SIZE: usize = 1_000;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Storage<T> {
    inner: Vec<Row<T>>,

    zero: usize,

    visible_lines: usize,

    len: usize,
}

impl<T: PartialEq> PartialEq for Storage<T> {
    fn eq(&self, other: &Self) -> bool {
        assert_eq!(self.zero, 0);
        assert_eq!(other.zero, 0);

        self.inner == other.inner && self.len == other.len
    }
}

impl<T> Storage<T> {
    #[inline]
    pub fn with_capacity(visible_lines: usize, columns: usize) -> Storage<T>
        where
            T: Clone + Default,
    {
        let mut inner = Vec::with_capacity(visible_lines);
        inner.resize_with(visible_lines, || Row::new(columns));

        Storage {
            inner,
            zero: 0,
            visible_lines,
            len: visible_lines,
        }
    }

    #[inline]
    pub fn grow_visible_lines(&mut self, next: usize)
        where
            T: Clone + Default,
    {
        let additional_lines = next - self.visible_lines;

        let columns = self[Line(0)].len();
        self.initialize(additional_lines, columns);

        self.visible_lines = next;
    }

    #[inline]
    pub fn shrink_visible_lines(&mut self, next: usize) {
        let shrinkage = self.visible_lines - next;
        self.shrink_lines(shrinkage);

        self.visible_lines = next;
    }

    #[inline]
    pub fn shrink_lines(&mut self, shrinkage: usize) {
        self.len -= shrinkage;

        if self.inner.len() > self.len + MAX_CACHE_SIZE {
            self.truncate();
        }
    }

    #[inline]
    pub fn truncate(&mut self) {
        self.rezero();

        self.inner.truncate(self.len);
    }

    #[inline]
    pub fn initialize(&mut self, additional_rows: usize, columns: usize)
        where
            T: Clone + Default,
    {
        if self.len + additional_rows > self.inner.len() {
            self.rezero();

            let realloc_size =
                self.inner.len() + max(additional_rows, MAX_CACHE_SIZE);
            self.inner.resize_with(realloc_size, || Row::new(columns));
        }

        self.len += additional_rows;
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn swap(&mut self, a: Line, b: Line) {
        debug_assert_eq!(mem::size_of::<Row<T>>(), mem::size_of::<usize>() * 4);

        let a = self.compute_index(a);
        let b = self.compute_index(b);

        unsafe {
            let a_ptr =
                self.inner.as_mut_ptr().add(a) as *mut MaybeUninit<usize>;
            let b_ptr =
                self.inner.as_mut_ptr().add(b) as *mut MaybeUninit<usize>;

            let mut tmp: MaybeUninit<usize>;
            for i in 0..4 {
                tmp = *a_ptr.offset(i);
                *a_ptr.offset(i) = *b_ptr.offset(i);
                *b_ptr.offset(i) = tmp;
            }
        }
    }

    #[inline]
    pub fn rotate(&mut self, count: isize) {
        debug_assert!(count.unsigned_abs() <= self.inner.len());

        let len = self.inner.len();
        self.zero = (self.zero as isize + count + len as isize) as usize % len;
    }

    #[inline]
    pub fn rotate_down(&mut self, count: usize) {
        self.zero = (self.zero + count) % self.inner.len();
    }

    #[inline]
    pub fn replace_inner(&mut self, vec: Vec<Row<T>>) {
        self.len = vec.len();
        self.inner = vec;
        self.zero = 0;
    }

    #[inline]
    pub fn take_all(&mut self) -> Vec<Row<T>> {
        self.truncate();

        let mut buffer = Vec::new();

        mem::swap(&mut buffer, &mut self.inner);
        self.len = 0;

        buffer
    }

    #[inline]
    fn compute_index(&self, requested: Line) -> usize {
        debug_assert!(requested.0 < self.visible_lines as i32);

        let positive = -(requested - self.visible_lines).0 as usize - 1;

        debug_assert!(positive < self.len);

        let zeroed = self.zero + positive;

        if zeroed >= self.inner.len() {
            zeroed - self.inner.len()
        } else {
            zeroed
        }
    }

    #[inline]
    fn rezero(&mut self) {
        if self.zero == 0 {
            return;
        }

        self.inner.rotate_left(self.zero);
        self.zero = 0;
    }
}

impl<T> Index<Line> for Storage<T> {
    type Output = Row<T>;

    #[inline]
    fn index(&self, index: Line) -> &Self::Output {
        let index = self.compute_index(index);
        &self.inner[index]
    }
}

impl<T> IndexMut<Line> for Storage<T> {
    #[inline]
    fn index_mut(&mut self, index: Line) -> &mut Self::Output {
        let index = self.compute_index(index);
        &mut self.inner[index]
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        grid::{
            row::Row,
            storage::{Storage, MAX_CACHE_SIZE},
            GridCell,
        },
        index::{Column, Line},
        term::cell::Flags,
    };

    impl GridCell for char {
        fn is_empty(&self) -> bool {
            *self == ' ' || *self == '\t'
        }

        fn reset(&mut self, template: &Self) {
            *self = *template;
        }

        fn flags(&self) -> &Flags {
            unimplemented!();
        }

        fn flags_mut(&mut self) -> &mut Flags {
            unimplemented!();
        }
    }

    #[test]
    fn with_capacity() {
        let storage = Storage::<char>::with_capacity(3, 1);

        assert_eq!(storage.inner.len(), 3);
        assert_eq!(storage.len, 3);
        assert_eq!(storage.zero, 0);
        assert_eq!(storage.visible_lines, 3);
    }

    #[test]
    fn indexing() {
        let mut storage = Storage::<char>::with_capacity(3, 1);

        storage[Line(0)] = filled_row('0');
        storage[Line(1)] = filled_row('1');
        storage[Line(2)] = filled_row('2');

        storage.zero += 1;

        assert_eq!(storage[Line(0)], filled_row('2'));
        assert_eq!(storage[Line(1)], filled_row('0'));
        assert_eq!(storage[Line(2)], filled_row('1'));
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn indexing_above_inner_len() {
        let storage = Storage::<char>::with_capacity(1, 1);
        let _ = &storage[Line(-1)];
    }

    #[test]
    fn rotate() {
        let mut storage = Storage::<char>::with_capacity(3, 1);
        storage.rotate(2);
        assert_eq!(storage.zero, 2);
        storage.shrink_lines(2);
        assert_eq!(storage.len, 1);
        assert_eq!(storage.inner.len(), 3);
        assert_eq!(storage.zero, 2);
    }

    #[test]
    fn grow_after_zero() {
        let mut storage: Storage<char> = Storage {
            inner: vec![filled_row('0'), filled_row('1'), filled_row('-')],
            zero: 0,
            visible_lines: 3,
            len: 3,
        };

        storage.grow_visible_lines(4);

        let mut expected = Storage {
            inner: vec![filled_row('0'), filled_row('1'), filled_row('-')],
            zero: 0,
            visible_lines: 4,
            len: 4,
        };
        expected
            .inner
            .append(&mut vec![filled_row('\0'); MAX_CACHE_SIZE]);

        assert_eq!(storage.visible_lines, expected.visible_lines);
        assert_eq!(storage.inner, expected.inner);
        assert_eq!(storage.zero, expected.zero);
        assert_eq!(storage.len, expected.len);
    }

    #[test]
    fn grow_before_zero() {
        let mut storage: Storage<char> = Storage {
            inner: vec![filled_row('-'), filled_row('0'), filled_row('1')],
            zero: 1,
            visible_lines: 3,
            len: 3,
        };

        storage.grow_visible_lines(4);

        let mut expected = Storage {
            inner: vec![filled_row('0'), filled_row('1'), filled_row('-')],
            zero: 0,
            visible_lines: 4,
            len: 4,
        };
        expected
            .inner
            .append(&mut vec![filled_row('\0'); MAX_CACHE_SIZE]);

        assert_eq!(storage.visible_lines, expected.visible_lines);
        assert_eq!(storage.inner, expected.inner);
        assert_eq!(storage.zero, expected.zero);
        assert_eq!(storage.len, expected.len);
    }

    #[test]
    fn shrink_before_zero() {
        let mut storage: Storage<char> = Storage {
            inner: vec![filled_row('2'), filled_row('0'), filled_row('1')],
            zero: 1,
            visible_lines: 3,
            len: 3,
        };

        storage.shrink_visible_lines(2);

        let expected = Storage {
            inner: vec![filled_row('2'), filled_row('0'), filled_row('1')],
            zero: 1,
            visible_lines: 2,
            len: 2,
        };
        assert_eq!(storage.visible_lines, expected.visible_lines);
        assert_eq!(storage.inner, expected.inner);
        assert_eq!(storage.zero, expected.zero);
        assert_eq!(storage.len, expected.len);
    }

    #[test]
    fn shrink_after_zero() {
        let mut storage: Storage<char> = Storage {
            inner: vec![filled_row('0'), filled_row('1'), filled_row('2')],
            zero: 0,
            visible_lines: 3,
            len: 3,
        };

        storage.shrink_visible_lines(2);

        let expected = Storage {
            inner: vec![filled_row('0'), filled_row('1'), filled_row('2')],
            zero: 0,
            visible_lines: 2,
            len: 2,
        };
        assert_eq!(storage.visible_lines, expected.visible_lines);
        assert_eq!(storage.inner, expected.inner);
        assert_eq!(storage.zero, expected.zero);
        assert_eq!(storage.len, expected.len);
    }

    #[test]
    fn shrink_before_and_after_zero() {
        let mut storage: Storage<char> = Storage {
            inner: vec![
                filled_row('4'),
                filled_row('5'),
                filled_row('0'),
                filled_row('1'),
                filled_row('2'),
                filled_row('3'),
            ],
            zero: 2,
            visible_lines: 6,
            len: 6,
        };

        storage.shrink_visible_lines(2);

        let expected = Storage {
            inner: vec![
                filled_row('4'),
                filled_row('5'),
                filled_row('0'),
                filled_row('1'),
                filled_row('2'),
                filled_row('3'),
            ],
            zero: 2,
            visible_lines: 2,
            len: 2,
        };
        assert_eq!(storage.visible_lines, expected.visible_lines);
        assert_eq!(storage.inner, expected.inner);
        assert_eq!(storage.zero, expected.zero);
        assert_eq!(storage.len, expected.len);
    }

    #[test]
    fn truncate_invisible_lines() {
        let mut storage: Storage<char> = Storage {
            inner: vec![
                filled_row('4'),
                filled_row('5'),
                filled_row('0'),
                filled_row('1'),
                filled_row('2'),
                filled_row('3'),
            ],
            zero: 2,
            visible_lines: 1,
            len: 2,
        };

        storage.truncate();

        let expected = Storage {
            inner: vec![filled_row('0'), filled_row('1')],
            zero: 0,
            visible_lines: 1,
            len: 2,
        };
        assert_eq!(storage.visible_lines, expected.visible_lines);
        assert_eq!(storage.inner, expected.inner);
        assert_eq!(storage.zero, expected.zero);
        assert_eq!(storage.len, expected.len);
    }

    #[test]
    fn truncate_invisible_lines_beginning() {
        let mut storage: Storage<char> = Storage {
            inner: vec![filled_row('1'), filled_row('2'), filled_row('0')],
            zero: 2,
            visible_lines: 1,
            len: 2,
        };

        storage.truncate();

        let expected = Storage {
            inner: vec![filled_row('0'), filled_row('1')],
            zero: 0,
            visible_lines: 1,
            len: 2,
        };
        assert_eq!(storage.visible_lines, expected.visible_lines);
        assert_eq!(storage.inner, expected.inner);
        assert_eq!(storage.zero, expected.zero);
        assert_eq!(storage.len, expected.len);
    }

    #[test]
    fn shrink_then_grow() {
        let mut storage: Storage<char> = Storage {
            inner: vec![
                filled_row('4'),
                filled_row('5'),
                filled_row('0'),
                filled_row('1'),
                filled_row('2'),
                filled_row('3'),
            ],
            zero: 2,
            visible_lines: 0,
            len: 6,
        };

        storage.shrink_lines(3);

        let shrinking_expected = Storage {
            inner: vec![
                filled_row('4'),
                filled_row('5'),
                filled_row('0'),
                filled_row('1'),
                filled_row('2'),
                filled_row('3'),
            ],
            zero: 2,
            visible_lines: 0,
            len: 3,
        };
        assert_eq!(storage.inner, shrinking_expected.inner);
        assert_eq!(storage.zero, shrinking_expected.zero);
        assert_eq!(storage.len, shrinking_expected.len);

        storage.initialize(1, 1);

        let growing_expected = Storage {
            inner: vec![
                filled_row('4'),
                filled_row('5'),
                filled_row('0'),
                filled_row('1'),
                filled_row('2'),
                filled_row('3'),
            ],
            zero: 2,
            visible_lines: 0,
            len: 4,
        };

        assert_eq!(storage.inner, growing_expected.inner);
        assert_eq!(storage.zero, growing_expected.zero);
        assert_eq!(storage.len, growing_expected.len);
    }

    #[test]
    fn initialize() {
        let mut storage: Storage<char> = Storage {
            inner: vec![
                filled_row('4'),
                filled_row('5'),
                filled_row('0'),
                filled_row('1'),
                filled_row('2'),
                filled_row('3'),
            ],
            zero: 2,
            visible_lines: 0,
            len: 6,
        };

        let init_size = 3;
        storage.initialize(init_size, 1);

        let mut expected_inner = vec![
            filled_row('0'),
            filled_row('1'),
            filled_row('2'),
            filled_row('3'),
            filled_row('4'),
            filled_row('5'),
        ];
        let expected_init_size = std::cmp::max(init_size, MAX_CACHE_SIZE);
        expected_inner.append(&mut vec![filled_row('\0'); expected_init_size]);
        let expected_storage = Storage {
            inner: expected_inner,
            zero: 0,
            visible_lines: 0,
            len: 9,
        };

        assert_eq!(storage.len, expected_storage.len);
        assert_eq!(storage.zero, expected_storage.zero);
        assert_eq!(storage.inner, expected_storage.inner);
    }

    #[test]
    fn rotate_wrap_zero() {
        let mut storage: Storage<char> = Storage {
            inner: vec![filled_row('-'), filled_row('-'), filled_row('-')],
            zero: 2,
            visible_lines: 0,
            len: 3,
        };

        storage.rotate(2);

        assert!(storage.zero < storage.inner.len());
    }

    fn filled_row(content: char) -> Row<char> {
        let mut row = Row::new(1);
        row[Column(0)] = content;
        row
    }
}
