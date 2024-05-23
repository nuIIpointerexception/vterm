use crate::errors::VulkanDebugError;

pub trait VulkanDebug {
    fn set_debug_name(
        &self,
        debug_name: impl Into<String>,
    ) -> Result<(), VulkanDebugError>;
}
