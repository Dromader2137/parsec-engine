use crate::graphics::{renderer::Renderer, window::WindowWrapper};

use super::{command_buffer::CommandBuffer, context::{VulkanContext, VulkanError}, fence::Fence, framebuffer::Framebuffer, renderpass::Renderpass, semaphore::Semaphore};

#[allow(unused)]
pub struct VulkanRenderer {
    renderpass: Renderpass,
    framebuffers: Vec<Framebuffer>,
    command_buffer: CommandBuffer,
    command_buffer_reuse_fence: Fence,
    rendering_semaphore: Semaphore,
    present_semaphore: Semaphore,
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
        let command_buffer = CommandBuffer::new(&context.device, &context.command_pool)?;
        command_buffer.begin(&context.device)?;
        command_buffer.end(&context.device)?;
        let command_buffer_reuse_fence = Fence::new(&context.device, true)?;
        let rendering_semaphore = Semaphore::new(&context.device)?;
        let present_semaphore = Semaphore::new(&context.device)?;

        Ok(VulkanRenderer { renderpass, framebuffers, command_buffer, command_buffer_reuse_fence, rendering_semaphore, present_semaphore })
    }
}

impl From<VulkanError> for crate::error::EngineError {
    fn from(value: VulkanError) -> Self {
        crate::error::EngineError::Graphics(format!("{:?}", value))
    }
}

impl Renderer for VulkanRenderer {
    fn handle_resize(&mut self) -> Result<(), crate::error::EngineError> {
        Ok(())
    }

    fn render(
        &mut self,
        vulkan_context: &VulkanContext,
        _window: &WindowWrapper,
    ) -> Result<(), crate::error::EngineError> {
        let (present_index, _) = vulkan_context.swapchain.acquire_next_image(&self.present_semaphore, &Fence::null()).map_err(VulkanError::from)?;
        self.command_buffer_reuse_fence.wait(&vulkan_context.device).map_err(VulkanError::from)?;
        self.command_buffer_reuse_fence.reset(&vulkan_context.device).map_err(VulkanError::from)?;
        self.command_buffer.reset(&vulkan_context.device).map_err(VulkanError::from)?;
        self.command_buffer.begin(&vulkan_context.device).map_err(VulkanError::from)?;
        let framebuffer = match self.framebuffers.get(present_index as usize) {
            Some(val) => val,
            None => return Err(crate::error::EngineError::Graphics("Framebuffer not found".to_string()))
        };
        self.command_buffer.begin_renderpass(&vulkan_context.device, &self.renderpass, framebuffer);
        self.command_buffer.end_renderpass(&vulkan_context.device);
        self.command_buffer.end(&vulkan_context.device).map_err(VulkanError::from)?;
        vulkan_context.graphics_queue.submit(&vulkan_context.device, &[&self.present_semaphore], &[&self.rendering_semaphore], &[&self.command_buffer], &self.command_buffer_reuse_fence).map_err(VulkanError::from)?;
        vulkan_context.swapchain.present(&vulkan_context.graphics_queue, &[&self.rendering_semaphore], present_index).map_err(VulkanError::from)?;
        Ok(())
    }
}
