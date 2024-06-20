use std::sync::Arc;

use ::ash::vk;

use crate::{
    errors::{CommandBufferError, CommandResult},
    vulkan::{command_buffer::CommandPool, render_device::RenderDevice},
};

#[derive(Clone)]
pub struct CommandBuffer {
    pub raw: vk::CommandBuffer,

    pub pool: Arc<CommandPool>,

    pub vk_dev: Arc<RenderDevice>,
}

impl CommandBuffer {
    pub fn new(
        pool: Arc<CommandPool>,
        command_level: vk::CommandBufferLevel,
    ) -> Result<Self, CommandBufferError> {
        let raw = unsafe { pool.allocate_command_buffer(command_level)? };
        Ok(Self { raw, vk_dev: pool.vk_dev.clone(), pool })
    }

    pub fn new_primary(pool: Arc<CommandPool>) -> Result<Self, CommandBufferError> {
        Self::new(pool, vk::CommandBufferLevel::PRIMARY)
    }

    pub unsafe fn begin_one_time_submit(&self) -> CommandResult<&Self> {
        let begin_info = vk::CommandBufferBeginInfo {
            flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
            ..Default::default()
        };
        self.vk_dev
            .logical_device
            .begin_command_buffer(self.raw, &begin_info)
            .map_err(CommandBufferError::UnableToBeginCommandBuffer)?;
        Ok(self)
    }

    pub unsafe fn end_commands(&self) -> CommandResult<()> {
        self.vk_dev
            .logical_device
            .end_command_buffer(self.raw)
            .map_err(CommandBufferError::UnableToEndCommandBuffer)?;
        Ok(())
    }

    pub unsafe fn end_renderpass(&self) -> &Self {
        self.vk_dev.logical_device.cmd_end_render_pass(self.raw);
        self
    }
}

impl Drop for CommandBuffer {
    fn drop(&mut self) {
        unsafe {
            self.pool.free_command_buffer(self.raw);
        }
    }
}
