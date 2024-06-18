use crate::ui::color::{Color, Gradient};

pub enum BorderStyle {
    Color(Color),
    Gradient(Gradient),
}

pub struct Border {
    style: BorderStyle,
    width: f32,
}