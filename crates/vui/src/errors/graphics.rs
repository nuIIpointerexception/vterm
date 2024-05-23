use ::thiserror::Error;

#[derive(Debug, Error)]
pub enum ImmediateModeGraphicsError {
    #[error("The per frame resources for swapchain image {} were not available! The last frame may not have been ended properly.", .0)]
    FrameResourcesUnavailable(usize),
}
