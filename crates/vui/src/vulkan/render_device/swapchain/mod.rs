use ::ash::{khr, vk};

use crate::{errors::SwapchainError, vulkan::render_device::RenderDevice};

mod images;
mod selection;

pub struct Swapchain {
    pub loader: khr::swapchain::Device,

    pub khr: vk::SwapchainKHR,

    pub image_views: Vec<vk::ImageView>,

    pub format: vk::Format,

    pub color_space: vk::ColorSpaceKHR,

    pub extent: vk::Extent2D,
}

impl Swapchain {
    /// Get the width of the swapchain image.
    pub fn width(&self) -> u32 {
        self.extent.width
    }

    /// Get the height of the swapchain image.
    pub fn height(&self) -> u32 {
        self.extent.height
    }
}

impl RenderDevice {
    pub fn with_swapchain<ReturnType, Func>(&self, func: Func) -> ReturnType
    where
        Func: FnOnce(&Swapchain) -> ReturnType,
    {
        let swapchain = self
            .swapchain
            .lock()
            .expect("Unable to lock the swapchain mutex");
        let borrow = swapchain.as_ref().expect("The swapchain does not exist");
        func(borrow)
    }

    pub fn swapchain_image_count(&self) -> u32 {
        self.with_swapchain(|swapchain| swapchain.image_views.len() as u32)
    }

    pub fn acquire_next_swapchain_image(
        &self,
        semaphore: vk::Semaphore,
        fence: vk::Fence,
    ) -> Result<usize, SwapchainError> {
        self.with_swapchain(|swapchain| {
            let result = unsafe {
                swapchain.loader.acquire_next_image(
                    swapchain.khr,
                    u64::MAX,
                    semaphore,
                    fence,
                )
            };
            if let Err(vk::Result::ERROR_OUT_OF_DATE_KHR) = result {
                return Err(SwapchainError::NeedsRebuild);
            }
            if let Ok((_, true)) = result {
                return Err(SwapchainError::NeedsRebuild);
            }
            let (index, _) = result.ok().unwrap();
            Ok(index as usize)
        })
    }

    pub fn rebuild_swapchain(
        &self,
        framebuffer_size: (u32, u32),
    ) -> Result<(), SwapchainError> {
        let mut current_swapchain = self
            .swapchain
            .lock()
            .expect("Unable to lock the swapchain mutex");

        let format = self.choose_surface_format();
        let present_mode = self.choose_present_mode();
        let extent = self.choose_swap_extent(framebuffer_size)?;
        let image_count = self.choose_image_count()?;

        let mut create_info = vk::SwapchainCreateInfoKHR {
            surface: self.window_surface.khr,

            image_format: format.format,
            image_color_space: format.color_space,
            image_extent: extent,
            min_image_count: image_count,
            image_array_layers: 1,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,

            present_mode,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            pre_transform: vk::SurfaceTransformFlagsKHR::IDENTITY,
            old_swapchain: if current_swapchain.is_some() {
                current_swapchain.as_ref().unwrap().khr
            } else {
                vk::SwapchainKHR::null()
            },
            clipped: 1,
            ..Default::default()
        };

        let indices =
            &[self.graphics_queue.family_id, self.present_queue.family_id];

        if self.present_queue.is_same(&self.graphics_queue) {
            create_info.image_sharing_mode = vk::SharingMode::EXCLUSIVE;
        } else {
            create_info.image_sharing_mode = vk::SharingMode::CONCURRENT;
            create_info.p_queue_family_indices = indices.as_ptr();
            create_info.queue_family_index_count = indices.len() as u32;
        };

        let loader =
            khr::swapchain::Device::new(&self.instance.ash, &self.logical_device);
        let swapchain = unsafe {
            loader
                .create_swapchain(&create_info, None)
                .map_err(SwapchainError::UnableToCreateSwapchain)?
        };

        let swapchain_images = unsafe {
            loader
                .get_swapchain_images(swapchain)
                .map_err(SwapchainError::UnableToGetSwapchainImages)?
        };

        let image_views =
            self.create_image_views(format.format, &swapchain_images)?;

        let previous_swapchain = current_swapchain.replace(Swapchain {
            loader,
            khr: swapchain,
            image_views,
            format: format.format,
            color_space: format.color_space,
            extent,
        });

        if let Some(old_swapchain) = previous_swapchain {
            unsafe { self.destroy_swapchain(old_swapchain)? };
        }

        Ok(())
    }

    pub(crate) unsafe fn destroy_swapchain(
        &self,
        swapchain: Swapchain,
    ) -> Result<(), SwapchainError> {
        self.logical_device
            .queue_wait_idle(self.graphics_queue.queue)
            .map_err(SwapchainError::UnableToDrainGraphicsQueue)?;
        self.logical_device
            .queue_wait_idle(self.present_queue.queue)
            .map_err(SwapchainError::UnableToDrainPresentQueue)?;
        self.logical_device
            .device_wait_idle()
            .map_err(SwapchainError::UnableToWaitForDeviceIdle)?;

        for view in swapchain.image_views {
            self.logical_device.destroy_image_view(view, None);
        }

        swapchain.loader.destroy_swapchain(swapchain.khr, None);

        Ok(())
    }
}
