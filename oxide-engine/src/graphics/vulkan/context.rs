use crate::graphics::window::WindowWrapper;

use super::{command_buffer::CommandPool, device::Device, instance::Instance, physical_device::PhysicalDevice, queue::Queue, surface::{InitialSurface, Surface}, VulkanError};

pub struct VulkanContext {
    pub instance: Instance,
    pub surface: Surface,
    pub physical_device: PhysicalDevice,
    pub device: Device,
    pub graphics_queue: Queue,
    pub command_pool: CommandPool
}


impl VulkanContext {
    pub fn new(
        window: &WindowWrapper,
    ) -> Result<VulkanContext, VulkanError> {
        let instance = Instance::new(window)?;
        let initial_surface = InitialSurface::new(&instance, window)?;
        let physical_device = PhysicalDevice::new(&instance, &initial_surface)?;
        let surface = initial_surface.into_surface(&physical_device)?;
        let device = Device::new(&instance, &physical_device)?;
        let graphics_queue = device.get_present_queue(physical_device.get_queue_family_index());
        let command_pool = CommandPool::new(&physical_device, &device)?;

        Ok(VulkanContext { instance, surface, physical_device, device, graphics_queue, command_pool })
    }

    pub fn cleanup(&self) -> Result<(), VulkanError> {
        self.device.wait_idle()?;
        self.command_pool.cleanup(&self.device);
        self.device.cleanup();
        self.surface.cleanup();
        self.instance.cleanup();
        Ok(())
    }
}
