use crate::{ui::color::Color, Vec2, Vec3, Vec4};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Vertex {
    pub pos: [f32; 4],

    pub rgba: [f32; 4],

    pub uv: [f32; 2],

    pub texture_index: i32,

    pub _pad: i32,
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            pos: [0.0, 0.0, 0.0, 1.0],
            rgba: [1.0, 1.0, 1.0, 1.0],
            uv: [0.0, 0.0],
            texture_index: 0,
            _pad: 0,
        }
    }
}

impl Vertex {
    pub fn new(pos: Vec3, rgba: Color, uv: Vec2, texture_index: i32) -> Vertex {
        Self {
            pos: [pos.x, pos.y, pos.z, 1.0],
            rgba: [rgba.r, rgba.g, rgba.b, rgba.a],
            uv: [uv.x, uv.y],
            texture_index,
            _pad: 0,
        }
    }
}
