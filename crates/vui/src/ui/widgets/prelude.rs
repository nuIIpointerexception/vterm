pub use crate::{
    gen_id,
    ui::{
        id::id_hash,
        primitives::{Axis, Justify, SpaceBetween},
        widgets::{
            Align, Col, Constraint, Container, Element, HAlignment,
            HSplit, Label, Row, VAlignment, Widget, Window,
            WithContainer,
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

pub fn label<T>(font: &Font, text: T) -> Label
    where
        T: AsRef<str>,
{
    Label::new(font, text)
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