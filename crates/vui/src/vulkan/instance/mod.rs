use ash::{Entry, khr::swapchain, vk};

use crate::errors::InstanceError;

pub struct Instance {
    pub ash: ash::Instance,
    pub entry: Entry,
}

impl Instance {
    pub fn new(
        instance: ash::Instance,
        entry: &Entry,
    ) -> Result<Self, InstanceError> {
        Ok(Self {
            ash: instance,
            entry: entry.clone(),
        })
    }

    pub fn create_logical_device(
        &self,
        physical_device: &vk::PhysicalDevice,
        queue_create_infos: &[vk::DeviceQueueCreateInfo],
    ) -> Result<ash::Device, InstanceError> {
        let extensions = vec![swapchain::NAME.as_ptr()];
        let features =
            vk::PhysicalDeviceFeatures::default().fill_mode_non_solid(true);
        let mut vk_11_features = vk::PhysicalDeviceVulkan11Features::default()
            .uniform_and_storage_buffer16_bit_access(true);

        let mut vk_12_features = vk::PhysicalDeviceVulkan12Features::default()
            .shader_sampled_image_array_non_uniform_indexing(true)
            .runtime_descriptor_array(true)
            .descriptor_binding_partially_bound(true)
            .descriptor_binding_variable_descriptor_count(true);
        let create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(queue_create_infos)
            .enabled_extension_names(&extensions)
            .enabled_features(&features)
            .push_next(&mut vk_11_features)
            .push_next(&mut vk_12_features);

        unsafe {
            self.ash
                .create_device(*physical_device, &create_info, None)
                .map_err(InstanceError::UnableToCreateLogicalDevice)
        }
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            self.ash.destroy_instance(None);
        }
    }
}
