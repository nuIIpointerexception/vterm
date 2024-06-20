use std::sync::{Arc, Mutex};

use ash::vk;

use crate::vulkan::{
    allocator::{Allocation, AllocatorError, ComposableAllocator, MemoryAllocator},
    render_device::RenderDevice,
};

pub struct LockedMemoryAllocator<Alloc: ComposableAllocator> {
    composed_allocator: Mutex<Alloc>,
    vk_dev: Arc<RenderDevice>,
}

impl<Alloc: ComposableAllocator> LockedMemoryAllocator<Alloc> {
    pub fn new(vk_dev: Arc<RenderDevice>, allocator: Alloc) -> Self {
        Self { composed_allocator: Mutex::new(allocator), vk_dev }
    }
}

impl<Alloc: ComposableAllocator> MemoryAllocator for LockedMemoryAllocator<Alloc> {
    unsafe fn allocate_memory(
        &self,
        memory_requirements: vk::MemoryRequirements,
        property_flags: vk::MemoryPropertyFlags,
    ) -> Result<Allocation, AllocatorError> {
        let memory_properties = self
            .vk_dev
            .instance
            .ash
            .get_physical_device_memory_properties(self.vk_dev.physical_device);
        let memory_type_index = memory_properties
            .memory_types
            .iter()
            .enumerate()
            .find(|(i, memory_type)| {
                let i = *i as u32;
                let type_supported = (memory_requirements.memory_type_bits & (1 << i)) != 0;
                let properties_supported = memory_type.property_flags.contains(property_flags);
                type_supported && properties_supported
            })
            .map(|(i, _memory_type)| i as u32)
            .ok_or(AllocatorError::MemoryTypeNotFound(property_flags, memory_requirements))?;

        let allocate_info = vk::MemoryAllocateInfo {
            memory_type_index,
            allocation_size: memory_requirements.size,
            ..Default::default()
        };

        let mut allocator = self
            .composed_allocator
            .lock()
            .expect("unable to acquire the composed memory allocator lock");
        allocator.allocate(allocate_info, memory_requirements.alignment)
    }

    unsafe fn free(&self, allocation: &Allocation) -> Result<(), AllocatorError> {
        let mut allocator = self
            .composed_allocator
            .lock()
            .expect("unable to acquire the composed memory allocator lock");
        allocator.free(allocation)
    }
}
