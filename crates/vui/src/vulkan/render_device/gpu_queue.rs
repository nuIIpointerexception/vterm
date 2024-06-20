use ash::vk;

#[derive(Debug, Clone, Copy)]
pub struct GpuQueue {
    pub queue: vk::Queue,
    pub family_id: u32,
    pub index: u32,
}

impl GpuQueue {
    pub fn from_raw(queue: vk::Queue, family_id: u32, index: u32) -> Self {
        Self { queue, family_id, index }
    }

    pub fn is_same(&self, queue: &GpuQueue) -> bool {
        self.family_id == queue.family_id && self.index == queue.index
    }
}
