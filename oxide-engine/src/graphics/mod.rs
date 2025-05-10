use vulkan::{VulkanError, context::VulkanContext, renderer::VulkanRenderer};
use window::{WindowError, WindowWrapper};

pub mod mesh_buffer;
pub mod vulkan;
pub mod window;

pub struct Graphics {
    pub window: WindowWrapper,
    pub vulkan_context: VulkanContext,
    pub renderer: VulkanRenderer,
}

#[derive(Debug)]
pub enum GraphicsError {
    WindowError(WindowError),
    VulkanError(VulkanError),
}

impl Graphics {
    pub fn new(event_loop: &winit::event_loop::ActiveEventLoop) -> Result<Graphics, GraphicsError> {
        let window = WindowWrapper::new(event_loop)?;
        let vulkan_context = VulkanContext::new(&window)?;
        let renderer = VulkanRenderer::new(&vulkan_context, &window)?;

        Ok(Graphics {
            window,
            vulkan_context,
            renderer,
        })
    }
}
