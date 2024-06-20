use crate::{vec2, Vec2};

#[derive(Debug, Copy, Clone)]
pub struct Input {
    pub mouse_position: Vec2,
}

impl Input {
    pub fn new() -> Self {
        Self { mouse_position: vec2(0.0, 0.0) }
    }

    pub fn handle_event(&mut self, event: &winit::event::WindowEvent) {
        match event {
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position = vec2(position.x as f32, position.y as f32);
            }
            _ => {}
        }
    }
}
