use ash::{extensions::khr, vk};

use crate::errors::WindowSurfaceError;

pub struct WindowSurface {
    pub loader: khr::Surface,
    pub khr: vk::SurfaceKHR,
}

impl WindowSurface {
    pub fn new(
        surface_khr: vk::SurfaceKHR,
        surface_loader: khr::Surface,
    ) -> Self {
        Self {
            loader: surface_loader,
            khr: surface_khr,
        }
    }

    pub unsafe fn get_physical_device_surface_support(
        &self,
        physical_device: &vk::PhysicalDevice,
        queue_family_index: u32,
    ) -> Result<bool, WindowSurfaceError> {
        self.loader
            .get_physical_device_surface_support(
                *physical_device,
                queue_family_index,
                self.khr,
            )
            .map_err(
                WindowSurfaceError::UnableToCheckPhysicalDeviceSurfaceSupport,
            )
    }

    pub unsafe fn supported_formats(
        &self,
        physical_device: &vk::PhysicalDevice,
    ) -> Vec<vk::SurfaceFormatKHR> {
        self.loader
            .get_physical_device_surface_formats(*physical_device, self.khr)
            .unwrap_or_else(|_| vec![])
    }

    pub unsafe fn supported_presentation_modes(
        &self,
        physical_device: &vk::PhysicalDevice,
    ) -> Vec<vk::PresentModeKHR> {
        self.loader
            .get_physical_device_surface_present_modes(
                *physical_device,
                self.khr,
            )
            .unwrap_or_else(|_| vec![])
    }

    pub unsafe fn surface_capabilities(
        &self,
        physical_device: &vk::PhysicalDevice,
    ) -> Result<vk::SurfaceCapabilitiesKHR, WindowSurfaceError> {
        self.loader
            .get_physical_device_surface_capabilities(
                *physical_device,
                self.khr,
            ).map_err(WindowSurfaceError::UnableToGetPhysicalDeviceSurfaceCapabilities)
    }
}

impl Drop for WindowSurface {
    fn drop(&mut self) {
        unsafe {
            self.loader.destroy_surface(self.khr, None);
        }
    }
}
