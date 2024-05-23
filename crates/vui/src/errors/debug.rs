use ::thiserror::Error;

use crate::errors::RenderDeviceError;

#[derive(Debug, Error)]
pub enum VulkanDebugError {
    #[error(transparent)]
    UnexpectedRenderDeviceError(#[from] RenderDeviceError),

    #[error(transparent)]
    UnknownRuntimeError(#[from] anyhow::Error),
}
