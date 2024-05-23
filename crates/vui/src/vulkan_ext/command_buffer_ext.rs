use ::ash::vk;

use crate::{
    errors::{CommandBufferError, CommandResult},
    vulkan::command_buffer::CommandBuffer,
};

pub trait CommandBufferExt {
    unsafe fn begin_one_time_submit(&self) -> CommandResult<&Self>;

    unsafe fn end_commands(&self) -> CommandResult<()>;

    unsafe fn end_renderpass(&self) -> &Self;
}

impl CommandBufferExt for CommandBuffer {
    unsafe fn begin_one_time_submit(&self) -> CommandResult<&Self> {
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

    unsafe fn end_commands(&self) -> CommandResult<()> {
        self.vk_dev
            .logical_device
            .end_command_buffer(self.raw)
            .map_err(CommandBufferError::UnableToEndCommandBuffer)?;
        Ok(())
    }

    unsafe fn end_renderpass(&self) -> &Self {
        self.vk_dev.logical_device.cmd_end_render_pass(self.raw);
        self
    }
}
