use super::{context::VulkanError, device::Device};

pub struct Swapchain {
    swapchain: ash::vk::SwapchainKHR,
    swapchain_loader: ash::khr::swapchain::Instance,
}

#[derive(Debug)]
pub enum SwapchainError {
}

impl From<SwapchainError> for VulkanError {
    fn from(value: SwapchainError) -> Self {
        VulkanError::Swapchain(value)
    }
}

impl SwapchainError {
    pub fn new(device: &Device, )
}
