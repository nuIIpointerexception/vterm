use crate::{
    graphics::{triangles::Frame, Vertex},
    ui::{
        color::Color,
        widgets::{CompositeStyle, Drawable, FillStyle},
    },
    vec2, vec3, Vec2,
};

pub struct Rectangle {
    pub width: f32,
    pub height: f32,
    pub position: Vec2,
    pub depth: f32,
    pub style: CompositeStyle,
}

impl Rectangle {
    /// Creates a new rectangle with the given width, height, position, and
    /// depth.
    ///
    /// Args:
    /// - width: f32
    /// - height: f32
    /// - position: Vec2
    /// - depth: f32
    /// - style: CompositeStyle
    ///
    /// Returns:
    /// - A new rectangle with the given width, height, position, and depth.
    pub fn new(width: f32, height: f32, position: Vec2, depth: f32, style: CompositeStyle) -> Self {
        Self { width, height, position, depth, style }
    }

    /// Draws the rectangle to the frame while applying the style.
    pub fn draw(&self, frame: &mut Frame) -> anyhow::Result<()> {
        let hw = 0.5 * self.width;
        let hh = 0.5 * self.height;
        let depth = self.depth;

        let mut drawable = DrawableRect {
            vertices: vec![
                Vertex::new(
                    vec3(self.position.x - hw, self.position.y - hh, depth),
                    Color::new(0.0, 0.0, 0.0, 0.0),
                    vec2(0.0, 0.0),
                    0,
                ),
                Vertex::new(
                    vec3(self.position.x + hw, self.position.y - hh, depth),
                    Color::new(0.0, 0.0, 0.0, 0.0),
                    vec2(1.0, 0.0),
                    0,
                ),
                Vertex::new(
                    vec3(self.position.x + hw, self.position.y + hh, depth),
                    Color::new(0.0, 0.0, 0.0, 0.0),
                    vec2(1.0, 1.0),
                    0,
                ),
                Vertex::new(
                    vec3(self.position.x - hw, self.position.y + hh, depth),
                    Color::new(0.0, 0.0, 0.0, 0.0),
                    vec2(0.0, 1.0),
                    0,
                ),
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
        };

        drawable.color(self.style.background);

        for vertex in drawable.vertices {
            frame.push_vertex(vertex)?;
        }
        frame.push_indices(&drawable.indices)?;

        Ok(())
    }
}

struct DrawableRect {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl Drawable for DrawableRect {
    fn color(&mut self, color: FillStyle) {
        match color {
            FillStyle::Color(color) => {
                for vertex in &mut self.vertices {
                    vertex.rgba = [color.r, color.g, color.b, color.a];
                }
            }
            FillStyle::Gradient(gradient) => {
                let num_vertices = self.vertices.len();
                for (i, vertex) in self.vertices.iter_mut().enumerate() {
                    let t = (i as f32) / ((num_vertices - 1) as f32);
                    vertex.rgba = [
                        gradient.start.r * (1.0 - t) + gradient.end.r * t,
                        gradient.start.g * (1.0 - t) + gradient.end.g * t,
                        gradient.start.b * (1.0 - t) + gradient.end.b * t,
                        gradient.start.a * (1.0 - t) + gradient.end.a * t,
                    ];
                }
            }
            FillStyle::None => (),
        }
    }
}
