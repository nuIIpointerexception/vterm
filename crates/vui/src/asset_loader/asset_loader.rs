use std::{path::Path, sync::Arc};

use ash::vk;
use image::{imageops, imageops::FilterType, io::Reader};

use crate::{
    asset_loader::{AssetLoaderError, CombinedImageSampler, MipmapData},
    errors::VulkanError,
    vulkan::{
        allocator::MemoryAllocator,
        buffer::GpuVec,
        command_buffer::OneTimeSubmitCommandPool,
        image::{sampler::Sampler, view::ImageView, Image},
        render_device::RenderDevice,
    },
};

#[derive(Clone)]
pub struct AssetLoader {
    textures: Vec<CombinedImageSampler>,
    default_sampler: Arc<Sampler>,
    staging_buffer: GpuVec<u8>,
    command_pool: OneTimeSubmitCommandPool,
    vk_alloc: Arc<dyn MemoryAllocator>,
    vk_dev: Arc<RenderDevice>,
}

impl AssetLoader {
    pub fn new(
        vk_dev: Arc<RenderDevice>,
        vk_alloc: Arc<dyn MemoryAllocator>,
    ) -> Result<Self, AssetLoaderError> {
        let mut loader = Self {
            textures: vec![],
            default_sampler: Arc::new(
                Sampler::linear(vk_dev.clone())
                    .map_err(VulkanError::ImageError)?,
            ),
            staging_buffer: GpuVec::new(
                vk_dev.clone(),
                vk_alloc.clone(),
                vk::BufferUsageFlags::TRANSFER_SRC,
                (8 * 4) * 512 * 512,
            )
            .map_err(VulkanError::BufferError)?,
            command_pool: OneTimeSubmitCommandPool::new(
                vk_dev.clone(),
                &vk_dev.graphics_queue,
            )
            .map_err(VulkanError::CommandBufferError)?,
            vk_alloc,
            vk_dev,
        };
        loader.create_texture_with_data(&[MipmapData {
            width: 1,
            height: 1,
            data: vec![0xFF, 0xFF, 0xFF, 0xFF],
        }])?;
        Ok(loader)
    }

    pub fn textures(&self) -> &[CombinedImageSampler] {
        &self.textures
    }

