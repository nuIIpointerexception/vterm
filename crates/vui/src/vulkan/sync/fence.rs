use std::sync::Arc;

use ash::vk;

use crate::{errors::FenceError, vulkan::render_device::RenderDevice};

pub struct Fence {
    pub raw: vk::Fence,

    pub vk_dev: Arc<RenderDevice>,
}

impl Fence {
    pub fn new(vk_dev: Arc<RenderDevice>) -> Result<Self, FenceError> {
        let raw = {
            let create_info = vk::FenceCreateInfo {
                flags: vk::FenceCreateFlags::SIGNALED,
                ..Default::default()
            };
            unsafe {
                vk_dev
                    .logical_device
                    .create_fence(&create_info, None)
                    .map_err(FenceError::UnableToCreateFence)?
            }
        };
        Ok(Self { raw, vk_dev })
    }

    pub fn wait_and_reset(&self) -> Result<(), FenceError> {
        self.wait()?;
        self.reset()
    }

    pub fn wait(&self) -> Result<(), FenceError> {
        unsafe {
            self.vk_dev
                .logical_device
                .wait_for_fences(&[self.raw], true, u64::MAX)
                .map_err(FenceError::UnexpectedWaitError)?;
        }
        Ok(())
    }

    pub fn reset(&self) -> Result<(), FenceError> {
        unsafe {
            self.vk_dev
                .logical_device
                .reset_fences(&[self.raw])
                .map_err(FenceError::UnexpectedResetError)?;
        }
        Ok(())
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        unsafe {
            self.vk_dev.logical_device.destroy_fence(self.raw, None);
        }
    }
}
