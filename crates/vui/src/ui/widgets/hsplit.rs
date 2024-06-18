use ::anyhow::Result;

use crate::{
    graphics::triangles::Frame,
    ui::{
        Input,
        InternalState,
        primitives::Dimensions, widgets::{Element, Widget},
    },
    vec2, Vec2,
};

use super::CompositeStyle;

pub struct HSplit<Message> {
    left: Option<Element<Message>>,
    right: Option<Element<Message>>,
    midpoint_offset: f32,
}

impl<Message> HSplit<Message> {
    pub fn new() -> Self {
        Self {
            left: None,
            right: None,
            midpoint_offset: 0.0,
        }
    }

    pub fn left<E>(self, element: E) -> Self
    where
        E: Into<Element<Message>>,
    {
        Self {
            left: Some(element.into()),
            ..self
        }
    }

    pub fn right<E>(self, element: E) -> Self
    where
        E: Into<Element<Message>>,
    {
        Self {
            right: Some(element.into()),
            ..self
        }
    }
}

impl<Message: 'static> Widget<Message> for HSplit<Message> {
    fn handle_event(
        &mut self,
        internal_state: &mut InternalState,
        input: &Input,
        event: &winit::event::WindowEvent,
    ) -> Result<Option<Message>> {
        if let Some(elem) = &mut self.left {
            if let Some(message) =
                elem.handle_event(internal_state, input, event)?
            {
                return Ok(Some(message));
            }
        }
        if let Some(elem) = &mut self.right {
            if let Some(message) =
                elem.handle_event(internal_state, input, event)?
            {
                return Ok(Some(message));
            }
        }
        Ok(None)
    }

    fn draw_frame(
        &mut self,
        internal_state: &mut InternalState,
        frame: &mut Frame,
    ) -> Result<()> {
        if let Some(elem) = &mut self.left {
            elem.draw_frame(internal_state, frame)?;
        }
        if let Some(elem) = &mut self.right {
            elem.draw_frame(internal_state, frame)?;
        }
        Ok(())
    }

    fn dimensions(
        &mut self,
        internal_state: &mut InternalState,
        max_size: &Dimensions,
    ) -> Dimensions {
        let half_size = Dimensions::new(0.5 * max_size.width, max_size.height);

        if let Some(elem) = &mut self.left {
            elem.dimensions(internal_state, &half_size);
        }
        if let Some(elem) = &mut self.right {
            elem.dimensions(internal_state, &half_size);
        }

        self.midpoint_offset = 0.5 * max_size.width;

        *max_size
    }

    fn set_top_left_position(
        &mut self,
        internal_state: &mut InternalState,
        position: Vec2,
    ) {
        if let Some(elem) = &mut self.left {
            elem.set_top_left_position(internal_state, position)
        }

        if let Some(elem) = &mut self.right {
            elem.set_top_left_position(
                internal_state,
                position + vec2(self.midpoint_offset, 0.0),
            )
        }
    }
}

impl<Message> Into<Element<Message>> for HSplit<Message>
where
    Message: 'static,
{
    fn into(self) -> Element<Message> {
        Element::new(self)
    }
}
