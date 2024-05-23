use ::thiserror::Error;

use crate::errors::VulkanError;

#[derive(Debug, Error)]
pub enum MSAAError {
    #[error("Unable to pick a supported depth format")]
    UnableToPickDepthFormat,

    #[error(transparent)]
    UnexpectedVulkanError(#[from] VulkanError),
}
