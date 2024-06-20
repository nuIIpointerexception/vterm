use ::ash::vk;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("The shader's source bytes must be evenly divisible into u32 words")]
    InvalidSourceLengthInShaderSPIRV,

    #[error("Improper bytes found in compiled SPIRV shader module source")]
    InvalidBytesInShaderSPIRV(#[source] core::array::TryFromSliceError),

    #[error("Unable to create the shader module")]
    UnableToCreateShaderModule(#[source] vk::Result),

    #[error("Unable to create the pipeline layout")]
    UnableToCreatePipelineLayout(#[source] vk::Result),

    #[error("Unable to create graphics pipeline")]
    UnableToCreateGraphicsPipeline(#[source] vk::Result),
}
