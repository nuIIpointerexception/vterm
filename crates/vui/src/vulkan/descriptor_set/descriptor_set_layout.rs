use std::{ffi::c_void, sync::Arc};

use ::ash::vk;

use crate::{errors::DescriptorSetError, vulkan::render_device::RenderDevice};

pub struct DescriptorSetLayout {
    pub raw: vk::DescriptorSetLayout,

    pub vk_dev: Arc<RenderDevice>,
}

impl DescriptorSetLayout {
    pub fn new(
        vk_dev: Arc<RenderDevice>,
        bindings: &[vk::DescriptorSetLayoutBinding],
    ) -> Result<Self, DescriptorSetError> {
        let bindings_and_flags: Vec<_> = bindings
            .iter()
            .map(|binding| (*binding, vk::DescriptorBindingFlags::empty()))
            .collect();
        Self::new_with_flags(vk_dev, &bindings_and_flags)
    }

    pub fn new_with_flags(
        vk_dev: Arc<RenderDevice>,
        bindings_and_flags: &[(vk::DescriptorSetLayoutBinding, vk::DescriptorBindingFlags)],
    ) -> Result<Self, DescriptorSetError> {
        let flags: Vec<vk::DescriptorBindingFlags> =
            bindings_and_flags.iter().map(|(_binding, flag)| *flag).collect();
        let bindings: Vec<vk::DescriptorSetLayoutBinding> =
            bindings_and_flags.iter().map(|(binding, _flag)| *binding).collect();
        let binding_flags_create_info = vk::DescriptorSetLayoutBindingFlagsCreateInfo {
            binding_count: flags.len() as u32,
            p_binding_flags: flags.as_ptr(),
            ..Default::default()
        };
        let create_info = vk::DescriptorSetLayoutCreateInfo {
            p_next: &binding_flags_create_info
                as *const vk::DescriptorSetLayoutBindingFlagsCreateInfo
                as *const c_void,
            flags: vk::DescriptorSetLayoutCreateFlags::empty(),
            p_bindings: bindings.as_ptr(),
            binding_count: bindings.len() as u32,
            ..Default::default()
        };
        let raw = unsafe {
            vk_dev
                .logical_device
                .create_descriptor_set_layout(&create_info, None)
                .map_err(DescriptorSetError::UnableToCreateLayout)?
        };
        Ok(Self { raw, vk_dev })
    }
}

impl Drop for DescriptorSetLayout {
    fn drop(&mut self) {
        unsafe {
            self.vk_dev.logical_device.destroy_descriptor_set_layout(self.raw, None);
        }
    }
}
