use std::sync::Arc;

use ash::vk;

use crate::{
    errors::PipelineError,
    vulkan::{
        descriptor_set::DescriptorSetLayout, render_device::RenderDevice,
    },
};

pub struct PipelineLayout {
    pub descriptor_layouts: Vec<Arc<DescriptorSetLayout>>,

    pub raw: vk::PipelineLayout,

    pub vk_dev: Arc<RenderDevice>,
}

impl PipelineLayout {
    pub fn new(
        vk_dev: Arc<RenderDevice>,
        descriptor_layouts: &[Arc<DescriptorSetLayout>],
        push_constant_ranges: &[vk::PushConstantRange],
    ) -> Result<Self, PipelineError> {
        let raw_descriptor_layout_ptrs: Vec<vk::DescriptorSetLayout> =
            descriptor_layouts.iter().map(|layout| layout.raw).collect();
        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {
            p_set_layouts: raw_descriptor_layout_ptrs.as_ptr(),
            set_layout_count: descriptor_layouts.len() as u32,
            p_push_constant_ranges: push_constant_ranges.as_ptr(),
            push_constant_range_count: push_constant_ranges.len() as u32,
            ..Default::default()
        };
        let raw = unsafe {
            vk_dev
                .logical_device
                .create_pipeline_layout(&pipeline_layout_create_info, None)
                .map_err(PipelineError::UnableToCreatePipelineLayout)?
        };
        Ok(Self {
            raw,
            descriptor_layouts: descriptor_layouts.to_owned(),
            vk_dev,
        })
    }
}

impl Drop for PipelineLayout {
    fn drop(&mut self) {
        unsafe {
            self.vk_dev
                .logical_device
                .destroy_pipeline_layout(self.raw, None);
        }
    }
}
