use ash::{Entry, vk};

use crate::{errors::InstanceError, vulkan::ffi::to_os_ptrs};

pub struct Instance {
    pub ash: ash::Instance,

    #[allow(unused)]
    layers: Vec<String>,

    #[allow(unused)]
    pub entry: Entry,
}

impl Instance {
    pub fn new(required_extensions: &[String]) -> Result<Self, InstanceError> {
        let (instance, entry) = create_instance(required_extensions)?;
        Ok(Self {
            ash: instance,
            entry,
            layers: debug_layers(),
        })
    }

    // This function runs awfully slow on nvidia drivers.
    // It's not a problem with the code, but with the drivers.
    pub fn create_logical_device(
        &self,
        physical_device: &vk::PhysicalDevice,
        physical_device_extensions: &[String],
        queue_create_infos: &[vk::DeviceQueueCreateInfo],
    ) -> Result<ash::Device, InstanceError> {
        let (_c_names, layer_name_ptrs) = unsafe { to_os_ptrs(&self.layers) };
        let (_c_ext_names, ext_name_ptrs) =
            unsafe { to_os_ptrs(physical_device_extensions) };

        let mut indexing_features =
            vk::PhysicalDeviceDescriptorIndexingFeatures {
                shader_sampled_image_array_non_uniform_indexing: vk::TRUE,
                runtime_descriptor_array: vk::TRUE,
                descriptor_binding_variable_descriptor_count: vk::TRUE,
                ..unsafe { std::mem::zeroed() }
            };
        let physical_device_features = vk::PhysicalDeviceFeatures2 {
            p_next: &mut indexing_features as *mut _ as *mut _,
            features: vk::PhysicalDeviceFeatures {
                geometry_shader: vk::TRUE,
                ..unsafe { std::mem::zeroed() }
            },
            ..unsafe { std::mem::zeroed() }
        };

        let create_info = vk::DeviceCreateInfo {
            p_next: &physical_device_features as *const _ as *const _,
            queue_create_info_count: queue_create_infos.len() as u32,
            p_queue_create_infos: queue_create_infos.as_ptr(),
            p_enabled_features: std::ptr::null(),
            pp_enabled_layer_names: layer_name_ptrs.as_ptr(),
            enabled_layer_count: layer_name_ptrs.len() as u32,
            pp_enabled_extension_names: ext_name_ptrs.as_ptr(),
            enabled_extension_count: physical_device_extensions.len() as u32,
            ..unsafe { std::mem::zeroed() }
        };

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

fn debug_layers() -> Vec<String> {
    vec![
            // "VK_LAYER_KHRONOS_validation".to_owned(),
            // "VK_LAYER_LUNARG_monitor".to_owned(),
    ]
}

fn create_instance(
    required_extensions: &[String],
) -> Result<(ash::Instance, Entry), InstanceError> {
    use std::ffi::CString;

    let entry = Entry::linked();

    let app_name = CString::new("vterm").unwrap();
    let engine_name = CString::new("vui").unwrap();

    let app_info = vk::ApplicationInfo {
        p_engine_name: engine_name.as_ptr(),
        p_application_name: app_name.as_ptr(),
        application_version: vk::make_api_version(0, 0, 1, 0),
        api_version: vk::make_api_version(0, 1, 2, 0),
        ..Default::default()
    };

    let (_layer_names, layer_ptrs) = unsafe { to_os_ptrs(&debug_layers()) };
    let (_ext_names, ext_ptrs) = unsafe { to_os_ptrs(required_extensions) };

    let create_info = vk::InstanceCreateInfo {
        p_application_info: &app_info,
        pp_enabled_layer_names: layer_ptrs.as_ptr(),
        enabled_layer_count: layer_ptrs.len() as u32,
        pp_enabled_extension_names: ext_ptrs.as_ptr(),
        enabled_extension_count: ext_ptrs.len() as u32,
        ..Default::default()
    };

    let instance = unsafe {
        entry
            .create_instance(&create_info, None)
            .map_err(InstanceError::UnableToCreateInstance)?
    };

    Ok((instance, entry))
}
