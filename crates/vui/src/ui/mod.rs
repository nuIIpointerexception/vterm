use crate::{math, Mat4};

pub mod color;
pub mod font;
mod id;
mod input;
mod internal_state;
pub mod primitives;
mod ui;
pub mod widgets;

pub use self::{
    font::Font,
    id::{id_hash, Id},
    input::Input,
    internal_state::InternalState,
    ui::{UIState, UI},
};

pub fn ui_screen_space_projection(viewport: primitives::Dimensions) -> Mat4 {
    math::projections::ortho(0.0, viewport.width, viewport.height, 0.0, 0.0, 1.0)
}
