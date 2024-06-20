use crate::{ui::primitives::Dimensions, vec2, Vec2};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Rect {
    pub top_left: Vec2,
    pub bottom_right: Vec2,
}

impl Rect {
    pub fn new(top: f32, left: f32, bottom: f32, right: f32) -> Self {
        Self { top_left: vec2(left, top), bottom_right: vec2(right, bottom) }
    }

    pub fn centered_at(x: f32, y: f32, width: f32, height: f32) -> Self {
        let half_height = 0.5 * height;
        let half_width = 0.5 * width;
        Self::new(y - half_height, x - half_width, y + half_height, x + half_width)
    }

    #[inline]
    pub fn left(&self) -> f32 {
        self.top_left.x
    }

    #[inline]
    pub fn right(&self) -> f32 {
        self.bottom_right.x
    }

    #[inline]
    pub fn top(&self) -> f32 {
        self.top_left.y
    }

    #[inline]
    pub fn bottom(&self) -> f32 {
        self.bottom_right.y
    }

    pub fn width(&self) -> f32 {
        (self.left() - self.right()).abs()
    }

    pub fn height(&self) -> f32 {
        (self.top() - self.bottom()).abs()
    }

    pub fn dimensions(&self) -> Dimensions {
        (self.width(), self.height()).into()
    }

    pub fn translate(&self, offset: Vec2) -> Self {
        Self { top_left: self.top_left + offset, bottom_right: self.bottom_right + offset }
    }

    pub fn set_top_left_position(&self, position: Vec2) -> Self {
        let offset = position - self.top_left;
        self.translate(offset)
    }

    pub fn top_left(&self) -> Vec2 {
        self.top_left
    }

    pub fn contains(&self, point: Vec2) -> bool {
        let horizontal = self.left() <= point.x && point.x <= self.right();
        let vertical = self.top() <= point.y && point.y <= self.bottom();
        horizontal && vertical
    }

    pub fn expand(&self, other: Rect) -> Self {
        Self {
            top_left: vec2(self.left().min(other.left()), self.top().min(other.top())),
            bottom_right: vec2(self.right().max(other.right()), self.bottom().max(other.bottom())),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_new() {
        let rect = Rect::new(10.0, -9.0, -10.0, 9.0);

        assert_eq!(rect.left(), rect.top_left.x);
        assert_eq!(rect.right(), rect.bottom_right.x);
        assert_eq!(rect.top(), rect.top_left.y);
        assert_eq!(rect.bottom(), rect.bottom_right.y);
    }

    #[test]
    fn test_width_and_height() {
        let rect = Rect::new(10.0, -9.0, -10.0, 9.0);
        assert_eq!(rect.width(), 18.0);
        assert_eq!(rect.height(), 20.0);
    }

    #[test]
    fn test_width_and_height_abs() {
        let rect = Rect::new(-10.0, 9.0, 10.0, -9.0);
        assert_eq!(rect.width(), 18.0);
        assert_eq!(rect.height(), 20.0);
    }

    #[test]
    fn test_translate() {
        let rect = Rect::new(10.0, -9.0, -10.0, 9.0).translate(vec2(9.0, 10.0));

        assert_eq!(rect.left(), 0.0);
        assert_eq!(rect.right(), 18.0);
        assert_eq!(rect.top(), 20.0);
        assert_eq!(rect.bottom(), 0.0);

        assert_eq!(rect.width(), 18.0);
        assert_eq!(rect.height(), 20.0);
    }

    #[test]
    fn test_contains() {
        let rect = Rect::centered_at(0.0, 0.0, 10.0, 10.0);

        assert!(rect.contains(vec2(0.0, 0.0)));

        assert!(rect.contains(vec2(-5.0, -5.0)), "msg");
        assert!(rect.contains(vec2(5.0, -5.0)));
        assert!(rect.contains(vec2(-5.0, 5.0)));
        assert!(rect.contains(vec2(5.0, 5.0)));

        assert!(!rect.contains(vec2(-6.0, 0.0)));
        assert!(!rect.contains(vec2(6.0, 0.0)));
        assert!(!rect.contains(vec2(0.0, 6.0)));
        assert!(!rect.contains(vec2(0.0, -6.0)));
    }

    #[test]
    fn test_expand() {
        let rect = Rect::new(0.0, 0.0, 10.0, 10.0);
        let other = Rect::new(20.0, 2.0, 23.0, 5.0);
        let expanded = rect.expand(other);

        assert_eq!(expanded.top(), 0.0);
        assert_eq!(expanded.left(), 0.0);
        assert_eq!(expanded.right(), 10.0);
        assert_eq!(expanded.bottom(), 23.0);
    }
}
