use ::ash::vk;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AllocatorError {
    #[error("failed to allocate memory using the Vulkan device")]
    LogicalDeviceAllocationFailed(#[source] vk::Result),

    #[error("no memory type could be found for flags {:?} and requirements {:?}", .0, .1)]
    MemoryTypeNotFound(vk::MemoryPropertyFlags, vk::MemoryRequirements),
}
