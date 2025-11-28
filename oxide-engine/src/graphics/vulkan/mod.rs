//! Module responsible for interaction with the Vulkan API. (Incomplete, undocumented and subject
//! to change).

use buffer::BufferError;
use command_buffer::{CommandBufferError, CommandPoolError};
use descriptor_set::DescriptorError;
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

use crate::graphics::{GraphicsError, renderer::RendererError};

pub mod buffer;
pub mod command_buffer;
pub mod context;
pub mod descriptor_set;
pub mod device;
pub mod fence;
pub mod format_size;
pub mod framebuffer;
pub mod graphics_pipeline;
pub mod image;
pub mod instance;
pub mod physical_device;
pub mod queue;
pub mod renderpass;
pub mod semaphore;
pub mod shader;
pub mod surface;
pub mod swapchain;

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
    DescriptorError(DescriptorError),
    RendererError(RendererError),
}

impl From<VulkanError> for GraphicsError {
    fn from(value: VulkanError) -> Self { GraphicsError::VulkanError(value) }
}
