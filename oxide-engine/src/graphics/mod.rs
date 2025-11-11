use std::sync::Arc;

use vulkan::VulkanError;
use window::{WindowError, WindowWrapper};

use crate::{
    app::ActiveEventLoopStore, ecs::system::{System, SystemBundle, SystemInput, SystemTrigger}, error::EngineError, graphics::{renderer::{init_renderer, render}, vulkan::context::init_vulkan}
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
                resources.add(window).unwrap();
                init_vulkan(resources).unwrap();
                init_renderer(resources).unwrap();
            }),
            System::new(SystemTrigger::Render, |SystemInput { resources, .. }| {
                render(resources).unwrap();
            }),
            System::new(SystemTrigger::Update, |SystemInput { resources, .. }| {
                let window = resources.get_mut::<Arc<WindowWrapper>>().unwrap();
                window.request_redraw();
            }),
        ]
    }
}