    pub fn create_texture_with_data(
        &mut self,
        mipmaps: &[MipmapData],
    ) -> Result<i32, AssetLoaderError> {
        let vulkan_image = self.create_empty_2d(
            mipmaps[0].width,
            mipmaps[0].height,
            mipmaps.len() as u32,
        )?;

        self.staging_buffer.clear();
        for mipmap in mipmaps {
            for byte in &mipmap.data {
                self.staging_buffer
                    .push_back(*byte)
                    .map_err(VulkanError::BufferError)?;
            }
        }

        self.command_pool
            .submit_sync_commands(|vk_dev, cmd| unsafe {
                let prepare_write_barrier = vk::ImageMemoryBarrier {
                    src_access_mask: vk::AccessFlags::empty(),
                    dst_access_mask: vk::AccessFlags::TRANSFER_WRITE,
                    old_layout: vk::ImageLayout::UNDEFINED,
                    new_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                    dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                    image: vulkan_image.raw,
                    subresource_range: vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: mipmaps.len() as u32,
                        base_array_layer: 0,
                        layer_count: 1,
                    },
                    ..Default::default()
                };
                vk_dev.logical_device.cmd_pipeline_barrier(
                    cmd,
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[prepare_write_barrier],
                );

                let mut buffer_offset = 0;
                for (current_level, mipmap) in mipmaps.iter().enumerate() {
                    let buffer_image_copy = vk::BufferImageCopy {
                        buffer_offset,
                        buffer_row_length: 0,
                        buffer_image_height: 0,
                        image_subresource: vk::ImageSubresourceLayers {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            mip_level: current_level as u32,
                            base_array_layer: 0,
                            layer_count: 1,
                        },
                        image_offset: vk::Offset3D::default(),
                        image_extent: vk::Extent3D {
                            width: mipmap.width,
                            height: mipmap.height,
                            depth: 1,
                        },
                    };
                    vk_dev.logical_device.cmd_copy_buffer_to_image(
                        cmd,
                        self.staging_buffer.buffer.raw,
                        vulkan_image.raw,
                        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                        &[buffer_image_copy],
                    );
                    buffer_offset += mipmap.data.len() as u64;
                }

                let prepare_read_barrier = vk::ImageMemoryBarrier {
                    src_access_mask: vk::AccessFlags::TRANSFER_WRITE,
                    dst_access_mask: vk::AccessFlags::SHADER_READ,
                    old_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    new_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                    src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                    dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                    image: vulkan_image.raw,
                    subresource_range: vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: mipmaps.len() as u32,
                        base_array_layer: 0,
                        layer_count: 1,
                    },
                    ..Default::default()
                };
                vk_dev.logical_device.cmd_pipeline_barrier(
                    cmd,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::FRAGMENT_SHADER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[prepare_read_barrier],
                );
            })
            .map_err(VulkanError::CommandBufferError)?;

        let image_view = Arc::new(
            ImageView::new_2d(
                Arc::new(vulkan_image),
                vk::Format::R8G8B8A8_SRGB,
                vk::ImageAspectFlags::COLOR,
            )
            .map_err(VulkanError::ImageError)?,
        );
        let texture =
            CombinedImageSampler::new(image_view, self.default_sampler.clone());
        self.textures.push(texture.clone());

        Ok((self.textures.len() - 1) as i32)
    }

    pub fn read_texture<T>(
        &mut self,
        path_to_texture_image: T,
    ) -> Result<i32, AssetLoaderError>
    where
        T: AsRef<Path>,
    {
        let loaded = Reader::open(path_to_texture_image)?.decode()?;
        let rgba = loaded.into_rgba8();
        let (width, height) = (rgba.width(), rgba.height());

        let mipmap_count = Self::compute_mipmap_count(width, height);
        let mipmaps: Vec<_> = (0..mipmap_count)
            .map(|i| {
                let mipmap = imageops::resize(
                    &rgba,
                    (width >> i).max(1),
                    (height >> i).max(1),
                    FilterType::Triangle,
                );
                MipmapData {
                    width: mipmap.width(),
                    height: mipmap.height(),
                    data: mipmap.into_raw(),
                }
            })
            .collect();

        self.create_texture_with_data(&mipmaps)
    }
}

impl AssetLoader {
    fn create_empty_2d(
        &mut self,
        width: u32,
        height: u32,
        mip_levels: u32,
    ) -> Result<Image, VulkanError> {
        let create_info = vk::ImageCreateInfo {
            flags: vk::ImageCreateFlags::empty(),
            image_type: vk::ImageType::TYPE_2D,
            format: vk::Format::R8G8B8A8_SRGB,
            extent: vk::Extent3D {
                width,
                height,
                depth: 1,
            },
            mip_levels,
            array_layers: 1,
            samples: vk::SampleCountFlags::TYPE_1,
            tiling: vk::ImageTiling::OPTIMAL,
            usage: vk::ImageUsageFlags::TRANSFER_DST
                | vk::ImageUsageFlags::SAMPLED,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };
        let image = Image::new(
            self.vk_dev.clone(),
            self.vk_alloc.clone(),
            &create_info,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;
        Ok(image)
    }

    fn compute_mipmap_count(width: u32, height: u32) -> u32 {
        let max_dimension = (width as f32).max(height as f32);
        let powers_of_two = max_dimension.log2().floor();
        (powers_of_two + 1.0) as u32
    }
}

#[cfg(test)]
mod test {
    use super::AssetLoader;

    #[test]
    fn test_mipmap_count() {
        assert_eq!(AssetLoader::compute_mipmap_count(1, 1), 1);
        assert_eq!(AssetLoader::compute_mipmap_count(2, 1), 2);
        assert_eq!(AssetLoader::compute_mipmap_count(1, 2), 2);
        assert_eq!(AssetLoader::compute_mipmap_count(512, 64), 10);
        assert_eq!(AssetLoader::compute_mipmap_count(513, 1023), 10);
        assert_eq!(AssetLoader::compute_mipmap_count(513, 1025), 11);
    }
}
