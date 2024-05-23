use ::ash::vk;
use thiserror::Error;

use crate::errors::VulkanDebugError;

#[derive(Debug, Error)]
pub enum FramebufferError {
    #[error("Unable to create the framebuffer")]
    UnableToCreateFramebuffer(#[source] vk::Result),

    #[error("Unable to create a framebuffer for swapchain image {}", .0)]
    UnableToCreateSwapchainFramebuffer(usize, #[source] vk::Result),

    #[error(transparent)]
    UnexpectedVulkanDebugError(#[from] VulkanDebugError),
}
