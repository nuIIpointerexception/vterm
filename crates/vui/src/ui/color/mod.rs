mod gradient;
mod style;

pub use gradient::Gradient;
pub use style::Style;

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }

    pub fn lerp(self, other: Self, t: f32) -> Self {
        Color::new(
            self.r * (1.0 - t) + other.r * t,
            self.g * (1.0 - t) + other.g * t,
            self.b * (1.0 - t) + other.b * t,
            self.a * (1.0 - t) + other.a * t,
        )
    }
}

impl From<(f32, f32, f32, f32)> for Color {
    fn from(tuple: (f32, f32, f32, f32)) -> Self {
        Color::new(tuple.0, tuple.1, tuple.2, tuple.3)
    }
}