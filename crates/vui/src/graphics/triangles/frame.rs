use std::sync::Arc;

use ::ash::vk;

use crate::{
    asset_loader::CombinedImageSampler,
    errors::VulkanError,
    graphics::{Vertex, VertexStream},
    vulkan::{
        allocator::MemoryAllocator,
        buffer::{Buffer, GpuVec},
        command_buffer::CommandBuffer,
        descriptor_set::{DescriptorPool, DescriptorSet, DescriptorSetLayout},
        pipeline::layout::PipelineLayout,
        render_device::RenderDevice,
    },
};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct UniformBufferData {
    pub view_projection: [[f32; 4]; 4],
}

pub struct Frame {
    _descriptor_pool: DescriptorPool,

    descriptor_set: DescriptorSet,

    uniform_data: Buffer,

    vertex_data: GpuVec<Vertex>,

    vertex_data_needs_rebound: bool,

    index_data: GpuVec<u32>,

    vk_dev: Arc<RenderDevice>,
}

impl Frame {
    pub fn new(
        vk_dev: Arc<RenderDevice>,
        vk_alloc: Arc<dyn MemoryAllocator>,
        textures: &[CombinedImageSampler],
        descriptor_layout: &DescriptorSetLayout,
    ) -> Result<Self, VulkanError> {
        let descriptor_pool = DescriptorPool::new(
            vk_dev.clone(),
            1,
            &[
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::STORAGE_BUFFER,
                    descriptor_count: 1,
                },
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::UNIFORM_BUFFER,
                    descriptor_count: 1,
                },
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                    descriptor_count: textures.len() as u32,
                },
            ],
        )?;
        let descriptor_set = descriptor_pool
            .allocate_with_variable_counts(
                descriptor_layout,
                1,
                textures.len() as u32,
            )?
            .pop()
            .unwrap();

        let vertex_data = GpuVec::new(
            vk_dev.clone(),
            vk_alloc.clone(),
            vk::BufferUsageFlags::STORAGE_BUFFER,
            1,
        )?;
        let index_data = GpuVec::new(
            vk_dev.clone(),
            vk_alloc.clone(),
            vk::BufferUsageFlags::INDEX_BUFFER,
            500,
        )?;
        let mut uniform_data = Buffer::new(
            vk_dev.clone(),
            vk_alloc.clone(),
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE
                | vk::MemoryPropertyFlags::HOST_COHERENT,
            std::mem::size_of::<UniformBufferData>() as u64,
        )?;
        uniform_data.map()?;

        unsafe {
            descriptor_set.bind_buffer(
                1,
                &uniform_data.raw,
                vk::DescriptorType::UNIFORM_BUFFER,
            );
            for (texture_index, texture) in textures.iter().enumerate() {
                descriptor_set.bind_combined_image_sampler(
                    2,
                    texture_index as u32,
                    &texture.image_view,
                    &texture.sampler,
                );
            }
        }

        Ok(Self {
            vertex_data,
            vertex_data_needs_rebound: true,
            index_data,
            uniform_data,
            _descriptor_pool: descriptor_pool,
            descriptor_set,
            vk_dev,
        })
    }

    pub fn set_view_projection(
        &mut self,
        view_projection: nalgebra::Matrix4<f32>,
    ) -> anyhow::Result<()> {
        self.uniform_data.data_mut::<UniformBufferData>().unwrap()[0] =
            UniformBufferData {
                view_projection: view_projection.into(),
            };
        Ok(())
    }
}

impl VertexStream for Frame {
    fn push_vertices(
        &mut self,
        vertices: &[Vertex],
        indices: &[u32],
    ) -> Result<(), anyhow::Error> {
        let base_index = self.vertex_data.len() as u32;
        for vertex in vertices {
            self.push_vertex(*vertex)?;
        }
        for index in indices {
            self.index_data.push_back(base_index + index)?;
        }
        Ok(())
    }
}

impl Frame {
    pub(super) unsafe fn write_frame_commands(
        &mut self,
        cmd: &CommandBuffer,
        pipeline_layout: &PipelineLayout,
    ) {
        if self.vertex_data_needs_rebound {
            self.rebind_vertex_data();
            self.vertex_data_needs_rebound = false;
        }

        self.vk_dev.logical_device.cmd_bind_descriptor_sets(
            cmd.raw,
            vk::PipelineBindPoint::GRAPHICS,
            pipeline_layout.raw,
            0,
            &[self.descriptor_set.raw],
            &[],
        );
        self.vk_dev.logical_device.cmd_bind_index_buffer(
            cmd.raw,
            self.index_data.buffer.raw,
            0,
            vk::IndexType::UINT32,
        );
        self.vk_dev.logical_device.cmd_draw_indexed(
            cmd.raw,
            self.index_data.len() as u32,
            1,
            0,
            0,
            0,
        );
    }

    pub(super) fn clear(&mut self) {
        self.vertex_data.clear();
        self.index_data.clear();
    }

    fn push_vertex(&mut self, vertex: Vertex) -> Result<(), anyhow::Error> {
        self.vertex_data_needs_rebound |= self.vertex_data.push_back(vertex)?;
        Ok(())
    }

    unsafe fn rebind_vertex_data(&mut self) {
        self.descriptor_set.bind_buffer(
            0,
            &self.vertex_data.buffer.raw,
            vk::DescriptorType::STORAGE_BUFFER,
        );
    }
}
