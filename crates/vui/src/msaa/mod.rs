use std::sync::Arc;

use ::ash::vk;
pub use error::MSAAError;

use crate::{
    errors::FramebufferError,
    vulkan::{
        allocator::MemoryAllocator, command_buffer::CommandBuffer, framebuffer::Framebuffer,
        image::view::ImageView, render_device::RenderDevice, render_pass::RenderPass,
    },
};

mod depth_target;
mod error;
mod pass;
mod render_target;

pub struct MSAARenderPass {
    pub render_pass: Arc<RenderPass>,

    pub msaa_render_target: Arc<ImageView>,

    pub depth_stencil_target: Arc<ImageView>,

    pub vk_dev: Arc<RenderDevice>,
}

impl MSAARenderPass {
    pub fn for_current_swapchain(
        vk_dev: Arc<RenderDevice>,
        vk_alloc: Arc<dyn MemoryAllocator>,
    ) -> Result<Self, MSAAError> {
        let msaa_render_target =
            MSAARenderPass::create_msaa_render_target(vk_dev.clone(), vk_alloc.clone())?;
        let depth_stencil_target = MSAARenderPass::create_depth_target(
            &msaa_render_target,
            vk_dev.clone(),
            vk_alloc.clone(),
        )?;
        let render_pass = MSAARenderPass::create_render_pass(
            &msaa_render_target,
            &depth_stencil_target,
            vk_dev.clone(),
        )?;
        Ok(Self { render_pass, msaa_render_target, depth_stencil_target, vk_dev })
    }

    pub fn create_swapchain_framebuffers(&self) -> Result<Vec<Framebuffer>, FramebufferError> {
        self.vk_dev.with_swapchain(|swapchain| -> Result<Vec<Framebuffer>, FramebufferError> {
            let mut framebuffers = vec![];
            for i in 0 .. swapchain.image_views.len() {
                let views = vec![
                    self.msaa_render_target.raw,
                    self.depth_stencil_target.raw,
                    swapchain.image_views[i],
                ];
                let framebuffer = Framebuffer::with_attachments(
                    self.vk_dev.clone(),
                    &self.render_pass,
                    &views,
                    swapchain.extent,
                )?;
                framebuffers.push(framebuffer);
            }
            Ok(framebuffers)
        })
    }

    /// # Safety
    ///
    /// The `begin_renderpass_inline` function is unsafe because it issues raw
    /// Vulkan commands. Ensure:
    ///
    /// 1. `command_buffer` is valid, initialized, and in the recording state.
    /// 2. `framebuffer` is valid and compatible with this `render_pass`.
    /// 3. Synchronization is handled to avoid race conditions.
    /// 4. `rgba_clear_color` is a valid 4-element RGBA array and `clear_depth` is a valid depth
    ///    value.
    /// 5. `render_area` in `render_pass_begin_info` is within framebuffer bounds.
    /// 6. `self.vk_dev` matches the device used for `command_buffer` and `framebuffer`.
    ///
    /// Improper use can cause undefined behavior or application crashes.

    pub unsafe fn begin_renderpass_inline(
        &self,
        command_buffer: &CommandBuffer,
        framebuffer: &Framebuffer,
        rgba_clear_color: [f32; 4],
        clear_depth: f32,
    ) {
        let clear_values = [
            vk::ClearValue { color: vk::ClearColorValue { float32: rgba_clear_color } },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue { depth: clear_depth, stencil: 0 },
            },
        ];
        let render_pass_begin_info = vk::RenderPassBeginInfo {
            render_pass: framebuffer.render_pass.raw,
            framebuffer: framebuffer.raw,
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: framebuffer.extent,
            },
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr(),
            ..Default::default()
        };
        self.vk_dev.logical_device.cmd_begin_render_pass(
            command_buffer.raw,
            &render_pass_begin_info,
            vk::SubpassContents::INLINE,
        );
    }

    /// # Safety
    ///
    /// The `end_renderpass` function is unsafe because it issues raw Vulkan
    /// commands. Ensure:
    ///
    /// 1. `command_buffer` is valid and in a recording state with an active render pass.
    /// 2. Proper synchronization to avoid race conditions.
    ///
    /// Misuse can lead to undefined behavior or application crashes.
    pub unsafe fn end_renderpass(&self, command_buffer: &CommandBuffer) {
        self.vk_dev.logical_device.cmd_end_render_pass(command_buffer.raw);
    }

    pub fn samples(&self) -> vk::SampleCountFlags {
        self.msaa_render_target.image.create_info.samples
    }
}
