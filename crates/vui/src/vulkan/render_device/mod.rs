use std::{fmt, sync::Mutex};

use ash::vk;

use crate::{
    errors::RenderDeviceError,
    vulkan::{instance::Instance, window_surface::WindowSurface},
};
pub use crate::vulkan::render_device::{
    gpu_queue::GpuQueue, queue_family_indices::QueueFamilyIndices,
    swapchain::Swapchain,
};

mod gpu_queue;
mod physical_device;
mod queue_family_indices;
mod swapchain;

pub struct RenderDevice {
    #[allow(unused)]
    pub physical_device: vk::PhysicalDevice,

    pub logical_device: ash::Device,

    pub graphics_queue: GpuQueue,

    pub present_queue: GpuQueue,

    pub swapchain: Mutex<Option<Swapchain>>,

    pub window_surface: WindowSurface,

    pub instance: Instance,
}

impl RenderDevice {
    pub fn new(
        instance: Instance,
        window_surface: WindowSurface,
    ) -> Result<Self, RenderDeviceError> {
        let physical_device =
            physical_device::find_optimal(&instance.ash, &window_surface)?;

        let queue_family_indices = QueueFamilyIndices::find(
            &instance.ash,
            &physical_device,
            &window_surface,
        )?;
        let logical_device = instance.create_logical_device(
            &physical_device,
            &physical_device::required_extensions(),
            &queue_family_indices.as_queue_create_infos(),
        )?;
        let (graphics_queue, present_queue) =
            queue_family_indices.get_queues(&logical_device);

        let vk_dev = Self {
            instance,
            physical_device,
            logical_device,
            graphics_queue,
            present_queue,
            window_surface,
            swapchain: Mutex::new(None),
        };

        Ok(vk_dev)
    }
}

impl Drop for RenderDevice {
    fn drop(&mut self) {
        unsafe {
            let mut swapchain = self
                .swapchain
                .lock()
                .expect("Unable to acquire the swapchain mutex");
            if let Some(swapchain) = swapchain.take() {
                self.destroy_swapchain(swapchain)
                    .expect("Error while destroying the swapchain");
            }
            self.logical_device
                .device_wait_idle()
                .expect("Error while waiting for device work to finish");
            self.logical_device.destroy_device(None);
        }
    }
}

impl fmt::Display for RenderDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let device_properties = unsafe {
            self.instance
                .ash
                .get_physical_device_properties(self.physical_device)
        };
        let device_name = unsafe {
            std::ffi::CStr::from_ptr(device_properties.device_name.as_ptr())
                .to_string_lossy()
                .into_owned()
        };
        let device_type = match device_properties.device_type {
            vk::PhysicalDeviceType::OTHER => "other",
            vk::PhysicalDeviceType::INTEGRATED_GPU => "integrated gpu",
            vk::PhysicalDeviceType::DISCRETE_GPU => "discrete gpu",
            vk::PhysicalDeviceType::VIRTUAL_GPU => "virtual gpu",
            vk::PhysicalDeviceType::CPU => "cpu",
            _ => "Unknown",
        };
        let driver_version: u32 = device_properties.driver_version;
        let driver_version_str = format!(
            "{}.{}.{}",
            vk::api_version_major(driver_version),
            vk::api_version_minor(driver_version),
            vk::api_version_patch(driver_version)
        );

        let api_version: u32 = device_properties.api_version;
        let api_version_str = format!(
            "{}.{}.{}",
            vk::api_version_major(api_version),
            vk::api_version_minor(api_version),
            vk::api_version_patch(api_version)
        );

        let device_features = unsafe {
            let features = self
                .instance
                .ash
                .get_physical_device_features(self.physical_device);
            features
        };

        write!(
            f,
            "GPU: {}\nType: {}\ndriver ver: {}\nvulkan api: {}\n\ndevice features:\n",
            device_name, device_type, driver_version_str, api_version_str
        )?;

        writeln!(f, "  geometry_shader: {}", device_features.geometry_shader)?;
        writeln!(
            f,
            "  tessellation_shader: {}",
            device_features.tessellation_shader
        )?;
        writeln!(f, "  multi_viewport: {}", device_features.multi_viewport)?;
        writeln!(
            f,
            "  texture_compression_bc: {}",
            device_features.texture_compression_bc
        )?;
        writeln!(
            f,
            "  texture_compression_etc2: {}",
            device_features.texture_compression_etc2
        )?;
        writeln!(
            f,
            "  texture_compression_astc_ldr: {}",
            device_features.texture_compression_astc_ldr
        )?;
        writeln!(f, "  shader_float64: {}", device_features.shader_float64)?;
        writeln!(f, "  shader_int64: {}", device_features.shader_int64)?;

        Ok(())
    }
}
