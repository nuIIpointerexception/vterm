use ::ash::vk;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DescriptorSetError {
    #[error("Unable to create the descriptor set layout")]
    UnableToCreateLayout(#[source] vk::Result),

    #[error("Unable to create the descriptor pool")]
    UnableToCreatePool(#[source] vk::Result),

    #[error("Unable to allocate descriptors from the pool")]
    UnableToAllocateDescriptors(#[source] vk::Result),
}
