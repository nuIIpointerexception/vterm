use ::std::sync::Arc;

use crate::{
    errors::VulkanError,
    vulkan::{
        command_buffer::{CommandBuffer, CommandPool},
        render_device::RenderDevice,
        sync::{Fence, Semaphore},
    },
};

pub struct PerFrame {
    pub acquire_semaphore: Option<Semaphore>,

    pub release_semaphore: Semaphore,

    pub queue_submit_fence: Fence,

    pub command_buffer: CommandBuffer,

    pub command_pool: Arc<CommandPool>,
}

impl PerFrame {
    pub fn new(vk_dev: Arc<RenderDevice>) -> Result<Self, VulkanError> {
        let acquire_semaphore = None;
        let release_semaphore = Semaphore::new(vk_dev.clone())?;
        let queue_submit_fence = Fence::new(vk_dev.clone())?;

        let command_pool = Arc::new(CommandPool::new_transient_graphics_pool(vk_dev.clone())?);
        let command_buffer = CommandBuffer::new_primary(command_pool.clone())?;

        Ok(Self {
            acquire_semaphore,
            release_semaphore,
            queue_submit_fence,
            command_pool,
            command_buffer,
        })
    }
}
