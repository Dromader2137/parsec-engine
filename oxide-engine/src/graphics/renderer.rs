use crate::error::EngineError;

use super::{vulkan::context::VulkanContext, window::WindowWrapper};

pub trait Renderer {
    fn render(
        &mut self,
        vulkan_context: &VulkanContext,
        window: &WindowWrapper,
    ) -> Result<(), EngineError>;
    fn handle_resize(&mut self) -> Result<(), EngineError>;
}
