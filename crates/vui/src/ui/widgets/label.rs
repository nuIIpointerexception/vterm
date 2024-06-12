use ::anyhow::Result;

use crate::{
    graphics::triangles::Frame,
    ui::{
        Font,
        Input,
        InternalState, primitives::{Dimensions, Rect, Tile}, widgets::{Element, Widget},
    },
    vec2, Vec2,
};

pub struct Label {
    glyph_tiles: Vec<Tile>,
    bounds: Rect,
}

impl Label {
    pub fn new<T>(font: &Font, content: T) -> Self
    where
        T: AsRef<str>,
    {
        let (glyph_tiles, bounds) = font.build_text_tiles(content);
        Self {
            glyph_tiles,
            bounds,
        }
    }
}

impl<Message> Widget<Message> for Label {
    fn handle_event(
        &mut self,
        _internal_state: &mut InternalState,
        _input: &Input,
        _event: &winit::event::WindowEvent,
    ) -> Result<Option<Message>> {
        Ok(None)
    }

    fn draw_frame(
        &self,
        _internal_state: &mut InternalState,
        frame: &mut Frame,
    ) -> Result<()> {
        for tile in &self.glyph_tiles {
            tile.fill(frame)?;
        }
        Ok(())
    }

    fn dimensions(
        &mut self,
        _internal_state: &mut InternalState,
        max_size: &Dimensions,
    ) -> Dimensions {
        if self.glyph_tiles.len() == 0 {
            (0, 0).into()
        } else {
            self.bounds.dimensions().min(max_size)
        }
    }

    fn set_top_left_position(
        &mut self,
        _internal_state: &mut InternalState,
        position: Vec2,
    ) {
        if self.glyph_tiles.len() == 0 {
            return;
        }

        let current_position = self.bounds.top_left;
        let raw_offset = position - current_position;
        let offset = vec2(raw_offset.x.round(), raw_offset.y.round());

        for tile in &mut self.glyph_tiles {
            tile.model = tile.model.translate(offset);
        }
    }
}

impl<Message> Into<Element<Message>> for Label
where
    Message: 'static,
{
    fn into(self) -> Element<Message> {
        Element::new(self)
    }
}
