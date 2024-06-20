use ::anyhow::Result;

use crate::{
    graphics::triangles::Frame,
    ui::{
        primitives::Dimensions,
        widgets::{Element, Widget},
        Input, InternalState,
    },
    Vec2,
};

#[derive(Debug, Copy, Clone)]
pub enum ComposedMessage<I, E> {
    Internal(I),
    External(E),
}

pub struct ComposedElement<E>(pub Element<E>);

impl<I, E> Widget<ComposedMessage<I, E>> for ComposedElement<E>
where
    E: 'static,
{
    fn handle_event(
        &mut self,
        internal_state: &mut InternalState,
        input: &Input,
        event: &winit::event::WindowEvent,
    ) -> Result<Option<ComposedMessage<I, E>>> {
        self.0
            .handle_event(internal_state, input, event)
            .map(|opt| opt.map(ComposedMessage::External))
    }

    fn draw_frame(&mut self, internal_state: &mut InternalState, frame: &mut Frame) -> Result<()> {
        self.0.draw_frame(internal_state, frame)
    }

    fn dimensions(
        &mut self,
        internal_state: &mut InternalState,
        max_size: &Dimensions,
    ) -> Dimensions {
        self.0.dimensions(internal_state, max_size)
    }

    fn set_top_left_position(&mut self, internal_state: &mut InternalState, position: Vec2) {
        self.0.set_top_left_position(internal_state, position);
    }
}

impl<I, E> Into<Element<ComposedMessage<I, E>>> for Element<E>
where
    E: 'static,
{
    fn into(self) -> Element<ComposedMessage<I, E>> {
        Element::new(ComposedElement(self))
    }
}
