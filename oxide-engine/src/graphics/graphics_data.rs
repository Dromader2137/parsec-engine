use super::{
    vulkan::{context::VulkanContext, renderer::VulkanRenderer, VulkanError},
    window::{WindowError, WindowWrapper},
};

pub struct GraphicsData {
    pub window: WindowWrapper,
    pub vulkan_context: VulkanContext,
    pub renderer: VulkanRenderer,
}

#[derive(Debug)]
pub enum GraphicsError {
    WindowError(WindowError),
    VulkanError(VulkanError)
}

impl GraphicsData {
    pub fn new(
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) -> Result<GraphicsData, GraphicsError> {
        let window = WindowWrapper::new(event_loop)?; 
        let vulkan_context = VulkanContext::new(&window)?;
        let renderer = VulkanRenderer::new(&vulkan_context, &window)?;

        Ok(GraphicsData {
            window,
            vulkan_context,
            renderer,
        })
    }
}
