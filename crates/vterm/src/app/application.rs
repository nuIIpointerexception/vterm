use std::sync::Arc;

use anyhow::{Context, Result};

use vui::{
    asset_loader::AssetLoader,
    errors::FrameError,
    graphics::triangles::Triangles,
    msaa::MSAARenderPass,
    pipeline::FramePipeline,
    timing::FrameRateLimit,
    vulkan::{
        allocator::{create_default_allocator, MemoryAllocator},
        framebuffer::Framebuffer,
        render_device::RenderDevice,
    },
    window::GlfwWindow,
};

use crate::{app::State, constants};

pub struct Application<S: State> {
    state: S,
    fps_limit: FrameRateLimit,
    paused: bool,

    frame_pipeline: FramePipeline,
    ui_layer: Triangles,
    app_layer: Triangles,
    _asset_loader: AssetLoader,
    msaa_renderpass: MSAARenderPass,
    framebuffers: Vec<Framebuffer>,
    swapchain_needs_rebuild: bool,
    vk_dev: Arc<RenderDevice>,
    vk_alloc: Arc<dyn MemoryAllocator>,

    glfw_window: GlfwWindow,
}

impl<S: State> Application<S> {
    pub fn new() -> Result<Self> {
        let mut glfw_window = GlfwWindow::new("Swapchain")?;
        let vk_dev = Arc::new(glfw_window.create_vulkan_device()?);
        let vk_alloc = create_default_allocator(vk_dev.clone());
        let frame_pipeline = FramePipeline::new(vk_dev.clone())?;
        let mut fps_limit = FrameRateLimit::new(60, 30);

        glfw_window.window.set_key_polling(true);
        glfw_window.window.set_framebuffer_size_polling(true);
        glfw_window.window.set_cursor_pos_polling(true);
        glfw_window.window.set_mouse_button_polling(true);

        let msaa_renderpass = MSAARenderPass::for_current_swapchain(
            vk_dev.clone(),
            vk_alloc.clone(),
        )?;
        let framebuffers = msaa_renderpass.create_swapchain_framebuffers()?;
        let mut asset_loader =
            AssetLoader::new(vk_dev.clone(), vk_alloc.clone())?;

        let state = S::init(
            &mut glfw_window,
            &mut fps_limit,
            &mut asset_loader,
            &vk_dev,
            &vk_alloc,
        )?;

        let ui_layer = Triangles::new(
            &msaa_renderpass,
            asset_loader.textures(),
            vk_alloc.clone(),
            vk_dev.clone(),
        )?;
        let app_layer = Triangles::new(
            &msaa_renderpass,
            asset_loader.textures(),
            vk_alloc.clone(),
            vk_dev.clone(),
        )?;

        Ok(Self {
            state,
            fps_limit,
            paused: false,

            frame_pipeline,
            msaa_renderpass,
            framebuffers,
            ui_layer,
            app_layer,
            _asset_loader: asset_loader,
            swapchain_needs_rebuild: true,
            vk_dev,
            vk_alloc,

            glfw_window,
        })
    }

    pub fn run(mut self) -> Result<()> {
        let event_receiver = self.glfw_window.take_event_receiver()?;
        while !self.glfw_window.window.should_close() {
            self.fps_limit.start_frame();
            for (_, event) in
                self.glfw_window.flush_window_events(&event_receiver)
            {
                self.handle_event(event)?;
            }
            if self.swapchain_needs_rebuild {
                self.rebuild_swapchain_resources()?;
                self.swapchain_needs_rebuild = false;
            }
            if !self.paused {
                let result = self.compose_frame();
                match result {
                    Err(FrameError::UnexpectedRuntimeError(_e)) => {
                        self.swapchain_needs_rebuild = true;
                    }
                    Err(FrameError::SwapchainNeedsRebuild) => {
                        self.swapchain_needs_rebuild = true;
                    }
                    _ => result?,
                }
            }
            self.fps_limit.sleep_to_limit();
            let frame_duration = self.fps_limit.avg_frame_time().as_secs_f64();

            constants::set_fps((1.0 / frame_duration) as u64);
        }
        Ok(())
    }

    fn compose_frame(&mut self) -> Result<(), FrameError> {
        let (index, cmds) = self.frame_pipeline.begin_frame()?;

        unsafe {
            self.msaa_renderpass.begin_renderpass_inline(
                cmds,
                &self.framebuffers[index],
                [0.05, 0.05, 0.05, 1.0],
                1.0,
            );
        }

        let mut ui_frame = self
            .ui_layer
            .acquire_frame(index)
            .with_context(|| "unable to acquire ui layer frame")?;

        let mut app_frame = self
            .app_layer
            .acquire_frame(index)
            .with_context(|| "unable to acquire application layer frame")?;

        self.state.draw_frame(&mut app_frame, &mut ui_frame)?;

        unsafe {
            self.app_layer.complete_frame(cmds, app_frame, index)?;
            self.ui_layer.complete_frame(cmds, ui_frame, index)?;
            self.msaa_renderpass.end_renderpass(cmds);
        }
        self.frame_pipeline.end_frame(index)
    }

    fn rebuild_swapchain_resources(&mut self) -> Result<()> {
        if self.paused {
            self.glfw_window.glfw.wait_events();
            return Ok(());
        }
        unsafe {
            self.vk_dev.logical_device.device_wait_idle()?;
        }
        let (w, h) = self.glfw_window.window.get_framebuffer_size();
        self.vk_dev.rebuild_swapchain((w as u32, h as u32))?;
        self.frame_pipeline.rebuild_swapchain_resources()?;

        self.msaa_renderpass = MSAARenderPass::for_current_swapchain(
            self.vk_dev.clone(),
            self.vk_alloc.clone(),
        )?;
        self.framebuffers =
            self.msaa_renderpass.create_swapchain_framebuffers()?;
        self.app_layer
            .rebuild_swapchain_resources(&self.msaa_renderpass)?;
        self.ui_layer
            .rebuild_swapchain_resources(&self.msaa_renderpass)?;

        self.state.rebuild_swapchain_resources(
            &self.glfw_window,
            (w as u32, h as u32),
        )
    }

    fn handle_event(&mut self, event: glfw::WindowEvent) -> Result<()> {
        use glfw::WindowEvent;
        match event {
            WindowEvent::Close => {
                self.glfw_window.window.set_should_close(true);
            }
            WindowEvent::FramebufferSize(w, h) => {
                self.paused = w == 0 || h == 0;
                self.swapchain_needs_rebuild = true;
            }
            _ => {}
        }

        self.state.handle_event(event, &mut self.glfw_window)
    }
}
