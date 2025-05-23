use std::sync::Arc;

use crate::graphics::window::WindowWrapper;

use super::{
    VulkanError,
    command_buffer::CommandPool,
    device::Device,
    instance::Instance,
    physical_device::PhysicalDevice,
    queue::Queue,
    surface::{InitialSurface, Surface},
};

pub struct VulkanContext {
    pub window: Arc<WindowWrapper>,
    pub instance: Arc<Instance>,
    pub surface: Arc<Surface>,
    pub physical_device: Arc<PhysicalDevice>,
    pub device: Arc<Device>,
    pub graphics_queue: Arc<Queue>,
    pub command_pool: Arc<CommandPool>,
}

impl VulkanContext {
    pub fn new(window: Arc<WindowWrapper>) -> Result<Arc<VulkanContext>, VulkanError> {
        let instance = Instance::new(window.clone())?;
        let initial_surface = InitialSurface::new(instance.clone(), window.clone())?;
        let physical_device = PhysicalDevice::new(instance.clone(), initial_surface.clone())?;
        let surface = Surface::from_initial_surface(initial_surface, physical_device.clone())?;
        let device = Device::new(physical_device.clone())?;
        let graphics_queue =
            Queue::present(device.clone(), physical_device.get_queue_family_index());
        let command_pool = CommandPool::new(device.clone())?;

        Ok(Arc::new(VulkanContext {
            window,
            instance,
            surface,
            physical_device,
            device,
            graphics_queue,
            command_pool,
        }))
    }
}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        self.device.wait_idle().unwrap_or(());
    }
}
