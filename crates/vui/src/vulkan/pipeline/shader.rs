use std::sync::Arc;

use ash::vk;

use crate::{errors::PipelineError, vulkan::render_device::RenderDevice};

const DEFAULT_ENTRY_POINT: &'static [u8] = b"main\0";

pub struct ShaderModule {
    pub raw: vk::ShaderModule,
    pub vk_dev: Arc<RenderDevice>,
}

impl ShaderModule {
    pub fn from_spirv(
        vk_dev: Arc<RenderDevice>,
        source: &'static [u8],
    ) -> Result<Self, PipelineError> {
        let source_u32 = Self::copy_to_u32(source)?;
        let create_info = vk::ShaderModuleCreateInfo {
            p_code: source_u32.as_ptr(),
            code_size: source_u32.len() * std::mem::size_of::<u32>(),
            ..Default::default()
        };
        let shader_module = unsafe {
            vk_dev
                .logical_device
                .create_shader_module(&create_info, None)
                .map_err(PipelineError::UnableToCreateShaderModule)?
        };
        Ok(Self {
            raw: shader_module,
            vk_dev,
        })
    }

    pub fn stage_create_info(
        &self,
        stage: vk::ShaderStageFlags,
    ) -> vk::PipelineShaderStageCreateInfo {
        vk::PipelineShaderStageCreateInfo {
            stage,
            module: self.raw,
            p_name: DEFAULT_ENTRY_POINT.as_ptr() as *const i8,
            ..Default::default()
        }
    }
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        unsafe {
            self.vk_dev
                .logical_device
                .destroy_shader_module(self.raw, None);
        }
    }
}

impl ShaderModule {
    fn copy_to_u32(bytes: &'static [u8]) -> Result<Vec<u32>, PipelineError> {
        use std::convert::TryInto;

        const U32_SIZE: usize = std::mem::size_of::<u32>();
        if bytes.len() % U32_SIZE != 0 {
            return Err(PipelineError::InvalidSourceLengthInShaderSPIRV);
        }

        let mut buffer: Vec<u32> = vec![];
        let mut input: &[u8] = &bytes;
        while input.len() > 0 {
            let (int_slice, rest) = input.split_at(U32_SIZE);
            input = rest;
            let word = u32::from_le_bytes(
                int_slice
                    .try_into()
                    .map_err(PipelineError::InvalidBytesInShaderSPIRV)?,
            );
            buffer.push(word);
        }

        Ok(buffer)
    }
}
