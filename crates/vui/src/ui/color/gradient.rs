use super::Color;

#[derive(Debug, Clone, Copy)]
pub struct Gradient {
    pub start: Color,
    pub end: Color,
}

impl Gradient {
    pub fn new(start: Color, end: Color) -> Self {
        Self { start, end }
    }
}