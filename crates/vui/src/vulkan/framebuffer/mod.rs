use std::sync::Arc;

use ::ash::vk;

use crate::{
    errors::FramebufferError,
    vulkan::{render_device::RenderDevice, render_pass::RenderPass},
};

pub struct Framebuffer {
    pub raw: vk::Framebuffer,

    pub render_pass: Arc<RenderPass>,

    pub extent: vk::Extent2D,

    pub vk_dev: Arc<RenderDevice>,
}

impl Framebuffer {
    pub fn with_swapchain_color_attachments(
        vk_dev: Arc<RenderDevice>,
        render_pass: &Arc<RenderPass>,
    ) -> Result<Vec<Self>, FramebufferError> {
        vk_dev.with_swapchain(|swapchain| -> Result<Vec<Self>, FramebufferError> {
            let mut framebuffers = vec![];
            for i in 0 .. swapchain.image_views.len() {
                let framebuffer = Self::with_attachments(
                    vk_dev.clone(),
                    render_pass,
                    &[swapchain.image_views[i]],
                    swapchain.extent,
                )?;
                framebuffers.push(framebuffer);
            }
            Ok(framebuffers)
        })
    }

    pub fn with_attachments(
        vk_dev: Arc<RenderDevice>,
        render_pass: &Arc<RenderPass>,
        images: &[vk::ImageView],
        extent: vk::Extent2D,
    ) -> Result<Self, FramebufferError> {
        let create_info = vk::FramebufferCreateInfo {
            flags: vk::FramebufferCreateFlags::empty(),
            render_pass: render_pass.raw,
            attachment_count: images.len() as u32,
            p_attachments: images.as_ptr(),
            width: extent.width,
            height: extent.height,
            layers: 1,
            ..Default::default()
        };
        let raw = unsafe {
            vk_dev
                .logical_device
                .create_framebuffer(&create_info, None)
                .map_err(FramebufferError::UnableToCreateFramebuffer)?
        };
        Ok(Self { raw, extent, render_pass: render_pass.clone(), vk_dev })
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            self.vk_dev.logical_device.destroy_framebuffer(self.raw, None);
        }
    }
}
