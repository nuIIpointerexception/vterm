use std::sync::Arc;

use ::ash::vk;

use crate::vulkan::{
    allocator::{Allocation, AllocatorError, ComposableAllocator},
    render_device::RenderDevice,
};

pub struct PassthroughAllocator {
    vk_dev: Arc<RenderDevice>,
}

impl PassthroughAllocator {
    pub fn new(vk_dev: Arc<RenderDevice>) -> Self {
        Self { vk_dev }
    }
}

impl ComposableAllocator for PassthroughAllocator {
    unsafe fn allocate(
        &mut self,
        allocate_info: vk::MemoryAllocateInfo,
        _alignment: u64,
    ) -> Result<Allocation, AllocatorError> {
        Ok(Allocation {
            memory: self
                .vk_dev
                .logical_device
                .allocate_memory(&allocate_info, None)
                .map_err(AllocatorError::LogicalDeviceAllocationFailed)?,
            offset: 0,
            byte_size: allocate_info.allocation_size,
            memory_type_index: allocate_info.memory_type_index,
        })
    }

    unsafe fn free(&mut self, allocation: &Allocation) -> Result<(), AllocatorError> {
        self.vk_dev.logical_device.free_memory(allocation.memory, None);
        Ok(())
    }
}
