use crate::graphics::{renderer::Renderer, window::WindowWrapper};

use super::{context::{VulkanContext, VulkanError}, framebuffer::Framebuffer, renderpass::Renderpass};

pub struct VulkanRenderer {
    renderpass: Renderpass,
    framebuffers: Vec<Framebuffer>,
}

impl VulkanRenderer {
    pub fn new(context: &VulkanContext, window: &WindowWrapper) -> Result<VulkanRenderer, VulkanError> {
        let renderpass = Renderpass::new(&context.surface, &context.device)?;
        let framebuffers = {
            let mut out = Vec::new();
            for image_view in context.swapchain_image_views.iter() {
                out.push(Framebuffer::new(&context.surface, &context.device, image_view, &renderpass, window)?);
            }
            out
        };

        Ok(VulkanRenderer { renderpass, framebuffers })
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
