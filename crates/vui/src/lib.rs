#![feature(trait_upcasting)]

pub mod asset_loader;
pub mod errors;
pub mod format;
pub mod graphics;
pub mod math;
pub mod msaa;
pub mod pipeline;
pub mod ui;
pub mod vulkan;

pub type Mat4 = nalgebra::Matrix4<f32>;
pub type Vec2 = nalgebra::Vector2<f32>;
pub type Vec3 = nalgebra::Vector3<f32>;
pub type Vec4 = nalgebra::Vector4<f32>;

#[inline]
pub fn vec2(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

#[inline]
pub fn vec3(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

#[inline]
pub fn vec4(x: f32, y: f32, z: f32, w: f32) -> Vec4 {
    Vec4::new(x, y, z, w)
}

#[macro_export]
macro_rules! builder_field {
    ($field:ident, $field_type:ty) => {
        pub fn $field(self, $field: $field_type) -> Self {
            Self { $field, ..self }
        }
    };
}

#[macro_export]
macro_rules! builder_field_into {
    ($field:ident, $field_type:ty) => {
        pub fn $field<T>(self, $field: T) -> Self
        where
            T: Into<$field_type>,
        {
            Self { $field: $field.into(), ..self }
        }
    };
}

#[macro_export]
macro_rules! builder_field_some {
    ($field:ident, $field_type:ty) => {
        pub fn $field(self, $field: $field_type) -> Self {
            Self { $field: Some($field), ..self }
        }
    };
}
