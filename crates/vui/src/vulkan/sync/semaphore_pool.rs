use ::std::sync::Arc;

use crate::{
    errors::SemaphoreError,
    vulkan::{render_device::RenderDevice, sync::Semaphore},
};

pub struct SemaphorePool {
    recycled_semaphores: Vec<Semaphore>,
    pub vk_dev: Arc<RenderDevice>,
}

impl SemaphorePool {
    pub fn new(vk_dev: Arc<RenderDevice>) -> Self {
        Self { recycled_semaphores: vec![], vk_dev }
    }

    pub fn get_semaphore(&mut self) -> Result<Semaphore, SemaphoreError> {
        if let Some(recycled) = self.recycled_semaphores.pop() {
            Ok(recycled)
        } else {
            Semaphore::new(self.vk_dev.clone())
        }
    }

    pub fn return_semaphore(&mut self, semaphore: Semaphore) {
        self.recycled_semaphores.push(semaphore);
    }
}
