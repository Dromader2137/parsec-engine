use std::sync::Arc;

use vulkan::{VulkanError, context::VulkanContext};
use window::{WindowError, WindowWrapper};

use crate::{
    app::ActiveEventLoopStore, ecs::system::{System, SystemBundle, SystemInput, SystemTrigger}, error::EngineError, graphics::renderer::VulkanRenderer
};

pub mod camera;
pub mod mesh;
pub mod renderer;
pub mod vulkan;
pub mod window;
pub mod transform;

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


#[derive(Default)]
pub struct GraphicsBundle {}
impl SystemBundle for GraphicsBundle {
    fn systems(self) -> Vec<System> {
        vec![
            System::new(SystemTrigger::LateStart, |SystemInput { resources, .. }| {
                let window = { 
                    let event_loop = resources.get::<ActiveEventLoopStore>().unwrap();
                    let event_loop_raw = event_loop.get_event_loop();
                    WindowWrapper::new(event_loop_raw, "Oxide Engine test").unwrap()
                };
                let context = VulkanContext::new(window.clone()).unwrap();
                let renderer = VulkanRenderer::new(context.clone()).unwrap();
                resources.add(window).unwrap();
                resources.add(context).unwrap();
                resources.add(renderer).unwrap();
            }),
            System::new(SystemTrigger::Render, |SystemInput { resources, .. }| {
                let mut renderer = resources.get_mut::<VulkanRenderer>().unwrap();
                renderer.render().unwrap();
            }),
            System::new(SystemTrigger::Update, |SystemInput { resources, .. }| {
                let window = resources.get_mut::<Arc<WindowWrapper>>().unwrap();
                window.request_redraw();
            }),
        ]
    }
}
