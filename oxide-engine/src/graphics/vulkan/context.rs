use crate::graphics::window::WindowWrapper;

use super::{device::{Device, DeviceError}, instance::{Instance, InstanceError}, physical_device::{PhysicalDevice, PhysicalDeviceError}, queue::{Queue, QueueError}, surface::{InitialSurface, Surface, SurfaceError}, swapchain::SwapchainError};

pub struct VulkanContext {
    instance: Instance,
    surface: Surface,
    physical_device: PhysicalDevice,
    device: Device,
    graphics_queue: Queue,
}

#[derive(Debug)]
pub enum VulkanError {
    InstanceError(InstanceError),
    PhysicalDeviceError(PhysicalDeviceError),
    SurfaceError(SurfaceError),
    DeviceError(DeviceError),
    QueueError(QueueError),
    Swapchain(SwapchainError),
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

        Ok(VulkanContext { instance, surface, physical_device, device, graphics_queue })
    }
}
