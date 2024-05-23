use std::sync::Arc;

use ::ash::vk;

use crate::{
    errors::VulkanError,
    msaa::MSAARenderPass,
    vulkan::{
        allocator::MemoryAllocator,
        image::{Image, view::ImageView},
        render_device::RenderDevice,
    },
};

impl MSAARenderPass {
    pub(super) fn create_msaa_render_target(
        vk_dev: Arc<RenderDevice>,
        vk_alloc: Arc<dyn MemoryAllocator>,
    ) -> Result<Arc<ImageView>, VulkanError> {
        let samples = Self::pick_max_supported_msaa_count(
            &vk_dev,
            vk::SampleCountFlags::TYPE_4,
        );
        let (swap_extent, format) =
            vk_dev.with_swapchain(|swap| (swap.extent, swap.format));
        let create_info = vk::ImageCreateInfo {
            flags: vk::ImageCreateFlags::empty(),
            image_type: vk::ImageType::TYPE_2D,
            extent: vk::Extent3D {
                width: swap_extent.width,
                height: swap_extent.height,
                depth: 1,
            },
            mip_levels: 1,
            array_layers: 1,
            format,
            samples,
            tiling: vk::ImageTiling::OPTIMAL,
            initial_layout: vk::ImageLayout::UNDEFINED,
            usage: vk::ImageUsageFlags::COLOR_ATTACHMENT
                | vk::ImageUsageFlags::TRANSIENT_ATTACHMENT,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };
        let msaa_render_target = Arc::new(Image::new(
            vk_dev.clone(),
            vk_alloc,
            &create_info,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?);
        let view = Arc::new(ImageView::new_2d(
            msaa_render_target,
            format,
            vk::ImageAspectFlags::COLOR,
        )?);
        Ok(view)
    }

    fn pick_max_supported_msaa_count(
        vk_dev: &RenderDevice,
        desired: vk::SampleCountFlags,
    ) -> vk::SampleCountFlags {
        let props = unsafe {
            vk_dev
                .instance
                .ash
                .get_physical_device_properties(vk_dev.physical_device)
        };
        let supported_samples = props
            .limits
            .framebuffer_color_sample_counts
            .min(props.limits.framebuffer_depth_sample_counts);

        [
            vk::SampleCountFlags::TYPE_64,
            vk::SampleCountFlags::TYPE_32,
            vk::SampleCountFlags::TYPE_16,
            vk::SampleCountFlags::TYPE_8,
            vk::SampleCountFlags::TYPE_4,
            vk::SampleCountFlags::TYPE_2,
        ]
        .iter()
        .find(|&&count| supported_samples.contains(count))
        .map_or(vk::SampleCountFlags::TYPE_1, |&count| desired.min(count))
    }
}
