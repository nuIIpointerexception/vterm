use std::sync::Arc;

use ash::vk;

use crate::{errors::SemaphoreError, vulkan::render_device::RenderDevice};

pub struct Semaphore {
    pub raw: vk::Semaphore,
    pub vk_dev: Arc<RenderDevice>,
}

impl Semaphore {
    pub fn new(vk_dev: Arc<RenderDevice>) -> Result<Self, SemaphoreError> {
        let create_info = vk::SemaphoreCreateInfo {
            ..Default::default()
        };
        let raw = unsafe {
            vk_dev
                .logical_device
                .create_semaphore(&create_info, None)
                .map_err(SemaphoreError::UnableToCreateSemaphore)?
        };
        Ok(Self { raw, vk_dev })
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        unsafe {
            self.vk_dev.logical_device.destroy_semaphore(self.raw, None);
        }
    }
}
