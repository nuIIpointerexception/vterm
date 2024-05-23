use ash::vk;

use crate::{errors::SwapchainError, vulkan::render_device::RenderDevice};

impl RenderDevice {
    pub(crate) fn create_image_views(
        &self,
        format: vk::Format,
        swapchain_images: &Vec<vk::Image>,
    ) -> Result<Vec<vk::ImageView>, SwapchainError> {
        let mut image_views = vec![];
        for (i, image) in swapchain_images.iter().enumerate() {
            let create_info = vk::ImageViewCreateInfo {
                image: *image,
                format,
                view_type: vk::ImageViewType::TYPE_2D,
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                components: vk::ComponentMapping {
                    r: vk::ComponentSwizzle::IDENTITY,
                    g: vk::ComponentSwizzle::IDENTITY,
                    b: vk::ComponentSwizzle::IDENTITY,
                    a: vk::ComponentSwizzle::IDENTITY,
                },
                ..Default::default()
            };
            let view = unsafe {
                self.logical_device
                    .create_image_view(&create_info, None)
                    .map_err(|err| {
                        SwapchainError::UnableToCreateSwapchainImageView(i, err)
                    })?
            };
            image_views.push(view);
        }

        Ok(image_views)
    }
}
