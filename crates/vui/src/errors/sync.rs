use ash::vk;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FenceError {
    #[error("Unable to create a new fence")]
    UnableToCreateFence(#[source] vk::Result),

    #[error("Error while waiting for fence")]
    UnexpectedWaitError(#[source] vk::Result),

    #[error("Error while resetting fence")]
    UnexpectedResetError(#[source] vk::Result),
}

#[derive(Debug, Error)]
pub enum SemaphoreError {
    #[error("Unable to create a new semaphore")]
    UnableToCreateSemaphore(#[source] vk::Result),
}
