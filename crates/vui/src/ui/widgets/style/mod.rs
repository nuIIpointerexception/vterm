mod border;
pub use border::Border;

use crate::ui::color::{Color, Gradient};

#[derive(Clone, Copy)]
pub enum FillStyle {
    Color(Color),
    Gradient(Gradient),
    None,
}

impl From<Color> for FillStyle {
    fn from(color: Color) -> Self {
        FillStyle::Color(color)
    }
}

impl From<Gradient> for FillStyle {
    fn from(gradient: Gradient) -> Self {
        FillStyle::Gradient(gradient)
    }
}

impl Style for FillStyle {
    fn apply_to(&mut self, target: &mut dyn Drawable) {
        target.color(self.clone());
    }
}

pub trait Drawable {
    fn color(&mut self, color: FillStyle);
}

pub trait Style {
    fn apply_to(&mut self, target: &mut dyn Drawable);
}

#[derive(Clone, Copy)]
pub struct CompositeStyle {
    pub background: FillStyle,
    pub border: FillStyle,
}

impl CompositeStyle {
    pub fn new() -> Self {
        CompositeStyle {
            background: FillStyle::None,
            border: FillStyle::None,
        }
    }

    pub fn with_background(mut self, style: FillStyle) -> Self {
        self.background = style;
        self
    }

    pub fn with_border(mut self, style: FillStyle) -> Self {
        self.border = style;
        self
    }
}

impl Default for CompositeStyle {
    fn default() -> Self {
        CompositeStyle::new()
    }
}
