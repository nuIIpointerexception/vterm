use std::sync::Arc;

use ::ash::vk;

use crate::{
    errors::CommandBufferError,
    vulkan::{command_buffer::CommandPool, render_device::RenderDevice},
};

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
        Ok(Self {
            raw,
            vk_dev: pool.vk_dev.clone(),
            pool,
        })
    }

    pub fn new_primary(
        pool: Arc<CommandPool>,
    ) -> Result<Self, CommandBufferError> {
        Self::new(pool, vk::CommandBufferLevel::PRIMARY)
    }
}

impl Drop for CommandBuffer {
    fn drop(&mut self) {
        unsafe {
            self.pool.free_command_buffer(self.raw);
        }
    }
}
