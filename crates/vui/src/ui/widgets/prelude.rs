pub use crate::{
    gen_id,
    ui::{
        id::id_hash,
        primitives::{Axis, Justify, SpaceBetween},
        widgets::{
            Align, Col, Constraint, Container, Element, HAlignment, HSplit, Image, Label,
            LabelStyle, Row, VAlignment, Widget, Window, WithContainer,
        },
        Font, Id,
    },
};

pub fn align<Message, W>(widget: W) -> Align<Message, W>
where
    W: Widget<Message>,
{
    Align::new(widget)
}

pub fn label(font: &Font, text: &str) -> Label {
    Label::new(font, text)
}

pub fn img(width: f32, height: f32, texture_index: i32) -> Image {
    Image::new(width, height, texture_index)
}

pub fn col<Message>() -> Col<Message> {
    Col::new()
}

pub fn row<Message>() -> Row<Message> {
    Row::new()
}

pub fn hsplit<Message>() -> HSplit<Message> {
    HSplit::new()
}
