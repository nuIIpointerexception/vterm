use std::sync::Arc;

use crate::vulkan::image::{sampler::Sampler, view::ImageView};

#[derive(Clone)]
pub struct CombinedImageSampler {
    pub image_view: Arc<ImageView>,
    pub sampler: Arc<Sampler>,
}

impl CombinedImageSampler {
    pub fn of(image_view: ImageView, sampler: Sampler) -> Self {
        Self::new(Arc::new(image_view), Arc::new(sampler))
    }

    pub fn new(image_view: Arc<ImageView>, sampler: Arc<Sampler>) -> Self {
        Self {
            image_view,
            sampler,
        }
    }
}
