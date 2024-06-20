use std::sync::Arc;

use ::ash::vk;

pub use self::frame::Frame;
pub use crate::errors::ImmediateModeGraphicsError;
use crate::{
    asset_loader::CombinedImageSampler,
    errors::VulkanError,
    msaa::MSAARenderPass,
    vulkan::{
        allocator::MemoryAllocator, command_buffer::CommandBuffer, pipeline::Pipeline,
        render_device::RenderDevice,
    },
};

mod frame;
mod pipeline;

pub struct Triangles {
    textures: Vec<CombinedImageSampler>,

    pipeline: Pipeline,

    frames: Vec<Option<Frame>>,

    vk_alloc: Arc<dyn MemoryAllocator>,

    vk_dev: Arc<RenderDevice>,
}

impl Triangles {
    pub fn new(
        msaa_renderpass: &MSAARenderPass,
        textures: &[CombinedImageSampler],
        vk_alloc: Arc<dyn MemoryAllocator>,
        vk_dev: Arc<RenderDevice>,
    ) -> Result<Self, VulkanError> {
        let pipeline = pipeline::create_pipeline(
            msaa_renderpass,
            textures.len() as u32,
            false,
            vk_dev.clone(),
        )?;
        let frames = {
            let mut frames = vec![];
            for _ in 0 .. vk_dev.swapchain_image_count() {
                let frame = Frame::new(
                    vk_dev.clone(),
                    vk_alloc.clone(),
                    textures,
                    &pipeline.pipeline_layout.descriptor_layouts[0],
                )?;
                frames.push(Some(frame));
            }
            frames
        };
        Ok(Self { textures: textures.to_owned(), pipeline, frames, vk_alloc, vk_dev })
    }

    pub fn rebuild_swapchain_resources(
        &mut self,
        msaa_renderpass: &MSAARenderPass,
    ) -> Result<(), VulkanError> {
        self.pipeline = pipeline::create_pipeline(
            msaa_renderpass,
            self.textures.len() as u32,
            false,
            self.vk_dev.clone(),
        )?;
        self.frames = {
            let mut frames = vec![];
            for _ in 0 .. self.vk_dev.swapchain_image_count() {
                let frame = Frame::new(
                    self.vk_dev.clone(),
                    self.vk_alloc.clone(),
                    &self.textures,
                    &self.pipeline.pipeline_layout.descriptor_layouts[0],
                )?;
                frames.push(Some(frame));
            }
            frames
        };
        Ok(())
    }

    pub fn acquire_frame(
        &mut self,
        swapchain_image_index: usize,
    ) -> Result<Frame, ImmediateModeGraphicsError> {
        let mut frame = self.frames[swapchain_image_index]
            .take()
            .ok_or(ImmediateModeGraphicsError::FrameResourcesUnavailable(swapchain_image_index))?;
        frame.clear();
        Ok(frame)
    }

    /// # Safety
    ///
    /// The caller must ensure that the command buffer is in the recording
    /// state.
    pub unsafe fn complete_frame(
        &mut self,
        cmd: &CommandBuffer,
        mut frame: Frame,
        swapchain_image_index: usize,
    ) -> Result<(), VulkanError> {
        self.vk_dev.logical_device.cmd_bind_pipeline(
            cmd.raw,
            vk::PipelineBindPoint::GRAPHICS,
            self.pipeline.raw,
        );
        frame.write_frame_commands(cmd, &self.pipeline.pipeline_layout);
        self.frames[swapchain_image_index] = Some(frame);
        Ok(())
    }
}
