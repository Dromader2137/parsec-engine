use crate::graphics::window::WindowWrapper;

use super::{context::VulkanError, device::Device, instance::Instance, physical_device::PhysicalDevice, surface::Surface};

pub struct Swapchain {
    swapchain: ash::vk::SwapchainKHR,
    swapchain_loader: ash::khr::swapchain::Device,
}

#[derive(Debug)]
pub enum SwapchainError {
    PresentModesError(ash::vk::Result),
    CreationError(ash::vk::Result),
}

impl From<SwapchainError> for VulkanError {
    fn from(value: SwapchainError) -> Self {
        VulkanError::SwapchainError(value)
    }
}

impl Swapchain {
    pub fn new(instance: &Instance, surface: &Surface, physical_device: &PhysicalDevice, device: &Device, window: &WindowWrapper) -> Result<Swapchain, SwapchainError> {
        let mut desired_image_count = surface.min_image_count() + 1;
        if surface.max_image_count() > 0 && desired_image_count > surface.max_image_count() {
            desired_image_count = surface.max_image_count()
        }

        let surface_resolution = match surface.current_extent().width {
            u32::MAX => ash::vk::Extent2D {
                width: window.get_width(),
                height: window.get_height(),
            },
            _ => surface.current_extent(),
        };

        let pre_transform = if surface.supported_transforms().contains(ash::vk::SurfaceTransformFlagsKHR::IDENTITY) {
            ash::vk::SurfaceTransformFlagsKHR::IDENTITY
        } else {
            surface.current_transform()
        };

        let present_modes = match surface.get_present_modes(physical_device) {
            Ok(val) => val,
            Err(err) => return Err(SwapchainError::PresentModesError(err))
        };

        let present_mode = present_modes
            .iter()
            .cloned()
            .find(|&mode| mode == ash::vk::PresentModeKHR::MAILBOX)
            .unwrap_or(ash::vk::PresentModeKHR::FIFO);

        let swapchain_loader = ash::khr::swapchain::Device::new(instance.get_instance_raw(), device.get_device_raw());

        let swapchain_create_info = ash::vk::SwapchainCreateInfoKHR::default()
            .surface(*surface.get_surface_raw())
            .min_image_count(desired_image_count)
            .image_color_space(surface.color_space())
            .image_format(surface.format())
            .image_extent(surface_resolution)
            .image_usage(ash::vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(ash::vk::SharingMode::EXCLUSIVE)
            .pre_transform(pre_transform)
            .composite_alpha(ash::vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .image_array_layers(1);

        let swapchain = match unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None) } {
            Ok(val) => val,
            Err(err) => return Err(SwapchainError::CreationError(err))
        };

        Ok(Swapchain { swapchain, swapchain_loader })
    }
}
