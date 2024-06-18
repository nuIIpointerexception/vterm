use anyhow::Result;

use crate::{
    graphics::{Vertex, VertexStream},
    ui::{color::Color, primitives::Rect},
    vec2, vec3,
};

#[derive(Debug, Copy, Clone)]
pub struct Tile {
    pub model: Rect,

    pub uv: Rect,

    pub depth: f32,

    pub color: Color,

    pub outline_width: f32,

    pub texture_index: i32,
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            model: Rect::new(1.0, -1.0, -1.0, 1.0),
            uv: Rect::new(0.0, 0.0, 1.0, 1.0),
            depth: 0.0,
            color: Color::new(1.0, 1.0, 1.0, 1.0),
            outline_width: 1.0,
            texture_index: 0,
        }
    }
}

impl Tile {
    pub fn fill(&self, vertices: &mut impl VertexStream) -> Result<()> {
        vertices.push_vertices(
            &[
                Vertex::new(
                    vec3(self.model.left(), self.model.top(), self.depth),
                    self.color,
                    vec2(self.uv.left(), self.uv.top()),
                    self.texture_index,
                ),
                Vertex::new(
                    vec3(self.model.right(), self.model.top(), self.depth),
                    self.color,
                    vec2(self.uv.right(), self.uv.top()),
                    self.texture_index,
                ),
                Vertex::new(
                    vec3(self.model.right(), self.model.bottom(), self.depth),
                    self.color,
                    vec2(self.uv.right(), self.uv.bottom()),
                    self.texture_index,
                ),
                Vertex::new(
                    vec3(self.model.left(), self.model.bottom(), self.depth),
                    self.color,
                    vec2(self.uv.left(), self.uv.bottom()),
                    self.texture_index,
                ),
            ],
            &[0, 1, 2, 2, 3, 0],
        )
    }

    pub fn outline(&self, vertices: &mut impl VertexStream) -> Result<()> {
        let outline_properties = Tile {
            depth: self.depth,
            color: self.color,
            texture_index: self.texture_index,
            ..Default::default()
        };

        let top_left = self.model.top_left;
        let top_right = vec2(self.model.right(), self.model.top());
        let bottom_left = vec2(self.model.left(), self.model.bottom());
        let bottom_right = self.model.bottom_right;

        let half_width = 0.5 * self.outline_width;
        let corner_top_left = Tile {
            model: Rect::new(
                top_left.y - half_width,
                top_left.x - half_width,
                top_left.y + half_width,
                top_left.x + half_width,
            ),
            uv: Rect::new(0.0, 0.0, 0.2, 0.2),
            ..outline_properties
        };
        let corner_top_right = Tile {
            model: Rect::new(
                top_right.y - half_width,
                top_right.x - half_width,
                top_right.y + half_width,
                top_right.x + half_width,
            ),
            uv: Rect::new(0.0, 0.8, 0.2, 1.0),
            ..outline_properties
        };
        let corner_bottom_left = Tile {
            model: Rect::new(
                bottom_left.y - half_width,
                bottom_left.x - half_width,
                bottom_left.y + half_width,
                bottom_left.x + half_width,
            ),
            uv: Rect::new(0.8, 0.0, 1.0, 0.2),
            ..outline_properties
        };
        let corner_bottom_right = Tile {
            model: Rect::new(
                bottom_right.y - half_width,
                bottom_right.x - half_width,
                bottom_right.y + half_width,
                bottom_right.x + half_width,
            ),
            uv: Rect::new(0.8, 0.8, 1.0, 1.0),
            ..outline_properties
        };

        let top = Tile {
            model: Rect::new(
                corner_top_left.model.top(),
                corner_top_left.model.right(),
                corner_top_right.model.bottom(),
                corner_top_right.model.left(),
            ),
            uv: Rect::new(
                corner_top_left.uv.top(),
                corner_top_left.uv.right(),
                corner_top_right.uv.bottom(),
                corner_top_right.uv.left(),
            ),
            ..outline_properties
        };
        let bottom = Tile {
            model: Rect::new(
                corner_bottom_left.model.top(),
                corner_bottom_left.model.right(),
                corner_bottom_right.model.bottom(),
                corner_bottom_right.model.left(),
            ),
            uv: Rect::new(
                corner_bottom_left.uv.top(),
                corner_bottom_left.uv.right(),
                corner_bottom_right.uv.bottom(),
                corner_bottom_right.uv.left(),
            ),
            ..outline_properties
        };
        let left = Tile {
            model: Rect::new(
                corner_top_left.model.bottom(),
                corner_top_left.model.left(),
                corner_bottom_left.model.top(),
                corner_bottom_left.model.right(),
            ),
            uv: Rect::new(
                corner_top_left.uv.bottom(),
                corner_top_left.uv.left(),
                corner_bottom_left.uv.top(),
                corner_bottom_left.uv.right(),
            ),
            ..outline_properties
        };
        let right = Tile {
            model: Rect::new(
                corner_top_right.model.bottom(),
                corner_top_right.model.left(),
                corner_bottom_right.model.top(),
                corner_bottom_right.model.right(),
            ),
            uv: Rect::new(
                corner_top_right.uv.bottom(),
                corner_top_right.uv.left(),
                corner_bottom_right.uv.top(),
                corner_bottom_right.uv.right(),
            ),
            ..outline_properties
        };

        top.fill(vertices)?;
        bottom.fill(vertices)?;
        left.fill(vertices)?;
        right.fill(vertices)?;

        corner_top_left.fill(vertices)?;
        corner_top_right.fill(vertices)?;
        corner_bottom_left.fill(vertices)?;
        corner_bottom_right.fill(vertices)?;

        Ok(())
    }
}
