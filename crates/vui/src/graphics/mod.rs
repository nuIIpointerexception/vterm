use ::anyhow::Result;

pub use self::{sprite::Sprite, vertex::Vertex, rectangle::Rectangle};

mod sprite;
mod vertex;
mod rectangle;

pub mod triangles;

pub trait VertexStream {
    fn push_vertices(
        &mut self,
        vertices: &[Vertex],
        indices: &[u32],
    ) -> Result<()>;
}
