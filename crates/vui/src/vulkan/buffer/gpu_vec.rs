use std::sync::Arc;

use ::ash::vk;

use crate::{
    errors::BufferError,
    vulkan::{
        allocator::MemoryAllocator, buffer::Buffer, render_device::RenderDevice,
    },
};

pub struct GpuVec<T: Copy> {
    pub buffer: Buffer,

    capacity: u32,

    length: u32,

    usage_flags: vk::BufferUsageFlags,

    _phantom_data: std::marker::PhantomData<T>,
}

impl<T: Copy> GpuVec<T> {
    pub fn new(
        vk_dev: Arc<RenderDevice>,
        vk_alloc: Arc<dyn MemoryAllocator>,
        buffer_usage_flags: vk::BufferUsageFlags,
        initial_capacity: u32,
    ) -> Result<Self, BufferError> {
        let mut buffer = Buffer::new(
            vk_dev,
            vk_alloc,
            buffer_usage_flags,
            vk::MemoryPropertyFlags::HOST_VISIBLE
                | vk::MemoryPropertyFlags::HOST_COHERENT,
            Self::element_count_to_bytes(initial_capacity),
        )?;
        buffer.map()?;
        Ok(Self {
            buffer,
            capacity: initial_capacity,
            length: 0,
            usage_flags: buffer_usage_flags,
            _phantom_data: std::marker::PhantomData::default(),
        })
    }

    pub fn push_back(&mut self, value: T) -> Result<bool, BufferError> {
        let mut replaced = false;
        if self.length == self.capacity {
            self.grow(self.length * 2)?;
            replaced = true;
        }
        let data = self.buffer.data_mut()?;
        data[self.len()] = value;
        self.length = self.length + 1;
        Ok(replaced)
    }

    pub fn clear(&mut self) {
        self.length = 0;
    }

    pub fn len(&self) -> usize {
        self.length as usize
    }

    pub fn len_bytes(&self) -> u64 {
        Self::element_count_to_bytes(self.length)
    }
}

impl<T: Copy> GpuVec<T> {
    fn element_count_to_bytes(count: u32) -> u64 {
        count as u64 * std::mem::size_of::<T>() as u64
    }

    fn grow(&mut self, desired_capacity: u32) -> Result<(), BufferError> {
        let mut buffer = Buffer::new(
            self.buffer.vk_dev.clone(),
            self.buffer.vk_alloc.clone(),
            self.usage_flags,
            vk::MemoryPropertyFlags::HOST_VISIBLE
                | vk::MemoryPropertyFlags::HOST_COHERENT,
            Self::element_count_to_bytes(desired_capacity),
        )?;
        buffer.map()?;
        self.capacity = desired_capacity;

        {
            let new_data = buffer.data_mut::<T>()?;
            let old_data = self.buffer.data::<T>()?;
            new_data[..old_data.len()].copy_from_slice(old_data);
        }

        std::mem::swap(&mut self.buffer, &mut buffer);
        Ok(())
    }
}
