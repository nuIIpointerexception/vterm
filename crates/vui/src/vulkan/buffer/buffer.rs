use std::sync::Arc;

use ::ash::vk;

use crate::{
    errors::BufferError,
    vulkan::{
        allocator::{Allocation, MemoryAllocator},
        render_device::RenderDevice,
    },
};

#[derive(Clone)]
pub struct Buffer {
    pub raw: vk::Buffer,

    pub allocation: Allocation,

    pub mapped_ptr: Option<*mut std::ffi::c_void>,

    pub vk_alloc: Arc<dyn MemoryAllocator>,

    pub vk_dev: Arc<RenderDevice>,
}

impl Buffer {
    pub fn new(
        vk_dev: Arc<RenderDevice>,
        vk_alloc: Arc<dyn MemoryAllocator>,
        buffer_usage_flags: vk::BufferUsageFlags,
        memory_property_flags: vk::MemoryPropertyFlags,
        size_in_bytes: u64,
    ) -> Result<Self, BufferError> {
        let create_info = vk::BufferCreateInfo {
            size: size_in_bytes,
            usage: buffer_usage_flags,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };
        let buffer_handle = unsafe {
            vk_dev.logical_device.create_buffer(&create_info, None).map_err(|err| {
                BufferError::UnableToCreateBuffer {
                    size: size_in_bytes,
                    usage: buffer_usage_flags,
                    source: err,
                }
            })?
        };
        let allocation = unsafe {
            let buffer_memory_requirements =
                vk_dev.logical_device.get_buffer_memory_requirements(buffer_handle);
            vk_alloc.allocate_memory(buffer_memory_requirements, memory_property_flags)?
        };
        unsafe {
            vk_dev
                .logical_device
                .bind_buffer_memory(buffer_handle, allocation.memory, allocation.offset)
                .map_err(BufferError::UnableToBindDeviceMemory)?;
        }

        Ok(Self { raw: buffer_handle, allocation, mapped_ptr: None, vk_alloc, vk_dev })
    }

    pub fn map(&mut self) -> Result<(), BufferError> {
        let ptr = unsafe {
            self.vk_dev
                .logical_device
                .map_memory(
                    self.allocation.memory,
                    self.allocation.offset,
                    self.allocation.byte_size,
                    vk::MemoryMapFlags::empty(),
                )
                .map_err(BufferError::UnableToMapDeviceMemory)?
        };
        self.mapped_ptr = Some(ptr);
        Ok(())
    }

    pub fn unmap(&mut self) {
        unsafe {
            self.vk_dev.logical_device.unmap_memory(self.allocation.memory);
        }
        self.mapped_ptr = None;
    }

    pub fn data<'element, Element: 'element + Copy>(
        &self,
    ) -> Result<&'element [Element], BufferError> {
        let ptr = self.mapped_ptr.ok_or(BufferError::NoMappedPointerFound)?;
        let elements = (self.allocation.byte_size as usize) / std::mem::size_of::<Element>();
        let data = unsafe { std::slice::from_raw_parts(ptr as *const Element, elements) };
        Ok(data)
    }

    pub fn data_mut<'element, Element: 'element + Copy>(
        &self,
    ) -> Result<&'element mut [Element], BufferError> {
        let ptr = self.mapped_ptr.ok_or(BufferError::NoMappedPointerFound)?;
        let elements = (self.allocation.byte_size as usize) / std::mem::size_of::<Element>();
        let data = unsafe { std::slice::from_raw_parts_mut(ptr as *mut Element, elements) };
        Ok(data)
    }
}
