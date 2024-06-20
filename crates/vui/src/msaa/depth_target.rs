use std::sync::Arc;

use ::ash::vk;

use crate::{
    errors::VulkanError,
    msaa::{MSAAError, MSAARenderPass},
    vulkan::{
        allocator::MemoryAllocator,
        image::{view::ImageView, Image},
        render_device::RenderDevice,
    },
};

impl MSAARenderPass {
    pub(super) fn create_depth_target(
        msaa_render_target: &ImageView,
        vk_dev: Arc<RenderDevice>,
        vk_alloc: Arc<dyn MemoryAllocator>,
    ) -> Result<Arc<ImageView>, MSAAError> {
        let format = Self::take_first_supported_depth_format(
            &vk_dev,
            &[
                vk::Format::D32_SFLOAT_S8_UINT,
                vk::Format::D24_UNORM_S8_UINT,
                vk::Format::D16_UNORM_S8_UINT,
            ],
        )?;
        let create_info = vk::ImageCreateInfo {
            flags: vk::ImageCreateFlags::empty(),
            image_type: vk::ImageType::TYPE_2D,
            extent: msaa_render_target.image.create_info.extent,
            mip_levels: 1,
            array_layers: 1,
            format,
            samples: msaa_render_target.image.create_info.samples,
            tiling: vk::ImageTiling::OPTIMAL,
            initial_layout: vk::ImageLayout::UNDEFINED,
            usage: vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT |
                vk::ImageUsageFlags::TRANSIENT_ATTACHMENT,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };
        let depth_target = Arc::new(
            Image::new(
                vk_dev.clone(),
                vk_alloc,
                &create_info,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            )
            .map_err(VulkanError::ImageError)?,
        );
        let view = Arc::new(
            ImageView::new_2d(
                depth_target,
                format,
                vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL,
            )
            .map_err(VulkanError::ImageError)?,
        );
        Ok(view)
    }

    fn take_first_supported_depth_format(
        vk_dev: &RenderDevice,
        candidates: &[vk::Format],
    ) -> Result<vk::Format, MSAAError> {
        for format in candidates {
            let format_properties = unsafe {
                vk_dev
                    .instance
                    .ash
                    .get_physical_device_format_properties(vk_dev.physical_device, *format)
            };
            if (format_properties.optimal_tiling_features &
                vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT) ==
                vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT
            {
                return Ok(*format);
            }
        }

        Err(MSAAError::UnableToPickDepthFormat)
    }
}
