use std::{borrow::BorrowMut, sync::Arc, time::Instant};

use anyhow::{Context, Result};
use ash::Entry;
use log::info;
#[cfg(debug_assertions)]
use logger::{initialize_logger, initialize_panic_hook};
use vui::{
    asset_loader::{self, AssetLoader}, errors::FrameError, graphics::triangles::Triangles, math::projections, msaa::MSAARenderPass, pipeline::FramePipeline, ui::{primitives::Dimensions, UIState, UI}, vulkan::{
        allocator::{create_default_allocator, MemoryAllocator},
        framebuffer::Framebuffer,
        render_device::RenderDevice,
    }, Mat4
};
#[cfg(windows)]
use windows_sys::Win32::System::Console::{
    AttachConsole, FreeConsole, ATTACH_PARENT_PROCESS,
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{DeviceEvent, DeviceId, StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    platform::{
        wayland::EventLoopBuilderExtWayland, x11::EventLoopBuilderExtX11,
    },
    window::{Window, WindowId},
};

use crate::{
    cli::{Args, WindowProtocol},
    terminal::Terminal,
};

mod cli;
mod lifecycle;
#[cfg(debug_assertions)]
mod logger;
mod terminal;

const WINDOW_TITLE: &str = "vterm";
const VULKAN_APP_NAME: &str = "vterm";
const VULKAN_APP_VERSION: (u32, u32, u32) = (0, 0, 0);
const VULKAN_ENGINE_NAME: &str = "viableui";
const VULKAN_ENGINE_VERSION: (u32, u32, u32) = (0, 0, 0);

struct AppState {
    window: Option<Window>,
    last_window_size: Option<PhysicalSize<u32>>,
    last_frame_timestamp: Instant,
    frame_index: usize,
    args: Args,

    frame_pipeline: Option<FramePipeline>,
    frame_layer: Option<Triangles>,
    asset_loader: Option<AssetLoader>,
    msaa_renderpass: Option<MSAARenderPass>,
    framebuffers: Vec<Framebuffer>,
    swapchain_needs_rebuild: bool,
    vk_dev: Option<Arc<RenderDevice>>,
    vk_alloc: Option<Arc<dyn MemoryAllocator>>,
    camera: Mat4,
    root: Option<UI<Terminal>>,
}

impl ApplicationHandler for AppState {
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        if cause == StartCause::Init {
            event_loop.set_control_flow(ControlFlow::Poll);
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title(WINDOW_TITLE)
            .with_resizable(true)
            .with_decorations(false)
            .with_visible(true);
        let window = event_loop.create_window(window_attributes).unwrap();
        let entry = unsafe { Entry::load() }.unwrap();
        let vk_dev = Arc::new(
            lifecycle::create_vulkan_device(&window, entry, &self.args)
                .unwrap(),
        );
        let vk_alloc = create_default_allocator(vk_dev.clone());
        let frame_pipeline = FramePipeline::new(vk_dev.clone()).unwrap();

        let msaa_renderpass = MSAARenderPass::for_current_swapchain(
            vk_dev.clone(),
            vk_alloc.clone(),
        )
        .unwrap();
        let framebuffers =
            msaa_renderpass.create_swapchain_framebuffers().unwrap();

        let mut asset_loader =
            AssetLoader::new(vk_dev.clone(), vk_alloc.clone()).unwrap();
        let (w, h): (u32, u32) = window.inner_size().into();

        let aspect_ratio = w as f32 / h as f32;
        let width = 10.0;
        let height = width * aspect_ratio;
        self.camera = vui::math::projections::ortho(
            -0.5 * width,
            0.5 * width,
            -0.5 * height,
            0.5 * height,
            0.0,
            1.0,
        );

        let root = UI::new(
            Dimensions::new(w as f32, h as f32),
            Terminal::new(1.0, asset_loader.borrow_mut()).unwrap(),
        );
        let frame_layer = Triangles::new(
            &msaa_renderpass,
            asset_loader.textures(),
            vk_alloc.clone(),
            vk_dev.clone(),
        )
        .unwrap();

        self.window = Some(window);
        self.last_window_size =
            Some(self.window.as_ref().unwrap().inner_size());
        self.vk_dev = Some(vk_dev);
        self.vk_alloc = Some(vk_alloc);
        self.frame_pipeline = Some(frame_pipeline);
        self.msaa_renderpass = Some(msaa_renderpass);
        self.framebuffers = framebuffers;
        self.asset_loader = Some(asset_loader);
        self.frame_layer = Some(frame_layer);
        self.root = Some(root);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _: WindowId,
        event: WindowEvent,
    ) {
        if let Some(message) =
            self.root.as_mut().unwrap().handle_event(&event).unwrap()
        {
            self.root.as_mut().unwrap().state_mut().update(&message);
        }

        match event {
            WindowEvent::Resized(new_size) => {
                if Some(new_size) != self.last_window_size {
                    self.last_window_size = Some(new_size);
                    self.swapchain_needs_rebuild = true;
                }
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => (),
        }
    }

    fn device_event(
        &mut self,
        _: &ActiveEventLoop,
        _: DeviceId,
        _event: DeviceEvent,
    ) {
        // TODO: Handle input events
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        let current_frame_timestamp = Instant::now();
        let _delta_time =
            (current_frame_timestamp - self.last_frame_timestamp).as_secs_f32();
        self.last_frame_timestamp = current_frame_timestamp;

        if self.swapchain_needs_rebuild {
            self.rebuild_swapchain_resources().unwrap();
            self.swapchain_needs_rebuild = false;
        }

        let result = self.compose_frame();
        match result {
            Err(FrameError::UnexpectedRuntimeError(_e)) => {
                self.swapchain_needs_rebuild = true;
            }
            Err(FrameError::SwapchainNeedsRebuild) => {
                self.swapchain_needs_rebuild = true;
            }
            _ => result.unwrap(),
        }

        self.window.as_mut().unwrap().set_visible(true);
        self.frame_index += 1;
    }

    fn exiting(&mut self, _: &ActiveEventLoop) {
        // Cleanup resources
    }
}

impl AppState {
    fn compose_frame(&mut self) -> Result<(), FrameError> {
        let (index, cmds) =
            self.frame_pipeline.as_mut().unwrap().begin_frame()?;

        unsafe {
            self.msaa_renderpass
                .as_mut()
                .unwrap()
                .begin_renderpass_inline(
                    cmds,
                    &self.framebuffers[index],
                    [0.05, 0.05, 0.05, 1.0],
                    1.0,
                );
        }

        let mut app_frame = self
            .frame_layer
            .as_mut()
            .unwrap()
            .acquire_frame(index)
            .with_context(|| "unable to acquire root layer frame")?;

        self.root.as_mut().unwrap().draw_frame(&mut app_frame)?;

        unsafe {
            self.frame_layer
                .as_mut()
                .unwrap()
                .complete_frame(cmds, app_frame, index)?;
            self.msaa_renderpass.as_mut().unwrap().end_renderpass(cmds);
        }

        self.frame_pipeline.as_mut().unwrap().end_frame(index)
    }

    fn rebuild_swapchain_resources(&mut self) -> Result<()> {
        unsafe {
            self.vk_dev
                .as_mut()
                .unwrap()
                .logical_device
                .device_wait_idle()?;
        }
        let (w, h): (u32, u32) =
            self.window.as_ref().unwrap().inner_size().into();
        self.vk_dev.as_mut().unwrap().rebuild_swapchain((w, h))?;
        self.frame_pipeline
            .as_mut()
            .unwrap()
            .rebuild_swapchain_resources()?;

        self.msaa_renderpass = Some(MSAARenderPass::for_current_swapchain(
            self.vk_dev.as_mut().unwrap().clone(),
            self.vk_alloc.as_mut().unwrap().clone(),
        )?);
        self.framebuffers = self
            .msaa_renderpass
            .as_mut()
            .unwrap()
            .create_swapchain_framebuffers()?;
        self.frame_layer
            .as_mut()
            .unwrap()
            .rebuild_swapchain_resources(
                self.msaa_renderpass.as_mut().unwrap(),
            )?;

        Ok(())
    }
}

pub fn setup_environment_variables() {
    #[cfg(unix)]
    {
        let terminfo = "xterm-256color";
        info!("[setup_environment_variables] terminfo: {terminfo}");
        std::env::set_var("TERM", terminfo);
    }

    std::env::set_var("TERM_PROGRAM", "vterm");
    std::env::set_var("TERM_PROGRAM_VERSION", env!("CARGO_PKG_VERSION"));

    std::env::set_var("COLORTERM", "truecolor");
    std::env::remove_var("DESKTOP_STARTUP_ID");
    std::env::remove_var("XDG_ACTIVATION_TOKEN");

    // TODO(nuii): add env vars from config...
}

pub fn main() {
    #[cfg(debug_assertions)]
    {
        initialize_logger();
        initialize_panic_hook();
    }

    #[cfg(windows)]
    unsafe {
        AttachConsole(ATTACH_PARENT_PROCESS);
    }

    let args = Args::parse();

    #[cfg(target_os = "linux")]
    {
        if std::path::PathBuf::from("/.flatpak-info").exists() {
            info!("Running in a Flatpak environment");
            // TODO(nuii): flatpak.
        }
    }
    setup_environment_variables();

    let event_loop = create_event_loop(&args);
    let mut app_state = AppState {
        window: None,
        last_window_size: None,
        last_frame_timestamp: Instant::now(),
        frame_index: 0,
        args,
        frame_pipeline: None,
        frame_layer: None,
        asset_loader: None,
        msaa_renderpass: None,
        framebuffers: Vec::new(),
        swapchain_needs_rebuild: false,
        vk_dev: None,
        vk_alloc: None,
        camera: Mat4::identity(),
        root: None,
    };
    event_loop.run_app(&mut app_state).unwrap();

    #[cfg(windows)]
    unsafe {
        FreeConsole();
    }
}

fn create_event_loop(args: &Args) -> EventLoop<()> {
    let mut event_loop = EventLoop::builder();
    match args.window_protocol {
        Some(WindowProtocol::Wayland) => event_loop.with_wayland(),
        Some(WindowProtocol::X11) => event_loop.with_x11(),
        None => &mut event_loop,
    };
    event_loop.build().unwrap()
}
