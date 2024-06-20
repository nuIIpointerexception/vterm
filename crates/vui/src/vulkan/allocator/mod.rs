use std::sync::Arc;

use ::ash::vk;

pub use self::{
    allocation::Allocation, composable::ComposableAllocator, locked_memory::LockedMemoryAllocator,
    passthrough::PassthroughAllocator,
};
use crate::{errors::AllocatorError, vulkan::render_device::RenderDevice};

mod allocation;
mod composable;
mod locked_memory;
mod passthrough;

pub trait MemoryAllocator: Send + Sync {
    unsafe fn allocate_memory(
        &self,
        memory_requirements: vk::MemoryRequirements,
        property_flags: vk::MemoryPropertyFlags,
    ) -> Result<Allocation, AllocatorError>;

    unsafe fn free(&self, allocation: &Allocation) -> Result<(), AllocatorError>;
}

pub fn create_default_allocator(vk_dev: Arc<RenderDevice>) -> Arc<dyn MemoryAllocator> {
    let locked_allocator =
        LockedMemoryAllocator::new(vk_dev.clone(), PassthroughAllocator::new(vk_dev.clone()));
    Arc::new(locked_allocator)
}
