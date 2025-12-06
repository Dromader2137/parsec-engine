use std::sync::atomic::{AtomicU32, Ordering};

use ash::vk::Extent2D;

use crate::graphics::{
    vulkan::{
        VulkanError,
        device::Device,
        fence::Fence,
        image::{Image, SwapchainImage},
        instance::Instance,
        physical_device::PhysicalDevice,
        queue::Queue,
        semaphore::Semaphore,
        surface::Surface,
    },
    window::WindowWrapper,
};

pub struct Swapchain {
    id: u32,
    device_id: u32,
    swapchain_image_ids: Vec<u32>,
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
    InstanceMismatch,
    WindowMismatch,
    PhysicalDeviceMismatch,
    SurfaceMismatch,
    DeviceMismatch,
}

impl From<SwapchainError> for VulkanError {
    fn from(value: SwapchainError) -> Self {
        VulkanError::SwapchainError(value)
    }
}

impl Swapchain {
    const ID_COUNTER: AtomicU32 = AtomicU32::new(0);

    pub fn new(
        instance: &Instance,
        physical_device: &PhysicalDevice,
        window: &WindowWrapper,
        surface: &Surface,
        device: &Device,
        old_swapchain: Option<&Swapchain>,
    ) -> Result<(Swapchain, Vec<SwapchainImage>), SwapchainError> {
        if physical_device.instance_id() != instance.id() {
            return Err(SwapchainError::InstanceMismatch);
        }

        if surface.window_id() != window.id() {
            return Err(SwapchainError::WindowMismatch);
        }

        if device.physical_device_id() != physical_device.id() {
            return Err(SwapchainError::PhysicalDeviceMismatch);
        }

        if device.surface_id() != surface.id() {
            return Err(SwapchainError::SurfaceMismatch);
        }

        if let Some(swapchain) = old_swapchain {
            if swapchain.device_id() != device.id() {
                return Err(SwapchainError::DeviceMismatch);
            }
        }

        let mut desired_image_count = surface.min_image_count() + 1;
        if surface.max_image_count() > 0
            && desired_image_count > surface.max_image_count()
        {
            desired_image_count = surface.max_image_count()
        }

        let surface_resolution = {
            let size = window.size();
            Extent2D {
                width: size.0,
                height: size.1,
            }
        };

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

        let swapchain_loader = ash::khr::swapchain::Device::new(
            instance.get_instance_raw(),
            device.get_device_raw(),
        );

        let mut swapchain_create_info =
            ash::vk::SwapchainCreateInfoKHR::default()
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

        if let Some(val) = old_swapchain {
            swapchain_create_info =
                swapchain_create_info.old_swapchain(*val.get_swapchain_raw());
        }

        let swapchain = match unsafe {
            swapchain_loader.create_swapchain(&swapchain_create_info, None)
        } {
            Ok(val) => val,
            Err(err) => return Err(SwapchainError::CreationError(err)),
        };

        let swapchain_images = match unsafe {
            swapchain_loader.get_swapchain_images(swapchain)
        } {
            Ok(val) => val
                .into_iter()
                .map(|x| SwapchainImage::from_raw_image(device, x))
                .collect::<Vec<_>>(),
            Err(err) => return Err(SwapchainError::ImageAcquisitionError(err)),
        };

        let id = Self::ID_COUNTER.load(Ordering::Acquire);
        Self::ID_COUNTER.store(id + 1, Ordering::Release);

        Ok((
            Swapchain {
                id,
                device_id: device.id(),
                swapchain,
                swapchain_image_ids: swapchain_images
                    .iter()
                    .map(|x| x.id())
                    .collect(),
                swapchain_loader,
            },
            swapchain_images,
        ))
    }

    pub fn acquire_next_image(
        &self,
        semaphore: &Semaphore,
        fence: &Fence,
    ) -> Result<(u32, bool), SwapchainError> {
        if semaphore.device_id() != self.device_id
            || fence.device_id() != self.device_id
        {
            return Err(SwapchainError::DeviceMismatch);
        }

        match unsafe {
            self.swapchain_loader.acquire_next_image(
                self.swapchain,
                u64::MAX,
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
        if present_queue.device_id() != self.device_id {
            return Err(SwapchainError::DeviceMismatch);
        }

        for semaphore in wait_semaphores.iter() {
            if semaphore.device_id() != self.device_id {
                return Err(SwapchainError::DeviceMismatch);
            }
        }

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

    pub fn device_id(&self) -> u32 { self.device_id }

    pub fn id(&self) -> u32 { self.id }

    pub fn swapchain_image_ids(&self) -> &[u32] { &self.swapchain_image_ids }
}
