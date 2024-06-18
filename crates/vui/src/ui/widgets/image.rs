use crate::{
    graphics::{triangles::Frame, Sprite},
    ui::{
        primitives::{Dimensions, Rect},
        widgets::{Element, Widget},
        Input, InternalState,
    },
    Vec2,
};

use super::{element::StylableWidget, CompositeStyle};

pub struct Image {
    sprite: Sprite,
    bounds: Rect,
    style: CompositeStyle,
}

impl Image {
    pub fn new(width: f32, height: f32, texture_index: i32) -> Self {
        let center = Vec2::new(width / 2.0, height / 2.0);
        let sprite = Sprite {
            width,
            height,
            position: center,
            angle_in_radians: 0.0,
            depth: 0.0,
            texture_index,
        };
        let bounds = Rect::new(0.0, 0.0, width, height);

        Self { sprite, bounds, style: CompositeStyle::default() }
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.sprite.width = width;
        self.sprite.height = height;
        self.sprite.position = Vec2::new(width / 2.0, height / 2.0);
        self.bounds = Rect::new(0.0, 0.0, width, height);
        self
    }
}

impl<Message: 'static> Widget<Message> for Image {
    fn handle_event(
        &mut self,
        _internal_state: &mut InternalState,
        _input: &Input,
        _event: &winit::event::WindowEvent,
    ) -> Result<Option<Message>, anyhow::Error> {
        Ok(None)
    }

    fn draw_frame(&mut self, _internal_state: &mut InternalState, frame: &mut Frame) -> anyhow::Result<()> {
        self.sprite.draw(frame)?;
        Ok(())
    }

    fn dimensions(&mut self, _internal_state: &mut InternalState, max_size: &Dimensions) -> Dimensions {
        self.bounds.dimensions().min(max_size)
    }

    fn set_top_left_position(&mut self, _internal_state: &mut InternalState, position: Vec2) {
        let center = Vec2::new(position.x + self.bounds.width() / 2.0, position.y + self.bounds.height() / 2.0);
        self.sprite.position = center;
        self.bounds = Rect::new(position.x, position.y, self.bounds.width(), self.bounds.height());
    }
}

impl<Message: 'static> StylableWidget<Message> for Image {
    fn bg(&self) -> CompositeStyle {
        CompositeStyle::default()
    }

    fn top_left(&self) -> Vec2 {
        self.bounds.top_left()
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }
}

impl<Message> Into<Element<Message>> for Image
where
    Message: 'static,
{
    fn into(self) -> Element<Message> {
        Element::new(self)
    }
}