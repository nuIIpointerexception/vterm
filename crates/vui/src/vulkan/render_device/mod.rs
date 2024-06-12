use std::sync::Mutex;

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
