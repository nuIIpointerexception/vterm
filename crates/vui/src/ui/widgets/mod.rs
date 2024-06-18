use ::anyhow::Result;

use crate::{
    graphics::triangles::Frame,
    ui::{Input, InternalState, primitives::Dimensions},
    Vec2,
};

pub use self::{
    align::{Align, HAlignment, VAlignment},
    col::Col,
    composite::{ComposedElement, ComposedMessage, Composite, CompositeWidget},
    container::{Constraint, Container, WithContainer},
    element::Element,
    hsplit::HSplit,
    label::Label,
    label::LabelStyle,
    row::Row,
    window::Window,
    image::Image,
    style::CompositeStyle,
    style::Drawable,
    style::Style,
    style::FillStyle,
};

mod align;
mod col;
mod composite;
mod container;
mod element;
mod hsplit;
mod label;
mod row;
mod window;
mod image;
mod style;

pub mod prelude;

pub trait Widget<Message> {
    fn handle_event(
        &mut self,
        internal_state: &mut InternalState,
        input: &Input,
        event: &winit::event::WindowEvent,
    ) -> Result<Option<Message>>;

    fn draw_frame(
        &mut self,
        internal_state: &mut InternalState,
        frame: &mut Frame,
    ) -> Result<()>;

    fn dimensions(
        &mut self,
        internal_state: &mut InternalState,
        max_size: &Dimensions,
    ) -> Dimensions;

    fn set_top_left_position(
        &mut self,
        internal_state: &mut InternalState,
        position: Vec2,
    );

}