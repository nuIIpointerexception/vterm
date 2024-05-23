use ash::vk;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InstanceError {
    #[error("Unable to setup the Vulkan debug callback")]
    DebugMessengerCreateFailed(#[source] vk::Result),

    #[error("Unable to list the available Vulkan extensions on this platform")]
    UnableToListAvailableExtensions(#[source] vk::Result),

    #[error("Required extensions are not available on this platform: {:?}", .0)]
    RequiredExtensionsNotFound(Vec<String>),

    #[error("Unable to list the available Vulkan layers on this platform")]
    UnableToListAvailableLayers(#[source] vk::Result),

    #[error("Required layers are not available on this platform: {:?}", .0)]
    RequiredLayersNotFound(Vec<String>),

    #[error("Error while creating the Vulkan function loader")]
    VulkanLoadingError(#[source] ash::LoadingError),

    #[error("Error while creating the Vulkan function loader")]
    InvalidDebugLayerName(#[source] std::str::Utf8Error),

    #[error("Unable to create the Vulkan instance")]
    UnableToCreateInstance(#[source] vk::Result),

    #[error("Unable to create the logical device")]
    UnableToCreateLogicalDevice(#[source] vk::Result),

    #[error("Error while waiting for the Vulkan device to idle")]
    UnableToWaitIdle(#[source] vk::Result),
}
