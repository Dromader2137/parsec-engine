use std::sync::Arc;

use vulkan::{VulkanError, context::VulkanContext, renderer::VulkanRenderer};
use window::{WindowError, WindowWrapper};

use crate::error::EngineError;

pub mod mesh;
pub mod vulkan;
pub mod window;

pub struct Graphics {
    pub window: Option<Arc<WindowWrapper>>,
    pub vulkan_context: Option<Arc<VulkanContext>>,
    pub renderer: Option<VulkanRenderer>,
}

#[derive(Debug)]
pub enum GraphicsError {
    WindowError(WindowError),
    VulkanError(VulkanError),
    Uninitialized,
}

impl From<GraphicsError> for EngineError {
    fn from(value: GraphicsError) -> Self {
        EngineError::GraphicsError(value)
    }
}

impl Graphics {
    pub fn new() -> Graphics {
        Graphics {
            window: None,
            vulkan_context: None,
            renderer: None,
        }
    }

    pub fn init(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_name: &str,
    ) -> Result<(), GraphicsError> {
        self.window = Some(WindowWrapper::new(event_loop, window_name)?);
        self.vulkan_context = Some(VulkanContext::new(self.window.as_ref().unwrap())?);
        self.renderer = Some(VulkanRenderer::new(
            self.vulkan_context.as_ref().unwrap().clone(),
            self.window.as_ref().unwrap().clone(),
        )?);

        Ok(())
    }

    pub fn render(&mut self) -> Result<(), GraphicsError> {
        match self.renderer.as_mut() {
            Some(val) => val.render(self.window.as_ref().unwrap().clone())?,
            None => return Err(GraphicsError::Uninitialized),
        };

        Ok(())
    }

    pub fn resize(&mut self) -> Result<(), GraphicsError> {
        match self.renderer.as_mut() {
            Some(val) => val.handle_resize()?,
            None => return Err(GraphicsError::Uninitialized),
        };

        Ok(())
    }

    pub fn request_redraw(&mut self) -> Result<(), GraphicsError> {
        match self.window.as_mut() {
            Some(val) => val.request_redraw(),
            None => return Err(GraphicsError::Uninitialized),
        };

        Ok(())
    }
}
