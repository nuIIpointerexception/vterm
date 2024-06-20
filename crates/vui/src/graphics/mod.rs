use ::anyhow::Result;

pub use self::{rectangle::Rectangle, sprite::Sprite, vertex::Vertex};

mod rectangle;
mod sprite;
mod vertex;

pub mod triangles;

pub trait VertexStream {
    fn push_vertices(&mut self, vertices: &[Vertex], indices: &[u32]) -> Result<()>;
}
