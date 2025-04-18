use crate::graphics::window::WindowWrapper;

use super::instance::{Instance, InstanceError};

pub struct VulkanContext {
    instance: Instance,
}

#[derive(Debug)]
pub enum VulkanContextError {
    InstanceError(InstanceError),
}

impl VulkanContext {
    pub fn new(
        event_loop: &winit::event_loop::ActiveEventLoop,
        window: &WindowWrapper,
    ) -> Result<VulkanContext, VulkanContextError> {
        let instance = Instance::new()?;

        Ok(VulkanContext { instance })
    }
}
