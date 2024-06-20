use crate::{ui::primitives::Dimensions, vec2, Vec2};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

impl Axis {
    pub(super) fn sum(&self, original: &Dimensions, to_add: &Dimensions) -> Dimensions {
        match *self {
            Axis::Horizontal => Dimensions::new(original.width + to_add.width, original.height),
            Axis::Vertical => Dimensions::new(original.width, original.height + to_add.height),
        }
    }

    pub(super) fn sub(&self, original: &Dimensions, to_sub: &Dimensions) -> Dimensions {
        match *self {
            Axis::Horizontal => {
                Dimensions::new((original.width - to_sub.width).abs(), original.height)
            }
            Axis::Vertical => {
                Dimensions::new(original.width, (original.height - to_sub.height).abs())
            }
        }
    }

    pub(super) fn add_scalar(&self, original: &Dimensions, to_add: f32) -> Dimensions {
        match *self {
            Axis::Horizontal => Dimensions::new(original.width + to_add, original.height),
            Axis::Vertical => Dimensions::new(original.width, original.height + to_add),
        }
    }

    pub(super) fn max(&self, original: &Dimensions, to_compare: &Dimensions) -> Dimensions {
        match *self {
            Axis::Horizontal => {
                Dimensions::new(original.width.max(to_compare.width), original.height)
            }
            Axis::Vertical => {
                Dimensions::new(original.width, original.height.max(to_compare.height))
            }
        }
    }

    pub(super) fn min(&self, original: &Dimensions, to_compare: &Dimensions) -> Dimensions {
        match *self {
            Axis::Horizontal => {
                Dimensions::new(original.width.min(to_compare.width), original.height)
            }
            Axis::Vertical => {
                Dimensions::new(original.width, original.height.min(to_compare.height))
            }
        }
    }

    pub(super) fn get(&self, dimensions: &Dimensions) -> f32 {
        match *self {
            Axis::Horizontal => dimensions.width,
            Axis::Vertical => dimensions.height,
        }
    }

    pub(super) fn vec2(&self, value: f32) -> Vec2 {
        match *self {
            Axis::Horizontal => vec2(value, 0.0),
            Axis::Vertical => vec2(0.0, value),
        }
    }
}
