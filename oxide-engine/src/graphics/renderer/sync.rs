use std::sync::Arc;

use crate::graphics::vulkan::{
    VulkanError, device::Device, fence::Fence, semaphore::Semaphore,
};

pub struct VulkanRendererFrameSync {
    pub command_buffer_fence: Fence,
    pub image_available_semaphore: Semaphore,
}

pub struct VulkanRendererImageSync {
    pub rendering_complete_semaphore: Semaphore,
}

impl VulkanRendererFrameSync {
    pub fn new(
        device: &Device,
    ) -> Result<VulkanRendererFrameSync, VulkanError> {
        Ok(VulkanRendererFrameSync {
            command_buffer_fence: Fence::new(device, true)?,
            image_available_semaphore: Semaphore::new(device)?,
        })
    }
}

impl VulkanRendererImageSync {
    pub fn new(
        device: &Device,
    ) -> Result<VulkanRendererImageSync, VulkanError> {
        Ok(VulkanRendererImageSync {
            rendering_complete_semaphore: Semaphore::new(device)?,
        })
    }
}
