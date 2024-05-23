use std::sync::Arc;

use ::ash::vk;

use crate::vulkan::{
    image::{sampler::Sampler, view::ImageView},
    render_device::RenderDevice,
};

pub struct DescriptorSet {
    pub raw: vk::DescriptorSet,

    pub vk_dev: Arc<RenderDevice>,
}

impl DescriptorSet {
    pub unsafe fn bind_buffer(
        &self,
        binding: u32,
        buffer: &vk::Buffer,
        descriptor_type: vk::DescriptorType,
    ) {
        let descriptor_buffer_info = vk::DescriptorBufferInfo {
            buffer: *buffer,
            offset: 0,
            range: vk::WHOLE_SIZE,
        };
        let write = vk::WriteDescriptorSet {
            dst_set: self.raw,
            dst_binding: binding,
            dst_array_element: 0,
            descriptor_count: 1,
            descriptor_type,
            p_image_info: std::ptr::null(),
            p_texel_buffer_view: std::ptr::null(),
            p_buffer_info: &descriptor_buffer_info,
            ..Default::default()
        };
        self.vk_dev
            .logical_device
            .update_descriptor_sets(&[write], &[]);
    }

    pub unsafe fn bind_combined_image_sampler(
        &self,
        binding: u32,
        array_element: u32,
        image_view: &ImageView,
        sampler: &Sampler,
    ) {
        let descriptor_image_info = vk::DescriptorImageInfo {
            sampler: sampler.raw,
            image_view: image_view.raw,
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        };
        let write = vk::WriteDescriptorSet {
            dst_set: self.raw,
            dst_binding: binding,
            dst_array_element: array_element,
            descriptor_count: 1,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            p_image_info: &descriptor_image_info,
            ..Default::default()
        };
        self.vk_dev
            .logical_device
            .update_descriptor_sets(&[write], &[]);
    }
}
