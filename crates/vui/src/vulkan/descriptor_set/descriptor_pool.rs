use std::{ffi::c_void, sync::Arc};

use ash::vk;

use crate::{
    errors::DescriptorSetError,
    vulkan::{
        descriptor_set::{DescriptorSet, DescriptorSetLayout},
        render_device::RenderDevice,
    },
};

pub struct DescriptorPool {
    pub raw: vk::DescriptorPool,

    pub vk_dev: Arc<RenderDevice>,
}

impl DescriptorPool {
    pub fn new(
        vk_dev: Arc<RenderDevice>,
        descriptor_count: u32,
        sizes: &[vk::DescriptorPoolSize],
    ) -> Result<Self, DescriptorSetError> {
        let create_info = vk::DescriptorPoolCreateInfo {
            flags: vk::DescriptorPoolCreateFlags::empty(),
            max_sets: descriptor_count,
            pool_size_count: sizes.len() as u32,
            p_pool_sizes: sizes.as_ptr(),
            ..Default::default()
        };
        let raw = unsafe {
            vk_dev
                .logical_device
                .create_descriptor_pool(&create_info, None)
                .map_err(DescriptorSetError::UnableToCreatePool)?
        };
        Ok(Self { raw, vk_dev })
    }

    pub fn allocate(
        &self,
        layout: &DescriptorSetLayout,
        count: u32,
    ) -> Result<Vec<DescriptorSet>, DescriptorSetError> {
        let mut layouts = vec![];
        for _ in 0 .. count {
            layouts.push(layout.raw);
        }
        let allocate_info = vk::DescriptorSetAllocateInfo {
            descriptor_pool: self.raw,
            descriptor_set_count: layouts.len() as u32,
            p_set_layouts: layouts.as_ptr(),
            ..Default::default()
        };
        let raw_sets = unsafe {
            self.vk_dev
                .logical_device
                .allocate_descriptor_sets(&allocate_info)
                .map_err(DescriptorSetError::UnableToAllocateDescriptors)?
        };
        let descriptor_sets: Vec<DescriptorSet> = raw_sets
            .into_iter()
            .map(|raw| DescriptorSet { raw, vk_dev: self.vk_dev.clone() })
            .collect();
        Ok(descriptor_sets)
    }

    pub fn allocate_with_variable_counts(
        &self,
        layout: &DescriptorSetLayout,
        descriptor_set_count: u32,
        variable_binding_count: u32,
    ) -> Result<Vec<DescriptorSet>, DescriptorSetError> {
        let mut descriptor_set_counts = vec![];
        let mut layouts = vec![];
        for _ in 0 .. descriptor_set_count {
            layouts.push(layout.raw);
            descriptor_set_counts.push(variable_binding_count);
        }
        let variable_descriptor_alloc_info = vk::DescriptorSetVariableDescriptorCountAllocateInfo {
            descriptor_set_count: layouts.len() as u32,
            p_descriptor_counts: descriptor_set_counts.as_ptr(),
            ..Default::default()
        };
        let allocate_info = vk::DescriptorSetAllocateInfo {
            p_next: &variable_descriptor_alloc_info
                as *const vk::DescriptorSetVariableDescriptorCountAllocateInfo
                as *const c_void,
            descriptor_pool: self.raw,
            descriptor_set_count: layouts.len() as u32,
            p_set_layouts: layouts.as_ptr(),
            ..Default::default()
        };
        let raw_sets = unsafe {
            self.vk_dev
                .logical_device
                .allocate_descriptor_sets(&allocate_info)
                .map_err(DescriptorSetError::UnableToAllocateDescriptors)?
        };
        let descriptor_sets: Vec<DescriptorSet> = raw_sets
            .into_iter()
            .map(|raw| DescriptorSet { raw, vk_dev: self.vk_dev.clone() })
            .collect();
        Ok(descriptor_sets)
    }
}

impl Drop for DescriptorPool {
    fn drop(&mut self) {
        unsafe {
            self.vk_dev.logical_device.destroy_descriptor_pool(self.raw, None);
        }
    }
}
