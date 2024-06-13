use ash::vk;

use crate::{
    errors::SwapchainError, markdown::MdList,
    vulkan::render_device::RenderDevice,
};

impl RenderDevice {
    pub(crate) fn choose_image_count(&self) -> Result<u32, SwapchainError> {
        let capabilities = unsafe {
            self.window_surface
                .surface_capabilities(&self.physical_device)?
        };

        let proposed_image_count = capabilities.min_image_count + 1;
        if capabilities.max_image_count > 0 {
            Ok(std::cmp::min(
                proposed_image_count,
                capabilities.max_image_count,
            ))
        } else {
            Ok(proposed_image_count)
        }
    }

    pub(crate) fn choose_surface_format(&self) -> vk::SurfaceFormatKHR {
        let formats = unsafe {
            self.window_surface.supported_formats(&self.physical_device)
        };

        log::debug!("available formats: {:#?}", MdList(&formats));

        let format = formats
            .iter()
            .cloned()
            .find(|format| {
                format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
                    && format.format == vk::Format::B8G8R8A8_SRGB
            })
            .unwrap_or_else(|| formats[0]);

        log::debug!("chosen format {:#?}", format);

        format
    }

    pub(crate) fn choose_present_mode(&self) -> vk::PresentModeKHR {
        let modes = unsafe {
            self.window_surface
                .supported_presentation_modes(&self.physical_device)
        };

        // prefer immediate mode, but fallback to mailbox or fifo
        let mode = modes
            .iter()
            .cloned()
            .find(|&m| m == vk::PresentModeKHR::IMMEDIATE)
            .unwrap_or_else(|| {
                if modes.contains(&vk::PresentModeKHR::MAILBOX) {
                    vk::PresentModeKHR::MAILBOX
                } else {
                    vk::PresentModeKHR::FIFO
                }
            });

        mode
    }

    pub(crate) fn choose_swap_extent(
        &self,
        framebuffer_size: (u32, u32),
    ) -> Result<vk::Extent2D, SwapchainError> {
        let capabilities = unsafe {
            self.window_surface
                .surface_capabilities(&self.physical_device)?
        };

        if capabilities.current_extent.width != u32::MAX {
            Ok(capabilities.current_extent)
        } else {
            let (width, height) = framebuffer_size;
            let extent = vk::Extent2D {
                width: width.clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: height.clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            };
            Ok(extent)
        }
    }
}
