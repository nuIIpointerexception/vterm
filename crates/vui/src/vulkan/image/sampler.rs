use std::sync::Arc;

use ash::vk;

use crate::{errors::ImageError, vulkan::render_device::RenderDevice};

pub struct Sampler {
    pub raw: vk::Sampler,
    pub vk_dev: Arc<RenderDevice>,
}

impl Sampler {
    pub fn linear(vk_dev: Arc<RenderDevice>) -> Result<Self, ImageError> {
        let sampler_create_info = vk::SamplerCreateInfo {
            flags: vk::SamplerCreateFlags::empty(),
            mag_filter: vk::Filter::LINEAR,
            min_filter: vk::Filter::LINEAR,
            mipmap_mode: vk::SamplerMipmapMode::LINEAR,
            mip_lod_bias: 0.0,
            address_mode_u: vk::SamplerAddressMode::CLAMP_TO_EDGE,
            address_mode_v: vk::SamplerAddressMode::CLAMP_TO_EDGE,
            address_mode_w: vk::SamplerAddressMode::CLAMP_TO_EDGE,
            ..Default::default()
        };
        Sampler::new(vk_dev, sampler_create_info)
    }

    pub fn new(
        vk_dev: Arc<RenderDevice>,
        sampler_create_info: vk::SamplerCreateInfo,
    ) -> Result<Self, ImageError> {
        let raw = unsafe {
            vk_dev
                .logical_device
                .create_sampler(&sampler_create_info, None)
                .map_err(ImageError::UnableToCreateSampler)?
        };
        Ok(Self { raw, vk_dev })
    }
}

impl Drop for Sampler {
    fn drop(&mut self) {
        unsafe {
            self.vk_dev.logical_device.destroy_sampler(self.raw, None);
        }
    }
}
