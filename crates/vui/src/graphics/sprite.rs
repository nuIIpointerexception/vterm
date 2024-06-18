use crate::{
    graphics::{triangles::Frame, Vertex},
    ui::color::Color,
    vec2, vec3, Vec2,
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
    /// Draws the sprite to the frame.
    pub fn draw(&self, vertices: &mut Frame) -> anyhow::Result<()> {
        let hw = 0.5 * self.width;
        let hh = 0.5 * self.height;
        let depth = self.depth;
        let texture_index = self.texture_index;
        let color = Color::new(1.0, 1.0, 1.0, 1.0);

        vertices.push_vertex(Vertex::new(vec3(self.position.x - hw, self.position.y - hh, depth), color, vec2(0.0, 0.0), texture_index))?;
        vertices.push_vertex(Vertex::new(vec3(self.position.x + hw, self.position.y - hh, depth), color, vec2(1.0, 0.0), texture_index))?;
        vertices.push_vertex(Vertex::new(vec3(self.position.x + hw, self.position.y + hh, depth), color, vec2(1.0, 1.0), texture_index))?;
        vertices.push_vertex(Vertex::new(vec3(self.position.x - hw, self.position.y + hh, depth), color, vec2(0.0, 1.0), texture_index))?;

        vertices.push_indices(&[0, 1, 2, 0, 2, 3])?;

        Ok(())
    }

    /// Sets the center position of the sprite.
    pub fn set_center(&mut self, center: Vec2) {
        self.position = center;
    }
}
