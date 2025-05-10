use crate::graphics::window::WindowWrapper;

use super::{
    VulkanError, device::Device, fence::Fence, image::Image, instance::Instance,
    physical_device::PhysicalDevice, queue::Queue, semaphore::Semaphore, surface::Surface,
};

pub struct Swapchain {
    swapchain: ash::vk::SwapchainKHR,
    swapchain_loader: ash::khr::swapchain::Device,
}

#[derive(Debug)]
pub enum SwapchainError {
    PresentModesError(ash::vk::Result),
    CreationError(ash::vk::Result),
    ImageAcquisitionError(ash::vk::Result),
    NextImageError(ash::vk::Result),
    PresentError(ash::vk::Result),
}

impl From<SwapchainError> for VulkanError {
    fn from(value: SwapchainError) -> Self {
        VulkanError::SwapchainError(value)
    }
}

impl Swapchain {
    pub fn new(
        instance: &Instance,
        surface: &Surface,
        physical_device: &PhysicalDevice,
        device: &Device,
        window: &WindowWrapper,
    ) -> Result<Swapchain, SwapchainError> {
        let mut desired_image_count = surface.min_image_count() + 1;
        if surface.max_image_count() > 0 && desired_image_count > surface.max_image_count() {
            desired_image_count = surface.max_image_count()
        }

        let surface_resolution = surface.current_extent(window);

        let pre_transform = if surface
            .supported_transforms()
            .contains(ash::vk::SurfaceTransformFlagsKHR::IDENTITY)
        {
            ash::vk::SurfaceTransformFlagsKHR::IDENTITY
        } else {
            surface.current_transform()
        };

        let present_modes = match unsafe {
            surface
                .get_surface_loader_raw()
                .get_physical_device_surface_present_modes(
                    *physical_device.get_physical_device_raw(),
                    *surface.get_surface_raw(),
                )
        } {
            Ok(val) => val,
            Err(err) => return Err(SwapchainError::PresentModesError(err)),
        };

        let present_mode = present_modes
            .iter()
            .cloned()
            .find(|&mode| mode == ash::vk::PresentModeKHR::MAILBOX)
            .unwrap_or(ash::vk::PresentModeKHR::FIFO);

        let swapchain_loader =
            ash::khr::swapchain::Device::new(instance.get_instance_raw(), device.get_device_raw());

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

        let swapchain =
            match unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None) } {
                Ok(val) => val,
                Err(err) => return Err(SwapchainError::CreationError(err)),
            };

        Ok(Swapchain {
            swapchain,
            swapchain_loader,
        })
    }

    pub fn get_images(&self) -> Result<Vec<Image>, SwapchainError> {
        match unsafe { self.swapchain_loader.get_swapchain_images(self.swapchain) } {
            Ok(val) => Ok(val.into_iter().map(|x| Image::from_raw_image(x)).collect()),
            Err(err) => Err(SwapchainError::ImageAcquisitionError(err)),
        }
    }

    pub fn acquire_next_image(
        &self,
        semaphore: &Semaphore,
        fence: &Fence,
    ) -> Result<(u32, bool), SwapchainError> {
        match unsafe {
            self.swapchain_loader.acquire_next_image(
                self.swapchain,
                10000000,
                *semaphore.get_semaphore_raw(),
                *fence.get_fence_raw(),
            )
        } {
            Ok(val) => Ok(val),
            Err(err) => Err(SwapchainError::NextImageError(err)),
        }
    }

    pub fn present(
        &self,
        present_queue: &Queue,
        wait_semaphores: &[&Semaphore],
        image_index: u32,
    ) -> Result<(), SwapchainError> {
        let wait_semaphores = wait_semaphores
            .iter()
            .map(|x| *x.get_semaphore_raw())
            .collect::<Vec<_>>();
        let swapchains = [self.swapchain];
        let image_indices = [image_index];

        let present_info = ash::vk::PresentInfoKHR::default()
            .wait_semaphores(&wait_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        if let Err(err) = unsafe {
            self.swapchain_loader
                .queue_present(*present_queue.get_queue_raw(), &present_info)
        } {
            return Err(SwapchainError::PresentError(err));
        }

        Ok(())
    }

    pub fn get_swapchain_raw(&self) -> &ash::vk::SwapchainKHR {
        &self.swapchain
    }

    pub fn get_swapchain_loader_raw(&self) -> &ash::khr::swapchain::Device {
        &self.swapchain_loader
    }

    pub fn cleanup(&self) {
        unsafe {
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None)
        };
    }
}
