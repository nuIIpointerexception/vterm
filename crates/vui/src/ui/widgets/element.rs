use std::cell::RefMut;
use std::sync::{Arc, Mutex};

use ::anyhow::Result;

use crate::{
    graphics::{triangles::Frame, Rectangle},
    ui::{color::{Color, Gradient}, primitives::{Dimensions, Rect}, widgets::{CompositeStyle, Widget}, Input, InternalState},
    Vec2,
};


#[derive(Clone)]
pub struct Element<Message> {
    pub(crate) widget: Arc<Mutex<dyn Widget<Message>>>,
}

impl<Message> Element<Message> {
    pub fn new(widget: impl Widget<Message> + 'static) -> Self {
        Self {
            widget: Arc::new(Mutex::new(widget)),
        }
    }
}

impl<Message: 'static> Widget<Message> for Element<Message> {
    fn handle_event(
        &mut self,
        internal_state: &mut InternalState,
        input: &Input,
        event: &winit::event::WindowEvent,
    ) -> Result<Option<Message>> {
        let mut widget = self.widget.lock().unwrap();
        widget.handle_event(internal_state, input, event)
    }

    fn draw_frame(
        &mut self,
        internal_state: &mut InternalState,
        frame: &mut Frame,
    ) -> Result<()> {
        let mut widget = self.widget.lock().unwrap();
        widget.draw_frame(internal_state, frame)
    }

    fn dimensions(
        &mut self,
        internal_state: &mut InternalState,
        max_size: &Dimensions,
    ) -> Dimensions {
        let mut widget = self.widget.lock().unwrap();
        widget.dimensions(internal_state, max_size)
    }

    fn set_top_left_position(
        &mut self,
        internal_state: &mut InternalState,
        position: Vec2,
    ) {
        let mut widget = self.widget.lock().unwrap();
        widget.set_top_left_position(internal_state, position)
    }
}

pub trait StylableWidget<Message>: Widget<Message> {
    fn bg(&self) -> CompositeStyle;
    fn top_left(&self) -> Vec2;
    fn bounds(&self) -> Rect;
}