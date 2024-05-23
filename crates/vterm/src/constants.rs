use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use vui::vulkan::render_device::RenderDevice;

static CURRENT_FPS: AtomicU64 = AtomicU64::new(0);

pub fn get_fps() -> u64 {
    CURRENT_FPS.load(Ordering::Relaxed)
}

pub fn set_fps(fps: u64) {
    CURRENT_FPS.store(fps, Ordering::Relaxed);
}

pub static mut GPU: Option<Arc<RenderDevice>> = None;

pub fn get_gpu() -> &'static Arc<RenderDevice> {
    unsafe { GPU.as_ref().unwrap() }
}

pub fn set_gpu(gpu: Arc<RenderDevice>) {
    unsafe { GPU = Some(gpu) }
}
