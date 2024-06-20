use std::sync::Arc;

use ::anyhow::Context;
use ash::vk;

use crate::{
    errors::{FenceError, FrameError, SwapchainError, VulkanError},
    pipeline::PerFrame,
    vulkan::{command_buffer::CommandBuffer, render_device::RenderDevice, sync::SemaphorePool},
};

pub struct FramePipeline {
    frames: Vec<PerFrame>,
    semaphore_pool: SemaphorePool,

    pub vk_dev: Arc<RenderDevice>,
}

impl FramePipeline {
    pub fn new(vk_dev: Arc<RenderDevice>) -> Result<Self, FrameError> {
        let mut frame_pipeline =
            Self { frames: vec![], semaphore_pool: SemaphorePool::new(vk_dev.clone()), vk_dev };
        frame_pipeline.rebuild_swapchain_resources()?;
        Ok(frame_pipeline)
    }

    pub fn begin_frame(&mut self) -> Result<(usize, &CommandBuffer), FrameError> {
        let current_image = self.acquire_next_image()?;
        let cmd = self.prepare_frame_command_buffer(current_image)?;
        Ok((current_image, cmd))
    }

    pub fn frame_cmds(&self, current_image: usize) -> &CommandBuffer {
        &self.frames[current_image].command_buffer
    }

    pub fn end_frame(&mut self, current_image: usize) -> Result<(), FrameError> {
        self.submit_and_present(current_image)?;
        Ok(())
    }

    pub fn rebuild_swapchain_resources(&mut self) -> Result<(), FrameError> {
        for frame in self.frames.drain(..) {
            frame.queue_submit_fence.wait_and_reset().map_err(VulkanError::FenceError)?;
        }
        for _i in 0 .. self.vk_dev.swapchain_image_count() {
            let frame = PerFrame::new(self.vk_dev.clone())?;
            self.frames.push(frame);
        }
        Ok(())
    }

    pub fn wait_for_all_frames(&mut self) -> Result<(), FenceError> {
        for frame in &mut self.frames {
            frame.queue_submit_fence.wait_and_reset()?;
        }
        Ok(())
    }
}

impl FramePipeline {
    fn acquire_next_image(&mut self) -> Result<usize, FrameError> {
        let acquire_semaphore = self
            .semaphore_pool
            .get_semaphore()
            .context("unable to get a semaphore for the next swapchain image")?;
        let index = {
            let result =
                self.vk_dev.acquire_next_swapchain_image(acquire_semaphore.raw, vk::Fence::null());
            if let Err(SwapchainError::NeedsRebuild) = result {
                return Err(FrameError::SwapchainNeedsRebuild);
            }
            result.context("unable to acquire the next swapchain image")?
        };

        let old_semaphore = self.frames[index].acquire_semaphore.replace(acquire_semaphore);
        if let Some(semaphore) = old_semaphore {
            self.semaphore_pool.return_semaphore(semaphore);
        }

        self.frames[index].queue_submit_fence.wait_and_reset().map_err(VulkanError::FenceError)?;

        self.frames[index].command_pool.reset().map_err(VulkanError::CommandBufferError)?;

        Ok(index)
    }

    fn prepare_frame_command_buffer(
        &mut self,
        current_image: usize,
    ) -> Result<&CommandBuffer, FrameError> {
        let current_frame = &self.frames[current_image];
        unsafe {
            current_frame.command_buffer.begin_one_time_submit().with_context(|| {
                format!("Unable to begin the command buffer for frame {}", current_image)
            })?;
        }
        Ok(&current_frame.command_buffer)
    }

    fn submit_and_present(&mut self, index: usize) -> Result<(), FrameError> {
        let current_frame = &self.frames[index];
        unsafe {
            current_frame
                .command_buffer
                .end_commands()
                .with_context(|| format!("Unable to end command buffer for frame {}", index))?;
        }

        let wait_stage = vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;
        let submit_info = vk::SubmitInfo {
            command_buffer_count: 1,
            p_command_buffers: &current_frame.command_buffer.raw,
            wait_semaphore_count: 1,
            p_wait_semaphores: &current_frame.acquire_semaphore.as_ref().unwrap().raw,
            p_wait_dst_stage_mask: &wait_stage,
            signal_semaphore_count: 1,
            p_signal_semaphores: &current_frame.release_semaphore.raw,
            ..Default::default()
        };
        unsafe {
            self.vk_dev
                .logical_device
                .queue_submit(
                    self.vk_dev.graphics_queue.queue,
                    &[submit_info],
                    current_frame.queue_submit_fence.raw,
                )
                .with_context(|| {
                    format!("Unable to submit graphics commands on frame {}", index)
                })?;
        }

        let index_u32 = index as u32;
        let current_frame = &self.frames[index];

        self.vk_dev.with_swapchain(|swapchain| {
            let present_info = vk::PresentInfoKHR {
                swapchain_count: 1,
                p_swapchains: &swapchain.khr,
                p_image_indices: &index_u32,
                wait_semaphore_count: 1,
                p_wait_semaphores: &current_frame.release_semaphore.raw,
                ..Default::default()
            };
            unsafe {
                swapchain
                    .loader
                    .queue_present(self.vk_dev.present_queue.queue, &present_info)
                    .with_context(|| "Unable to present the swapchain image")
            }
        })?;
        Ok(())
    }
}

impl Drop for FramePipeline {
    fn drop(&mut self) {
        self.wait_for_all_frames().expect("Unable to wait for all frames to complete!");
    }
}
