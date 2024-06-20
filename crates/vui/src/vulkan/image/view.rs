use std::sync::Arc;

use ash::vk;

use crate::vulkan::{
    image::{Image, ImageError},
    render_device::RenderDevice,
};

pub struct ImageView {
    pub raw: vk::ImageView,

    pub image: Arc<Image>,

    pub vk_dev: Arc<RenderDevice>,
}

impl ImageView {
    pub fn new(
        image: Arc<Image>,
        create_info: &vk::ImageViewCreateInfo,
    ) -> Result<Self, ImageError> {
        let raw = unsafe {
            image
                .vk_dev
                .logical_device
                .create_image_view(create_info, None)
                .map_err(ImageError::UnableToCreateView)?
        };
        Ok(Self { raw, vk_dev: image.vk_dev.clone(), image })
    }

    pub fn new_2d(
        image: Arc<Image>,
        format: vk::Format,
        aspect_mask: vk::ImageAspectFlags,
    ) -> Result<Self, ImageError> {
        let create_info = vk::ImageViewCreateInfo {
            flags: vk::ImageViewCreateFlags::empty(),
            image: image.raw,
            view_type: vk::ImageViewType::TYPE_2D,
            format,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            ..Default::default()
        };
        Self::new(image, &create_info)
    }
}

impl Drop for ImageView {
    fn drop(&mut self) {
        unsafe {
            self.vk_dev.logical_device.destroy_image_view(self.raw, None);
        }
    }
}
