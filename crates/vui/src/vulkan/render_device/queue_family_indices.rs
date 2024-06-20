use ash::vk;

use crate::{
    errors::QueueSelectionError,
    vulkan::{render_device::GpuQueue, window_surface::WindowSurface},
};

const SINGLE_QUEUE_PRIORITY: [f32; 1] = [1.0];

pub struct QueueFamilyIndices {
    graphics_family_index: u32,

    present_family_index: u32,
}

impl QueueFamilyIndices {
    pub fn find(
        ash: &ash::Instance,
        physical_device: &vk::PhysicalDevice,
        window_surface: &WindowSurface,
    ) -> Result<Self, QueueSelectionError> {
        let queue_families =
            unsafe { ash.get_physical_device_queue_family_properties(*physical_device) };

        let mut graphics_family = None;
        let mut present_family = None;

        queue_families.iter().enumerate().for_each(|(i, family)| {
            if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                graphics_family = Some(i as u32);
            }

            let present_support = unsafe {
                window_surface.get_physical_device_surface_support(&physical_device, i as u32)
            };
            match present_support {
                Ok(true) => {
                    present_family = Some(i as u32);
                }
                Err(ref error) => {
                    log::warn!("Error while checking surface support for device: {:?}", error);
                }
                _ => {}
            }
        });

        let graphics_family_index =
            graphics_family.ok_or(QueueSelectionError::UnableToFindGraphicsQueue)?;

        let present_family_index =
            present_family.ok_or(QueueSelectionError::UnableToFindPresentQueue)?;

        Ok(Self { graphics_family_index, present_family_index })
    }

    pub fn as_queue_create_infos(&self) -> Vec<vk::DeviceQueueCreateInfo> {
        let mut create_infos = vec![vk::DeviceQueueCreateInfo {
            queue_family_index: self.graphics_family_index,
            p_queue_priorities: SINGLE_QUEUE_PRIORITY.as_ptr(),
            queue_count: 1,
            ..Default::default()
        }];

        if self.graphics_family_index != self.present_family_index {
            create_infos.push(vk::DeviceQueueCreateInfo {
                queue_family_index: self.present_family_index,
                p_queue_priorities: SINGLE_QUEUE_PRIORITY.as_ptr(),
                queue_count: 1,
                ..Default::default()
            });
        }

        create_infos
    }

    pub fn get_queues(&self, logical_device: &ash::Device) -> (GpuQueue, GpuQueue) {
        let raw_graphics_queue =
            unsafe { logical_device.get_device_queue(self.graphics_family_index, 0) };
        let graphics_queue = GpuQueue::from_raw(raw_graphics_queue, self.graphics_family_index, 0);

        let is_same_family = self.graphics_family_index == self.present_family_index;
        let present_queue = if is_same_family {
            graphics_queue
        } else {
            let raw_present_queue =
                unsafe { logical_device.get_device_queue(self.present_family_index, 0) };
            GpuQueue::from_raw(raw_present_queue, self.present_family_index, 0)
        };

        (graphics_queue, present_queue)
    }
}
