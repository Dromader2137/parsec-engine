use crate::graphics::{renderer::Renderer, window::WindowWrapper};

use super::context::VulkanContext;

#[derive(Debug)]
pub struct VulkanRenderer {}

impl VulkanRenderer {
    pub fn new(context: &VulkanContext, window: &WindowWrapper) -> Result<VulkanRenderer, ()> {
        Ok(VulkanRenderer {})
    }
}

impl Renderer for VulkanRenderer {
    fn handle_resize(&mut self) -> Result<(), crate::error::EngineError> {
        Ok(())
    }

    fn render(
        &mut self,
        vulkan_context: &VulkanContext,
        window: &WindowWrapper,
    ) -> Result<(), crate::error::EngineError> {
        Ok(())
    }
}
