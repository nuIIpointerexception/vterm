use ::std::sync::Arc;

use ash::vk;

use crate::{errors::RenderPassError, vulkan::render_device::RenderDevice};

pub struct RenderPass {
    pub raw: vk::RenderPass,
    pub vk_dev: Arc<RenderDevice>,
}

impl RenderPass {
    pub fn new(
        vk_dev: Arc<RenderDevice>,
        create_info: &vk::RenderPassCreateInfo,
    ) -> Result<Self, RenderPassError> {
        let raw = unsafe {
            vk_dev
                .logical_device
                .create_render_pass(create_info, None)
                .map_err(RenderPassError::UnableToCreateRenderPass)?
        };
        Ok(Self { raw, vk_dev })
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        unsafe {
            self.vk_dev
                .logical_device
                .destroy_render_pass(self.raw, None);
        }
    }
}
