use ::anyhow::Result;

pub use self::{
    align::{Align, HAlignment, VAlignment},
    col::Col,
    composite::{ComposedElement, ComposedMessage, Composite, CompositeWidget},
    container::{Constraint, Container, WithContainer},
    element::Element,
    hsplit::HSplit,
    image::Image,
    label::{Label, LabelStyle},
    row::Row,
    style::{CompositeStyle, Drawable, FillStyle, Style},
    window::Window,
};
use crate::{
    graphics::triangles::Frame,
    ui::{primitives::Dimensions, Input, InternalState},
    Vec2,
};

mod align;
mod col;
mod composite;
mod container;
mod element;
mod hsplit;
mod image;
mod label;
mod row;
mod style;
mod window;

pub mod prelude;

pub trait Widget<Message> {
    fn handle_event(
        &mut self,
        internal_state: &mut InternalState,
        input: &Input,
        event: &winit::event::WindowEvent,
    ) -> Result<Option<Message>>;

    fn draw_frame(&mut self, internal_state: &mut InternalState, frame: &mut Frame) -> Result<()>;

    fn dimensions(
        &mut self,
        internal_state: &mut InternalState,
        max_size: &Dimensions,
    ) -> Dimensions;

    fn set_top_left_position(&mut self, internal_state: &mut InternalState, position: Vec2);
}
