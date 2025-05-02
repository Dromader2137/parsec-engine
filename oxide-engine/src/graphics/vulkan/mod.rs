use buffer::BufferError;
use command_buffer::{CommandBufferError, CommandPoolError};
use device::DeviceError;
use fence::FenceError;
use framebuffer::FramebufferError;
use graphics_pipeline::GraphicsPipelineError;
use image::ImageError;
use instance::InstanceError;
use physical_device::PhysicalDeviceError;
use queue::QueueError;
use renderpass::RenderpassError;
use semaphore::SemaphoreError;
use shader::ShaderError;
use surface::SurfaceError;
use swapchain::SwapchainError;

use super::graphics_data::GraphicsError;

pub mod context;
pub mod instance;
pub mod physical_device;
pub mod device;
pub mod surface;
pub mod renderer;
pub mod queue;
pub mod swapchain;
pub mod image;
pub mod framebuffer;
pub mod command_buffer;
pub mod renderpass;
pub mod fence;
pub mod semaphore;
pub mod graphics_pipeline;
pub mod shader;
pub mod buffer;

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
    BufferError(BufferError),
}

impl From<VulkanError> for GraphicsError {
    fn from(value: VulkanError) -> Self {
        GraphicsError::VulkanError(value)
    }
}
