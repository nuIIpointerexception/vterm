use std::sync::Arc;

use ash::vk;

use crate::{
    errors::ImageError,
    vulkan::{
        allocator::{Allocation, MemoryAllocator},
        render_device::RenderDevice,
    },
};

pub mod sampler;
pub mod view;

pub struct Image {
    pub raw: vk::Image,

    pub create_info: vk::ImageCreateInfo<'static>,

    pub allocation: Allocation,

    pub vk_alloc: Arc<dyn MemoryAllocator>,

    pub vk_dev: Arc<RenderDevice>,
}

impl Image {
    pub fn new(
        vk_dev: Arc<RenderDevice>,
        vk_alloc: Arc<dyn MemoryAllocator>,
        create_info: &vk::ImageCreateInfo<'static>,
        memory_property_flags: vk::MemoryPropertyFlags,
    ) -> Result<Self, ImageError> {
        let raw = unsafe {
            vk_dev
                .logical_device
                .create_image(create_info, None)
                .map_err(ImageError::UnableToCreateImage)?
        };
        let memory_requirements =
            unsafe { vk_dev.logical_device.get_image_memory_requirements(raw) };

        let allocation = unsafe {
            vk_alloc
                .allocate_memory(memory_requirements, memory_property_flags)?
        };

        unsafe {
            vk_dev
                .logical_device
                .bind_image_memory(raw, allocation.memory, allocation.offset)
                .map_err(ImageError::UnableToBindImageMemory)?;
        }

        Ok(Self {
            raw,
            create_info: *create_info,
            allocation,
            vk_alloc,
            vk_dev,
        })
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            self.vk_dev.logical_device.destroy_image(self.raw, None);
            self.vk_alloc
                .free(&self.allocation)
                .expect("unable to free the image's memory");
        }
    }
}
