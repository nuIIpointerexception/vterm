use ::anyhow::Result;

use crate::{
    graphics::triangles::Frame,
    ui::{Input, InternalState, primitives::Dimensions, widgets::Widget},
    Vec2,
};

pub struct Element<Message> {
    pub(crate) widget: Box<dyn Widget<Message>>,
}

impl<Message> Element<Message> {
    pub fn new(widget: impl Widget<Message> + 'static) -> Self {
        Self {
            widget: Box::new(widget),
        }
    }
}

impl<Message> Widget<Message> for Element<Message> {
    fn handle_event(
        &mut self,
        internal_state: &mut InternalState,
        input: &Input,
        event: &winit::event::WindowEvent,
    ) -> Result<Option<Message>> {
        self.widget.handle_event(internal_state, input, event)
    }

    fn draw_frame(
        &self,
        internal_state: &mut InternalState,
        frame: &mut Frame,
    ) -> Result<()> {
        self.widget.draw_frame(internal_state, frame)
    }

    fn dimensions(
        &mut self,
        internal_state: &mut InternalState,
        max_size: &Dimensions,
    ) -> Dimensions {
        self.widget.dimensions(internal_state, max_size)
    }

    fn set_top_left_position(
        &mut self,
        internal_state: &mut InternalState,
        position: Vec2,
    ) {
        self.widget.set_top_left_position(internal_state, position)
    }
}
