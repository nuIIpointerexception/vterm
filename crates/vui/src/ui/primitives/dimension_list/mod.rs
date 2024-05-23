mod axis;

pub use self::axis::Axis;
use crate::{builder_field, ui::primitives::Dimensions, vec2, Vec2};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Justify {
    Begin,
    Center,
    End,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SpaceBetween {
    Fixed(f32),

    EvenSpaceBetween,

    EvenSpaceAround,
}

pub struct DimensionList {
    main_axis: Axis,
    off_axis: Axis,
    children: Vec<(Dimensions, Justify)>,
    total_children_size: Dimensions,
    max_size: Dimensions,
    space_between: SpaceBetween,
}

impl DimensionList {
    pub fn new(main_axis: Axis, off_axis: Axis) -> Self {
        Self {
            main_axis,
            off_axis,
            total_children_size: Dimensions::new(0.0, 0.0),
            max_size: Dimensions::new(0.0, 0.0),
            children: Vec::new(),
            space_between: SpaceBetween::Fixed(0.0),
        }
    }

    pub fn horizontal() -> Self {
        Self::new(Axis::Horizontal, Axis::Vertical)
    }

    pub fn vertical() -> Self {
        Self::new(Axis::Vertical, Axis::Horizontal)
    }

    builder_field!(space_between, SpaceBetween);

    pub fn set_max_size(&mut self, max_size: &Dimensions) {
        self.max_size = *max_size;
    }

    pub fn dimensions(&self) -> Dimensions {
        match self.space_between {
            SpaceBetween::Fixed(_) => {
                self.total_children_size
            }
            _ => {
                self.off_axis.min(&self.max_size, &self.total_children_size)
            }
        }
    }

    pub fn add_child_dimensions(
        &mut self,
        child_dimensions: Dimensions,
        justify: Justify,
    ) -> Dimensions {
        self.children.push((child_dimensions, justify));

        self.total_children_size = self
            .main_axis
            .sum(&self.total_children_size, &child_dimensions);

        if let SpaceBetween::Fixed(size) = self.space_between {
            if self.children.len() > 1 {
                self.total_children_size =
                    self.main_axis.add_scalar(&self.total_children_size, size);
            }
        }

        self.total_children_size = self
            .off_axis
            .max(&self.total_children_size, &child_dimensions);

        self.main_axis
            .sub(&self.max_size, &self.total_children_size)
    }

    pub fn compute_child_positions(&self) -> Vec<Vec2> {
        let main_axis_remaining_size = self.main_axis.get(&self.max_size)
            - self.main_axis.get(&self.total_children_size);
        let main_axis_offset = match self.space_between {
            SpaceBetween::Fixed(size) => self.main_axis.vec2(size),
            SpaceBetween::EvenSpaceBetween => {
                let space_count = (self.children.len() - 1).max(1) as f32;
                let offset = main_axis_remaining_size / space_count;
                self.main_axis.vec2(offset)
            }
            SpaceBetween::EvenSpaceAround => {
                let space_count = (self.children.len() + 1) as f32;
                let offset = main_axis_remaining_size / space_count;
                self.main_axis.vec2(offset)
            }
        };

        let mut position = match self.space_between {
            SpaceBetween::EvenSpaceAround => main_axis_offset,
            _ => vec2(0.0, 0.0),
        };

        let mut child_positions = Vec::with_capacity(self.children.len());
        for (child, justify) in &self.children {
            let off_axis_remaining_size =
                self.off_axis.get(&self.total_children_size)
                    - self.off_axis.get(child);
            let off_axis_offset = match *justify {
                Justify::Begin => self.off_axis.vec2(0.0),
                Justify::End => self.off_axis.vec2(off_axis_remaining_size),
                Justify::Center => {
                    self.off_axis.vec2(0.5 * off_axis_remaining_size)
                }
            };

            child_positions.push(position + off_axis_offset);

            position += main_axis_offset
                + self.main_axis.vec2(self.main_axis.get(child));
        }

        child_positions
    }
}
