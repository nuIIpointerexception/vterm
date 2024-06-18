use super::{Color, Gradient};

#[derive(Debug, Clone, Copy)]
pub enum Style {
    Color(Color),
    Gradient(Gradient),
}