use ::ash::vk;
use thiserror::Error;

use crate::errors::AllocatorError;

#[derive(Debug, Error)]
pub enum BufferError {
    #[error("Unable to map device memory")]
    UnableToMapDeviceMemory(#[source] vk::Result),

    #[error(
        "Device memory pointer was not found, did you try calling .map()?"
    )]
    NoMappedPointerFound,

    #[error(
        "Unable to create a new device buffer for {} bytes with flags {:?}",
        .size,
        .usage
    )]
    UnableToCreateBuffer {
        size: u64,
        usage: vk::BufferUsageFlags,
        source: vk::Result,
    },

    #[error(transparent)]
    UnableToAllocateBufferMemory(#[from] AllocatorError),

    #[error("Unable to bind device memory to buffer")]
    UnableToBindDeviceMemory(#[source] vk::Result),
}
