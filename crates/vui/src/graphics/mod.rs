use ::anyhow::Result;

mod sprite;
mod vertex;

pub mod triangles;

pub use self::{sprite::Sprite, vertex::Vertex};

pub trait VertexStream {
    fn push_vertices(
        &mut self,
        vertices: &[Vertex],
        indices: &[u32],
    ) -> Result<()>;
}
