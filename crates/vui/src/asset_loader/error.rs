use std::io;

use ::image::ImageError;
use thiserror::Error;

use crate::errors::VulkanError;

#[derive(Debug, Error)]
pub enum AssetLoaderError {
    #[error("An unexpected Vulkan error occured!")]
    VulkanErrorWhileLoadingAssets(#[from] VulkanError),

    #[error("Unable to open the texture file")]
    UnableToOpenFile(#[from] io::Error),

    #[error("Unable to decode the texture file into rgba.")]
    UnableToDecodeImage(#[from] ImageError),

    #[error("Image not found")]
    ImageNotFound,
}
