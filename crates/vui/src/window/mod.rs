use std::sync::mpsc::Receiver;

use ash::{extensions::khr::Surface, vk, vk::Handle};

use crate::{
    errors::WindowError,
    vulkan::{
        instance::Instance, render_device::RenderDevice,
        window_surface::WindowSurface,
    },
};

pub type EventReceiver = Receiver<(f64, glfw::WindowEvent)>;

pub struct GlfwWindow {
    pub glfw: glfw::Glfw,
    pub window: glfw::Window,
    event_receiver: Option<EventReceiver>,
    window_pos: (i32, i32),
    window_size: (i32, i32),
    pub refresh_rate: u32,
    pub video_mode: glfw::VidMode,
}

impl GlfwWindow {
    pub fn new(window_title: &str) -> Result<Self, WindowError> {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)?;

        if !glfw.vulkan_supported() {
            return Err(WindowError::VulkanNotSupported);
        }

        glfw.window_hint(glfw::WindowHint::ClientApi(
            glfw::ClientApiHint::NoApi,
        ));
        glfw.window_hint(glfw::WindowHint::ScaleToMonitor(true));

        let (video_mode, refresh_rate) =
            Self::get_primary_monitor_video_mode(&mut glfw)?;

        let initial_size = (1280, 720);

        let (mut window, event_receiver) = glfw
            .create_window(
                initial_size.0,
                initial_size.1,
                window_title,
                glfw::WindowMode::Windowed,
            )
            .ok_or(WindowError::WindowCreateFailed)?;

        let window_pos = window.get_pos();
        let window_size = window.get_size();

        window.set_size_limits(Some(320), Some(225), None, None);

        Ok(Self {
            glfw,
            window,
            event_receiver: Some(event_receiver),
            window_pos,
            window_size,
            refresh_rate,
            video_mode,
        })
    }

    fn get_primary_monitor_video_mode(
        glfw: &mut glfw::Glfw,
    ) -> Result<(glfw::VidMode, u32), WindowError> {
        glfw.with_primary_monitor_mut(
            |_, primary| -> Result<(glfw::VidMode, u32), WindowError> {
                let monitor = primary.ok_or(WindowError::NoPrimaryMonitor)?;
                let video_mode = monitor
                    .get_video_mode()
                    .ok_or(WindowError::PrimaryVideoModeMissing)?;
                Ok((video_mode, video_mode.refresh_rate))
            },
        )
    }

    pub fn take_event_receiver(
        &mut self,
    ) -> Result<EventReceiver, WindowError> {
        self.event_receiver
            .take()
            .ok_or(WindowError::EventReceiverLost)
    }

    pub fn flush_window_events<'events>(
        &mut self,
        event_receiver: &'events EventReceiver,
    ) -> glfw::FlushedMessages<'events, (f64, glfw::WindowEvent)> {
        self.glfw.poll_events();
        glfw::flush_messages(event_receiver)
    }

    pub fn toggle_fullscreen(&mut self) -> Result<(), WindowError> {
        use glfw::WindowMode;

        let is_fullscreen = self
            .window
            .with_window_mode(|mode| matches!(mode, WindowMode::FullScreen(_)));

        if is_fullscreen {
            let (x, y) = self.window_pos;
            let (w, h) = self.window_size;
            self.window.set_monitor(
                WindowMode::Windowed,
                x,
                y,
                w as u32,
                h as u32,
                None,
            );
        } else {
            self.window_size = self.window.get_size();
            self.window_pos = self.window.get_pos();
            self.glfw.with_primary_monitor_mut(
                |_, monitor_opt| -> Result<(), WindowError> {
                    let monitor =
                        monitor_opt.ok_or(WindowError::NoPrimaryMonitor)?;
                    let video_mode = monitor
                        .get_video_mode()
                        .ok_or(WindowError::PrimaryVideoModeMissing)?;
                    self.window.set_monitor(
                        WindowMode::FullScreen(monitor),
                        0,
                        0,
                        video_mode.width,
                        video_mode.height,
                        Some(video_mode.refresh_rate),
                    );
                    Ok(())
                },
            )?;
        }
        Ok(())
    }

    pub fn create_vulkan_device(&self) -> Result<RenderDevice, WindowError> {
        let required_extensions = self
            .glfw
            .get_required_instance_extensions()
            .ok_or(WindowError::RequiredExtensionsUnavailable)?;
        let instance = Instance::new(&required_extensions)?;

        let mut surface_handle: u64 = 0;
        let result = vk::Result::from_raw(self.window.create_window_surface(
            instance.ash.handle().as_raw() as usize,
            std::ptr::null(),
            &mut surface_handle,
        ) as i32);
        if result != vk::Result::SUCCESS {
            return Err(WindowError::UnableToCreateSurface(result));
        }

        let window_surface = WindowSurface::new(
            vk::SurfaceKHR::from_raw(surface_handle),
            Surface::new(&instance.entry, &instance.ash),
        );

        let device = RenderDevice::new(instance, window_surface)
            .map_err(WindowError::UnexpectedRenderDeviceError)?;

        let (w, h) = self.window.get_framebuffer_size();
        device.rebuild_swapchain((w as u32, h as u32))?;

        Ok(device)
    }
}
