use crate::graphics::window::WindowWrapper;

use super::{command_buffer::{CommandBufferError, CommandPool, CommandPoolError}, device::{Device, DeviceError}, fence::FenceError, framebuffer::FramebufferError, graphics_pipeline::GraphicsPipelineError, image::ImageError, instance::{Instance, InstanceError}, physical_device::{PhysicalDevice, PhysicalDeviceError}, queue::{Queue, QueueError}, renderpass::RenderpassError, semaphore::SemaphoreError, shader::ShaderError, surface::{InitialSurface, Surface, SurfaceError}, swapchain::SwapchainError};

pub struct VulkanContext {
    pub instance: Instance,
    pub surface: Surface,
    pub physical_device: PhysicalDevice,
    pub device: Device,
    pub graphics_queue: Queue,
    pub command_pool: CommandPool
}

#[derive(Debug)]
pub enum VulkanError {
    InstanceError(InstanceError),
    PhysicalDeviceError(PhysicalDeviceError),
    SurfaceError(SurfaceError),
    DeviceError(DeviceError),
    QueueError(QueueError),
    SwapchainError(SwapchainError),
    ImageError(ImageError),
    FramebufferError(FramebufferError),
    RenderpassError(RenderpassError),
    CommandBufferError(CommandBufferError),
    CommandPoolError(CommandPoolError),
    FenceError(FenceError),
    SemaphoreError(SemaphoreError),
    ShaderError(ShaderError),
    GrphicsPipelineError(GraphicsPipelineError),
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
}
