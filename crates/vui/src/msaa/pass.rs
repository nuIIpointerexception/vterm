use std::sync::Arc;

use ash::vk;

use crate::{
    errors::VulkanError,
    msaa::MSAARenderPass,
    vulkan::{
        image::view::ImageView, render_device::RenderDevice,
        render_pass::RenderPass,
    },
};

impl MSAARenderPass {
    pub(super) fn create_render_pass(
        msaa_render_target: &ImageView,
        depth_stencil_target: &ImageView,
        vk_dev: Arc<RenderDevice>,
    ) -> Result<Arc<RenderPass>, VulkanError> {
        fn create_description(
            format: vk::Format,
            samples: vk::SampleCountFlags,
            load_op: vk::AttachmentLoadOp,
            store_op: vk::AttachmentStoreOp,
            final_layout: vk::ImageLayout,
        ) -> vk::AttachmentDescription {
            vk::AttachmentDescription {
                flags: vk::AttachmentDescriptionFlags::empty(),
                format,
                samples,
                load_op,
                store_op,
                stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
                stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
                initial_layout: vk::ImageLayout::UNDEFINED,
                final_layout,
            }
        }

        let color_attachment = create_description(
            msaa_render_target.image.create_info.format,
            msaa_render_target.image.create_info.samples,
            vk::AttachmentLoadOp::CLEAR,
            vk::AttachmentStoreOp::STORE,
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        );
        let depth_stencil_attachment = create_description(
            depth_stencil_target.image.create_info.format,
            depth_stencil_target.image.create_info.samples,
            vk::AttachmentLoadOp::CLEAR,
            vk::AttachmentStoreOp::DONT_CARE,
            vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        );
        let color_resolve_attachment = create_description(
            msaa_render_target.image.create_info.format,
            vk::SampleCountFlags::TYPE_1,
            vk::AttachmentLoadOp::DONT_CARE,
            vk::AttachmentStoreOp::STORE,
            vk::ImageLayout::PRESENT_SRC_KHR,
        );

        let attachments = [
            color_attachment,
            depth_stencil_attachment,
            color_resolve_attachment,
        ];
        let attachment_references = [
            vk::AttachmentReference {
                attachment: 0,
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            },
            vk::AttachmentReference {
                attachment: 1,
                layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            },
            vk::AttachmentReference {
                attachment: 2,
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            },
        ];
        let subpass = vk::SubpassDescription {
            flags: vk::SubpassDescriptionFlags::empty(),
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            color_attachment_count: 1,
            p_color_attachments: &attachment_references[0],
            p_depth_stencil_attachment: &attachment_references[1],
            p_resolve_attachments: &attachment_references[2],
            ..Default::default()
        };
        let dependencies = [vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            src_access_mask: vk::AccessFlags::empty(),
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ
                | vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            dependency_flags: vk::DependencyFlags::empty(),
        }];
        let render_pass_info = vk::RenderPassCreateInfo {
            flags: vk::RenderPassCreateFlags::empty(),
            attachment_count: attachments.len() as u32,
            p_attachments: attachments.as_ptr(),
            subpass_count: 1,
            p_subpasses: &subpass,
            dependency_count: dependencies.len() as u32,
            p_dependencies: dependencies.as_ptr(),
            ..Default::default()
        };

        Ok(Arc::new(RenderPass::new(vk_dev, &render_pass_info)?))
    }
}
