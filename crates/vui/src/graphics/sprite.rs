use ::anyhow::Result;

use crate::{
    graphics::{triangles::Frame, Vertex, VertexStream},
    vec2, Vec2, vec3, vec4,
};

#[derive(Debug, Copy, Clone)]
pub struct Sprite {
    pub width: f32,

    pub height: f32,

    pub position: Vec2,

    pub angle_in_radians: f32,

    pub depth: f32,

    pub texture_index: i32,
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
            position: vec2(0.0, 0.0),
            angle_in_radians: 0.0,
            depth: 0.0,
            texture_index: 0,
        }
    }
}

impl Sprite {
    pub fn draw(&self, vertices: &mut Frame) -> Result<()> {
        let rotation_matrix = nalgebra::Rotation2::new(self.angle_in_radians);

        let hw = 0.5 * self.width;
        let hh = 0.5 * self.height;
        let top_left = self.position + rotation_matrix * vec2(-hw, hh);
        let top_right = self.position + rotation_matrix * vec2(hw, hh);
        let bottom_left = self.position + rotation_matrix * vec2(-hw, -hh);
        let bottom_right = self.position + rotation_matrix * vec2(hw, -hh);

        let uv_left = 0.0;
        let uv_right = 1.0;
        let uv_top = 0.0;
        let uv_bottom = 1.0;

        vertices.push_vertices(
            &[
                Vertex::new(
                    vec3(top_left.x, top_left.y, self.depth),
                    vec4(1.0, 1.0, 1.0, 1.0),
                    vec2(uv_left, uv_top),
                    self.texture_index,
                ),
                Vertex::new(
                    vec3(top_right.x, top_right.y, self.depth),
                    vec4(1.0, 1.0, 1.0, 1.0),
                    vec2(uv_right, uv_top),
                    self.texture_index,
                ),
                Vertex::new(
                    vec3(bottom_right.x, bottom_right.y, self.depth),
                    vec4(1.0, 1.0, 1.0, 1.0),
                    vec2(uv_right, uv_bottom),
                    self.texture_index,
                ),
                Vertex::new(
                    vec3(bottom_left.x, bottom_left.y, self.depth),
                    vec4(1.0, 1.0, 1.0, 1.0),
                    vec2(uv_left, uv_bottom),
                    self.texture_index,
                ),
            ],
            &[0, 1, 2, 0, 2, 3],
        )
    }
}
