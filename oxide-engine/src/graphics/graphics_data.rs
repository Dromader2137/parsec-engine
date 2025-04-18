use crate::error::EngineError;

use super::{
    renderer::Renderer,
    vulkan::{context::VulkanContext, renderer::VulkanRenderer},
    window::WindowWrapper,
};

pub struct GraphicsData {
    pub window: WindowWrapper,
    pub vulkan_context: VulkanContext,
    pub renderer: Box<dyn Renderer>,
}

impl GraphicsData {
    pub fn new(
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) -> Result<GraphicsData, EngineError> {
        let window = match WindowWrapper::new(event_loop) {
            Ok(val) => val,
            Err(err) => return Err(EngineError::Graphics(format!("{:?}", err))),
        };

        let vulkan_context = match VulkanContext::new(event_loop, &window) {
            Ok(val) => val,
            Err(err) => return Err(EngineError::Graphics(format!("{:?}", err))),
        };

        let renderer = match VulkanRenderer::new(&vulkan_context, &window) {
            Ok(val) => Box::new(val),
            Err(err) => return Err(EngineError::Graphics(format!("{:?}", err))),
        };

        Ok(GraphicsData {
            window,
            vulkan_context,
            renderer,
        })
    }
}
