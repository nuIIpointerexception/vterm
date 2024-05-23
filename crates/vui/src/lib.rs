mod markdown;

pub mod asset_loader;
pub mod errors;
pub mod graphics;
pub mod math;
pub mod msaa;
pub mod pipeline;
pub mod timing;
pub mod ui;
pub mod vulkan;
pub mod vulkan_ext;
pub mod window;

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

#[allow(unused)]
macro_rules! builder_field {
    ($field:ident, $field_type:ty) => {
        pub fn $field(self, $field: $field_type) -> Self {
            Self { $field, ..self }
        }
    };
}

#[allow(unused)]
macro_rules! builder_field_into {
    ($field:ident, $field_type:ty) => {
        pub fn $field<T>(self, $field: T) -> Self
        where
            T: Into<$field_type>,
        {
            Self {
                $field: $field.into(),
                ..self
            }
        }
    };
}

#[allow(unused)]
macro_rules! builder_field_some {
    ($field:ident, $field_type:ty) => {
        pub fn $field(self, $field: $field_type) -> Self {
            Self {
                $field: Some($field),
                ..self
            }
        }
    };
}

#[allow(unused)]
pub(crate) use {builder_field, builder_field_into, builder_field_some};
