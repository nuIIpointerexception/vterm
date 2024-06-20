use std::sync::Arc;

use ::ash::vk;

use crate::{
    errors::CommandBufferError,
    vulkan::render_device::{GpuQueue, RenderDevice},
};

pub struct CommandPool {
    pub raw: vk::CommandPool,

    pub vk_dev: Arc<RenderDevice>,
}

impl CommandPool {
    pub fn new(
        vk_dev: Arc<RenderDevice>,
        queue: &GpuQueue,
        flags: vk::CommandPoolCreateFlags,
    ) -> Result<Self, CommandBufferError> {
        let raw = {
            let create_info = vk::CommandPoolCreateInfo {
                queue_family_index: queue.family_id,
                flags,
                ..Default::default()
            };
            unsafe {
                vk_dev
                    .logical_device
                    .create_command_pool(&create_info, None)
                    .map_err(CommandBufferError::UnableToCreateCommandPool)?
            }
        };
        Ok(Self { raw, vk_dev })
    }

    pub fn new_transient_graphics_pool(
        vk_dev: Arc<RenderDevice>,
    ) -> Result<Self, CommandBufferError> {
        Self::new(vk_dev.clone(), &vk_dev.graphics_queue, vk::CommandPoolCreateFlags::TRANSIENT)
    }

    pub unsafe fn allocate_command_buffers(
        &self,
        level: vk::CommandBufferLevel,
        command_buffer_count: u32,
    ) -> Result<Vec<vk::CommandBuffer>, CommandBufferError> {
        let create_info = vk::CommandBufferAllocateInfo {
            command_pool: self.raw,
            level,
            command_buffer_count,
            ..Default::default()
        };
        let buffer = self
            .vk_dev
            .logical_device
            .allocate_command_buffers(&create_info)
            .map_err(CommandBufferError::UnableToAllocateBuffer)?;
        Ok(buffer)
    }

    pub unsafe fn allocate_command_buffer(
        &self,
        level: vk::CommandBufferLevel,
    ) -> Result<vk::CommandBuffer, CommandBufferError> {
        let buffers = self.allocate_command_buffers(level, 1)?;
        Ok(buffers[0])
    }

    pub unsafe fn free_command_buffers(&self, command_buffers: &[vk::CommandBuffer]) {
        self.vk_dev.logical_device.free_command_buffers(self.raw, command_buffers);
    }

    pub unsafe fn free_command_buffer(&self, command_buffer: vk::CommandBuffer) {
        self.free_command_buffers(&[command_buffer]);
    }

    pub fn reset(&self) -> Result<(), CommandBufferError> {
        unsafe {
            self.vk_dev
                .logical_device
                .reset_command_pool(self.raw, vk::CommandPoolResetFlags::empty())
                .map_err(CommandBufferError::UnableToResetPool)?;
        }
        Ok(())
    }
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        unsafe { self.vk_dev.logical_device.destroy_command_pool(self.raw, None) }
    }
}
