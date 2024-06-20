use std::sync::Arc;

use ::ash::vk;

use crate::{
    errors::CommandBufferError,
    vulkan::{
        command_buffer::{CommandBuffer, CommandPool},
        render_device::{GpuQueue, RenderDevice},
    },
};

#[derive(Clone)]
pub struct OneTimeSubmitCommandPool {
    pool: Arc<CommandPool>,
    cmd: CommandBuffer,
    queue: GpuQueue,

    pub vk_dev: Arc<RenderDevice>,
}

impl OneTimeSubmitCommandPool {
    pub fn new(vk_dev: Arc<RenderDevice>, queue: &GpuQueue) -> Result<Self, CommandBufferError> {
        let pool = Arc::new(CommandPool::new(
            vk_dev.clone(),
            queue,
            vk::CommandPoolCreateFlags::TRANSIENT,
        )?);
        let cmd = CommandBuffer::new_primary(pool.clone())?;
        Ok(Self { pool, cmd, queue: *queue, vk_dev })
    }

    pub fn submit_sync_commands<Func, T>(&self, func: Func) -> Result<T, CommandBufferError>
    where
        Func: FnOnce(&Arc<RenderDevice>, vk::CommandBuffer) -> T,
    {
        self.pool.reset()?;
        unsafe {
            let begin_info = vk::CommandBufferBeginInfo {
                flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
                ..Default::default()
            };
            self.vk_dev
                .logical_device
                .begin_command_buffer(self.cmd.raw, &begin_info)
                .map_err(CommandBufferError::UnableToBeginCommandBuffer)?;

            let result: T = func(&self.vk_dev, self.cmd.raw);

            self.vk_dev
                .logical_device
                .end_command_buffer(self.cmd.raw)
                .map_err(CommandBufferError::UnableToEndCommandBuffer)?;

            let submit_info = vk::SubmitInfo {
                command_buffer_count: 1,
                p_command_buffers: &self.cmd.raw,
                ..Default::default()
            };
            self.vk_dev
                .logical_device
                .queue_submit(self.queue.queue, &[submit_info], vk::Fence::null())
                .map_err(CommandBufferError::UnableToSubmitCommandBuffer)?;
            self.vk_dev
                .logical_device
                .device_wait_idle()
                .map_err(CommandBufferError::UnableToWaitForDeviceIdle)?;

            Ok(result)
        }
    }
}
