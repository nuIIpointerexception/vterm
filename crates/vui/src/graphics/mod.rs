use ::anyhow::Result;

pub use self::{sprite::Sprite, vertex::Vertex};

mod sprite;
mod vertex;

mod color;
pub mod triangles;

pub trait VertexStream {
    fn push_vertices(
        &mut self,
        vertices: &[Vertex],
        indices: &[u32],
    ) -> Result<()>;
}
