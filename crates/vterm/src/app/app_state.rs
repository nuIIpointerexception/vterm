use ::std::sync::Arc;

use vui::{
    asset_loader::AssetLoader,
    graphics::triangles::Frame,
    timing::FrameRateLimit,
    vulkan::{allocator::MemoryAllocator, render_device::RenderDevice},
    window::GlfwWindow,
};

pub trait State {
    fn init(
        window: &mut GlfwWindow,
        fps_limit: &mut FrameRateLimit,
        asset_loader: &mut AssetLoader,
        vk_dev: &Arc<RenderDevice>,
        vk_alloc: &Arc<dyn MemoryAllocator>,
    ) -> anyhow::Result<Self>
    where
        Self: Sized;

    fn draw_frame(
        &mut self,
        app_frame: &mut Frame,
        ui_frame: &mut Frame,
    ) -> anyhow::Result<()>;

    fn rebuild_swapchain_resources(
        &mut self,
        _window: &GlfwWindow,
        _framebuffer_size: (u32, u32),
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn handle_event(
        &mut self,
        _event: glfw::WindowEvent,
        _window: &mut GlfwWindow,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
