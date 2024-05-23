use ::ash::vk;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommandBufferError {
    #[error("Unable to create a new command buffer pool")]
    UnableToCreateCommandPool(#[source] vk::Result),

    #[error("Unable to allocate a command buffer from the command pool")]
    UnableToAllocateBuffer(#[source] vk::Result),

    #[error("Unable to reset the command pool")]
    UnableToResetPool(#[source] vk::Result),

    #[error("Unable to begin the command buffer")]
    UnableToBeginCommandBuffer(#[source] vk::Result),

    #[error("Unable to end the command buffer")]
    UnableToEndCommandBuffer(#[source] vk::Result),

    #[error("Unable to submit the command buffer for execution")]
    UnableToSubmitCommandBuffer(#[source] vk::Result),

    #[error("Error while waiting for the device to idle.")]
    UnableToWaitForDeviceIdle(#[source] vk::Result),
}

pub type CommandResult<T> = Result<T, CommandBufferError>;
