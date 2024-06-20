use ::ash::vk;

use crate::vulkan::allocator::{Allocation, AllocatorError};

pub trait ComposableAllocator: Send + Sync {
    /// # Safety
    ///
    /// The `allocate` function is unsafe because it directly interacts with
    /// Vulkan's memory allocation.
    /// - `allocate_info` must be a valid `vk::MemoryAllocateInfo` structure.
    /// - `alignment` must be properly aligned as required by Vulkan.
    /// - Proper synchronization must be ensured when accessing the allocator to avoid race
    ///   conditions.
    unsafe fn allocate(
        &mut self,
        allocate_info: vk::MemoryAllocateInfo,
        alignment: u64,
    ) -> Result<Allocation, AllocatorError>;

    /// # Safety
    ///
    /// The `free` function is unsafe because it directly interacts with
    /// Vulkan's memory deallocation.
    /// - `allocation` must be a valid `Allocation` that was previously allocated by this allocator.
    /// - Proper synchronization must be ensured when accessing the allocator to avoid race
    ///   conditions.
    unsafe fn free(&mut self, allocation: &Allocation) -> Result<(), AllocatorError>;
}

impl ComposableAllocator for Box<dyn ComposableAllocator> {
    unsafe fn allocate(
        &mut self,
        allocate_info: vk::MemoryAllocateInfo,
        size_in_bytes: u64,
    ) -> Result<Allocation, AllocatorError> {
        self.as_mut().allocate(allocate_info, size_in_bytes)
    }

    unsafe fn free(&mut self, allocation: &Allocation) -> Result<(), AllocatorError> {
        self.as_mut().free(allocation)
    }
}
