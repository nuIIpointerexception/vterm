use ash::vk;

use crate::{
    errors::PhysicalDeviceError,
    vulkan::{
        render_device::QueueFamilyIndices, window_surface::WindowSurface,
    },
};

pub fn find_optimal(
    ash: &ash::Instance,
    window_surface: &WindowSurface,
) -> Result<vk::PhysicalDevice, PhysicalDeviceError> {
    let physical_devices = unsafe {
        ash.enumerate_physical_devices()
            .map_err(PhysicalDeviceError::UnableToEnumerateDevices)?
    };
    let physical_device = physical_devices
        .iter()
        .find(|device| is_device_suitable(ash, device, window_surface))
        .ok_or(PhysicalDeviceError::NoSuitableDeviceFound)?;
    Ok(*physical_device)
}

fn is_device_suitable(
    ash: &ash::Instance,
    physical_device: &vk::PhysicalDevice,
    window_surface: &WindowSurface,
) -> bool {
    let queues_supported =
        QueueFamilyIndices::find(ash, physical_device, window_surface).is_ok();

    let format_available = unsafe { !window_surface.supported_formats(physical_device).is_empty() };


    let presentation_mode_available = unsafe {
        !window_surface
            .supported_presentation_modes(physical_device)
            .is_empty()
    };

    queues_supported
        && format_available
        && presentation_mode_available
}