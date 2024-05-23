use ash::vk;
use thiserror::Error;

use crate::errors::{InstanceError, RenderDeviceError, SwapchainError};

#[derive(Error, Debug)]
pub enum WindowError {
    #[error("Failed to create the GLFW window")]
    UnableToInitGLFW(#[from] glfw::InitError),

    #[error("Vulkan is not supported on this device")]
    VulkanNotSupported,

    #[error("The GLFW Window could not be created")]
    WindowCreateFailed,

    #[error("The Window's event receiver has already been taken")]
    EventReceiverLost,

    #[error("There is no primary monitor available to this GLFW instance")]
    NoPrimaryMonitor,

    #[error("There is no video mode associated with the primary monitor")]
    PrimaryVideoModeMissing,

    #[error("GLFW is unable to determine the required vulkan extensions for this platform")]
    RequiredExtensionsUnavailable,

    #[error("Unexpected instance error")]
    UnexpectedInstanceError(#[from] InstanceError),

    #[error("Unable to create the Vulkan surface")]
    UnableToCreateSurface(#[source] vk::Result),

    #[error("Unable to create the Vulkan render device")]
    UnexpectedRenderDeviceError(#[from] RenderDeviceError),

    #[error("Unexpected swapchain error")]
    UnexpectedSwapchainError(#[from] SwapchainError),
}

#[derive(Debug, Error)]
pub enum WindowSurfaceError {
    #[error(
        "Unable to determine if the device can present images with this queue"
    )]
    UnableToCheckPhysicalDeviceSurfaceSupport(#[source] vk::Result),

    #[error("Unable to get the surface capabilities for a physical device")]
    UnableToGetPhysicalDeviceSurfaceCapabilities(#[source] vk::Result),
}
