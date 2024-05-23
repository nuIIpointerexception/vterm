use ::thiserror::Error;
use ash::vk;

use crate::errors::VulkanError;

#[derive(Debug, Error)]
pub enum FrameError {
    #[error("The swapchain needs to be rebuilt")]
    SwapchainNeedsRebuild,

    #[error(transparent)]
    VkError(vk::Result),

    #[error(transparent)]
    UnexpectedRuntimeError(#[from] anyhow::Error),

    #[error(transparent)]
    UnexpectedVulkanError(#[from] VulkanError),
}
