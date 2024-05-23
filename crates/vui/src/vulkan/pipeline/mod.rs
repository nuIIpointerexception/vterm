use std::sync::Arc;

use ash::vk;

use crate::{
    errors::PipelineError,
    vulkan::{pipeline::layout::PipelineLayout, render_device::RenderDevice},
};

pub mod layout;
pub mod shader;

pub struct Pipeline {
    pub pipeline_layout: Arc<PipelineLayout>,

    pub raw: vk::Pipeline,

    pub bind_point: vk::PipelineBindPoint,

    pub vk_dev: Arc<RenderDevice>,
}

impl Pipeline {
    pub fn new_graphics_pipeline(
        create_info: vk::GraphicsPipelineCreateInfo,
        pipeline_layout: Arc<PipelineLayout>,
        vk_dev: Arc<RenderDevice>,
    ) -> Result<Pipeline, PipelineError> {
        let raw = unsafe {
            vk_dev
                .logical_device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[create_info],
                    None,
                )
                .map_err(|(_, err)| {
                    PipelineError::UnableToCreateGraphicsPipeline(err)
                })?[0]
        };
        Ok(Self {
            pipeline_layout,
            raw,
            bind_point: vk::PipelineBindPoint::GRAPHICS,
            vk_dev,
        })
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        unsafe {
            self.vk_dev.logical_device.destroy_pipeline(self.raw, None);
        }
    }
}
