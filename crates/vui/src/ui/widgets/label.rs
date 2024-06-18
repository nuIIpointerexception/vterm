use ::anyhow::Result;

use crate::{
    graphics::{triangles::Frame, Rectangle},
    ui::{
        color::{Color, Gradient, Style},
        primitives::{Dimensions, Rect, Tile},
        widgets::{Element, Widget},
        Font, Input, InternalState,
    },
    vec2, Vec2,
};

use super::{element::StylableWidget, CompositeStyle, FillStyle};

pub enum LabelStyle {
    Plain,
    Italic,
}

pub struct Label {
    glyph_tiles: Vec<Tile>,
    bounds: Rect,
    style: Style,
    text_size: Vec2,
    bg: Rectangle,
}

impl Label {
    pub fn new(font: &Font, content: &str) -> Self {
        let (glyph_tiles, bounds) = font.build_text_tiles(content);
        
        let lines: Vec<&str> = content.lines().collect();
        let line_height = font.line_height();
        let text_width = lines.iter().map(|line| {
            font.calculate_text_bounds(line).width()
        }).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0);
        let text_height = line_height * lines.len() as f32;

        // TODO(nuii): remove once this only gets called once.
        println!("text_width: {}, text_height: {}", text_width, text_height);
        
        Self {
            glyph_tiles,
            bounds,
            style: Style::Color(Color::new(1.0, 1.0, 1.0, 1.0)),
            text_size: Vec2::new(text_width, text_height),
            bg: Rectangle::new(text_width, text_height, bounds.top_left, 0.0, CompositeStyle::default().with_background(FillStyle::Color(Color::new(1.0, 1.0, 1.0, 0.1)))),
        }
    }

    pub fn colored(mut self, color: Color) -> Self {
        self.style = Style::Color(color);
        self
    }

    pub fn gradient(mut self, start: Color, end: Color) -> Self {
        self.style = Style::Gradient(Gradient { start, end });
        self
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
        &mut self,
        _internal_state: &mut InternalState,
        frame: &mut Frame,
    ) -> Result<()> {
        let bg_top_left = Vec2::new(self.bounds.top_left.x + self.text_size.x / 2.0, self.bounds.top_left.y + self.text_size.y / 2.0);
        self.bg.position = bg_top_left;
        self.bg.draw(frame)?;
        let num_tiles = self.glyph_tiles.len() as f32;
        for (i, tile) in self.glyph_tiles.iter().enumerate() {
            let mut tile = tile.clone();
            match &self.style {
                Style::Color(color) => tile.color = *color,
                Style::Gradient(gradient) => {
                    let t = i as f32 / num_tiles;
                    tile.color = Color::lerp(gradient.start, gradient.end, t);
                }
            }
            tile.fill(frame)?;
        }
        Ok(())
    }

    fn dimensions(
        &mut self,
        _internal_state: &mut InternalState,
        max_size: &Dimensions,
    ) -> Dimensions {
        if self.glyph_tiles.is_empty() {
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
        if self.glyph_tiles.is_empty() {
            return;
        }
        let current_position = self.bounds.top_left;
        let raw_offset = position - current_position;
        let offset = vec2(raw_offset.x.round(), raw_offset.y.round());
        for tile in &mut self.glyph_tiles {
            tile.model = tile.model.translate(offset);
        }
        self.bounds.top_left = position;
    }
}

impl<Message: 'static> StylableWidget<Message> for Label {
    fn bg(&self) -> CompositeStyle {
        CompositeStyle::default()
    }

    fn top_left(&self) -> Vec2 {
        self.bounds.top_left
    }

    fn bounds(&self) -> Rect {
        self.bounds
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
