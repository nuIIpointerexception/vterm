use thiserror::Error;

use crate::errors::{
    AllocatorError, BufferError, CommandBufferError, DescriptorSetError, FenceError,
    FramebufferError, ImageError, InstanceError, PhysicalDeviceError, PipelineError,
    QueueSelectionError, RenderDeviceError, RenderPassError, SemaphoreError, SwapchainError,
    VulkanDebugError, WindowSurfaceError,
};

#[derive(Debug, Error)]
pub enum VulkanError {
    #[error(transparent)]
    InstanceError(#[from] InstanceError),

    #[error(transparent)]
    PhysicalDeviceError(#[from] PhysicalDeviceError),

    #[error(transparent)]
    QueueSelectionError(#[from] QueueSelectionError),

    #[error(transparent)]
    RenderDeviceError(#[from] RenderDeviceError),

    #[error(transparent)]
    SwapchainError(#[from] SwapchainError),

    #[error(transparent)]
    SemaphorePoolError(#[from] SemaphoreError),

    #[error(transparent)]
    WindowSurfaceError(#[from] WindowSurfaceError),

    #[error(transparent)]
    AllocatorError(#[from] AllocatorError),

    #[error(transparent)]
    BufferError(#[from] BufferError),

    #[error(transparent)]
    PipelineError(#[from] PipelineError),

    #[error(transparent)]
    VulkanDebugError(#[from] VulkanDebugError),

    #[error(transparent)]
    FramebufferError(#[from] FramebufferError),

    #[error(transparent)]
    FenceError(#[from] FenceError),

    #[error(transparent)]
    CommandBufferError(#[from] CommandBufferError),

    #[error(transparent)]
    RenderPassError(#[from] RenderPassError),

    #[error(transparent)]
    DescriptorSetError(#[from] DescriptorSetError),

    #[error(transparent)]
    ImageError(#[from] ImageError),
}
