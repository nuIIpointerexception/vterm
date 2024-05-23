use std::{borrow::BorrowMut, env, path::PathBuf, sync::Arc};

use anyhow::Result;

use app::{run_application, State};
use terminal::{Terminal, TerminalMessage};
use vui::{
    asset_loader::AssetLoader,
    graphics::{Sprite, triangles::Frame},
    Mat4,
    math::projections,
    timing::FrameRateLimit,
    ui::UI,
    vulkan::{allocator::MemoryAllocator, render_device::RenderDevice},
    window::GlfwWindow,
};

mod app;
mod constants;
mod terminal;

struct App {
    sprite_texture: i32,
    ui: UI<Terminal>,
    app_camera: Mat4,
    rotation_angle: f32,
}

impl App {
    fn projection(aspect_ratio: f32) -> Mat4 {
        let height = 10.0;
        let width = height * aspect_ratio;
        projections::ortho(
            -0.5 * width,
            0.5 * width,
            -0.5 * height,
            0.5 * height,
            0.0,
            1.0,
        )
    }
}

impl State for App {
    fn init(
        window: &mut GlfwWindow,
        fps_limit: &mut FrameRateLimit,
        asset_loader: &mut AssetLoader,
        vk_dev: &Arc<RenderDevice>,
        _vk_alloc: &Arc<dyn MemoryAllocator>,
    ) -> Result<Self> {
        let scale = window.window.get_content_scale();
        let (w, h) = window.window.get_framebuffer_size();
        let aspect_ratio = w as f32 / h as f32;

        constants::set_gpu(vk_dev.clone());

        fps_limit.set_target_fps(window.refresh_rate);

        let current_dir = env::current_dir()?;
        let mut file_path = PathBuf::from(current_dir);
        file_path.push("assets/rust.png");
        let sprite_texture = asset_loader.read_texture(file_path)?;

        Ok(Self {
            sprite_texture,
            ui: UI::new(
                window.window.get_framebuffer_size().into(),
                Terminal::new(scale.0, asset_loader)?,
            ),
            app_camera: Self::projection(aspect_ratio),
            rotation_angle: 0.0,
        })
    }

    fn draw_frame(
        &mut self,
        app_frame: &mut Frame,
        ui_frame: &mut Frame,
    ) -> Result<()> {
        self.ui.draw_frame(ui_frame)?;

        app_frame.set_view_projection(self.app_camera)?;

        Sprite {
            width: 6.0,
            height: 6.0,
            texture_index: self.sprite_texture,
            angle_in_radians: self.rotation_angle,
            ..Default::default()
        }
        .draw(app_frame)?;

        self.rotation_angle += 0.01;

        Ok(())
    }

    fn handle_event(
        &mut self,
        event: glfw::WindowEvent,
        window: &mut GlfwWindow,
    ) -> Result<()> {
        match self.ui.handle_event(&event)? {
            Some(TerminalMessage::ToggleFullscreen) => {
                window.toggle_fullscreen()?
            }
            _ => (),
        }

        match event {
            glfw::WindowEvent::FramebufferSize(w, h) => {
                self.app_camera = Self::projection(w as f32 / h as f32);
            }
            _ => (),
        }

        Ok(())
    }
}

impl App {}

fn main() -> Result<()> {
    run_application::<App>()
}
