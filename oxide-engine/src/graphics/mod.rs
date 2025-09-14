use std::sync::Arc;

use vulkan::{VulkanError, context::VulkanContext};
use window::{WindowError, WindowWrapper};

use crate::{
    error::EngineError,
    graphics::{
        renderer::{DefaultVertex, VulkanRenderer},
        vulkan::{descriptor_set::DescriptorSetBinding, shader::ShaderType},
    },
};

pub mod camera;
pub mod mesh;
pub mod renderer;
pub mod vulkan;
pub mod window;

pub struct Graphics {
    window: Option<Arc<WindowWrapper>>,
    vulkan_context: Option<Arc<VulkanContext>>,
    renderer: Option<VulkanRenderer>,
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
        self.vulkan_context = Some(VulkanContext::new(self.window.as_ref().unwrap().clone())?);
        self.renderer = Some(VulkanRenderer::new(self.vulkan_context.as_ref().unwrap().clone())?);

        Ok(())
    }

    pub fn renderer(&self) -> Result<&VulkanRenderer, GraphicsError> {
        self.renderer.as_ref().map_or(Err(GraphicsError::Uninitialized), |x| Ok(x))
    }
    
    pub fn renderer_mut(&mut self) -> Result<&mut VulkanRenderer, GraphicsError> {
        self.renderer.as_mut().map_or(Err(GraphicsError::Uninitialized), |x| Ok(x))
    }
    
    pub fn get_screen_aspect_ratio(&self) -> Result<f32, GraphicsError> {
        self.renderer
            .as_ref()
            .map_or(Err(GraphicsError::Uninitialized), |renderer| {
                Ok(renderer.get_aspect_ratio())
            })
    }

    pub fn render(&mut self) -> Result<(), GraphicsError> {
        match self.renderer.as_mut() {
            Some(val) => val.render()?,
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

    pub fn add_shader(&mut self, name: &str, code: &[u32], shader_type: ShaderType) -> Result<(), GraphicsError> {
        match self.renderer.as_mut() {
            Some(renderer) => {
                renderer.create_shader(name, code, shader_type)?;
            }
            None => return Err(GraphicsError::Uninitialized),
        };

        Ok(())
    }

    pub fn add_material_base(
        &mut self,
        name: &str,
        vertex_name: &str,
        fragment_name: &str,
        layout: Vec<Vec<DescriptorSetBinding>>,
    ) -> Result<(), GraphicsError> {
        match self.renderer.as_mut() {
            Some(renderer) => {
                renderer.create_material_base(name, vertex_name, fragment_name, layout)?;
            }
            None => return Err(GraphicsError::Uninitialized),
        };

        Ok(())
    }

    pub fn add_buffer<T: Clone + Copy>(&mut self, name: &str, data: Vec<T>) -> Result<(), GraphicsError> {
        match self.renderer.as_mut() {
            Some(renderer) => {
                renderer.create_buffer(name, data)?;
            }
            None => return Err(GraphicsError::Uninitialized),
        };

        Ok(())
    }

    pub fn add_material(
        &mut self,
        name: &str,
        base_name: &str,
        buffer_names: Vec<Vec<&str>>,
    ) -> Result<(), GraphicsError> {
        match self.renderer.as_mut() {
            Some(renderer) => {
                renderer.create_material(name, base_name, buffer_names)?;
            }
            None => return Err(GraphicsError::Uninitialized),
        };

        Ok(())
    }

    pub fn add_mesh(
        &mut self,
        mesh_name: &str,
        vertices: Vec<DefaultVertex>,
        indices: Vec<u32>,
    ) -> Result<(), GraphicsError> {
        match self.renderer.as_mut() {
            Some(renderer) => {
                renderer.create_mesh(mesh_name, vertices, indices)?;
            }
            None => return Err(GraphicsError::Uninitialized),
        };

        Ok(())
    }
}
