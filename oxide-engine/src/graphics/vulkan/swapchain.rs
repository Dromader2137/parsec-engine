use ash::vk::Extent2D;

use crate::{
    graphics::{
        vulkan::{
            device::VulkanDevice,
            fence::VulkanFence,
            image::{VulkanImage, VulkanSwapchainImage},
            instance::VulkanInstance,
            physical_device::VulkanPhysicalDevice,
            queue::VulkanQueue,
            semaphore::VulkanSemaphore,
            surface::VulkanSurface,
        },
        window::Window,
    },
    utils::id_counter::IdCounter,
};

pub struct VulkanSwapchain {
    id: u32,
    device_id: u32,
    _swapchain_image_ids: Vec<u32>,
    swapchain: ash::vk::SwapchainKHR,
    swapchain_loader: ash::khr::swapchain::Device,
}

#[derive(Debug, thiserror::Error)]
pub enum VulkanSwapchainError {
    #[error("Failed to create Swapchain: {0}")]
    CreationError(ash::vk::Result),
    #[error("Failed to acquiere next image: {0}")]
    ImageAcquisitionError(ash::vk::Result),
    #[error("Failed to acquiere next image: {0}")]
    NextImageError(ash::vk::Result),
    #[error("Failed to present image: {0}")]
    PresentError(ash::vk::Result),
    #[error("Physical device created on another instance")]
    InstanceMismatch,
    #[error("Surface created for another window")]
    WindowMismatch,
    #[error("Device created on another physical device")]
    PhysicalDeviceMismatch,
    #[error("Device created for another surface")]
    SurfaceMismatch,
    #[error("Swapchaing created on another surface")]
    DeviceMismatch,
    #[error("Swapchain out of date")]
    OutOfDate,
}

static ID_COUNTER: once_cell::sync::Lazy<IdCounter> =
    once_cell::sync::Lazy::new(|| IdCounter::new(0));
impl VulkanSwapchain {
    pub fn new(
        instance: &VulkanInstance,
        physical_device: &VulkanPhysicalDevice,
        window: &Window,
        surface: &VulkanSurface,
        device: &VulkanDevice,
        old_swapchain: Option<&VulkanSwapchain>,
    ) -> Result<
        (VulkanSwapchain, Vec<VulkanSwapchainImage>),
        VulkanSwapchainError,
    > {
        if physical_device.instance_id() != instance.id() {
            return Err(VulkanSwapchainError::InstanceMismatch);
        }

        if surface.window_id() != window.id() {
            return Err(VulkanSwapchainError::WindowMismatch);
        }

        if device.physical_device_id() != physical_device.id() {
            return Err(VulkanSwapchainError::PhysicalDeviceMismatch);
        }

        if device.surface_id() != surface.id() {
            return Err(VulkanSwapchainError::SurfaceMismatch);
        }

        if let Some(swapchain) = old_swapchain {
            if swapchain.device_id() != device.id() {
                return Err(VulkanSwapchainError::DeviceMismatch);
            }
        }

        let mut desired_image_count = surface.max_image_count().min(3);
        if desired_image_count == 0 {
            desired_image_count = surface.min_image_count().min(3)
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

        let present_mode = ash::vk::PresentModeKHR::FIFO;

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
            Err(err) => return Err(VulkanSwapchainError::CreationError(err)),
        };

        let swapchain_images = match unsafe {
            swapchain_loader.get_swapchain_images(swapchain)
        } {
            Ok(val) => val
                .into_iter()
                .map(|x| {
                    VulkanSwapchainImage::from_raw_image(
                        device,
                        surface.format(),
                        x,
                    )
                })
                .collect::<Vec<_>>(),
            Err(err) => {
                return Err(VulkanSwapchainError::ImageAcquisitionError(err));
            },
        };

        Ok((
            VulkanSwapchain {
                id: ID_COUNTER.next(),
                device_id: device.id(),
                swapchain,
                _swapchain_image_ids: swapchain_images
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
        semaphore: &VulkanSemaphore,
        fence: &VulkanFence,
    ) -> Result<(u32, bool), VulkanSwapchainError> {
        if semaphore.device_id() != self.device_id
            || fence.device_id() != self.device_id
        {
            return Err(VulkanSwapchainError::DeviceMismatch);
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
            Err(err) => {
                if err == ash::vk::Result::ERROR_OUT_OF_DATE_KHR {
                    return Err(VulkanSwapchainError::OutOfDate);
                } else {
                    return Err(VulkanSwapchainError::NextImageError(err));
                }
            },
        }
    }

    pub fn present(
        &self,
        present_queue: &VulkanQueue,
        wait_semaphores: &[&VulkanSemaphore],
        image_index: u32,
    ) -> Result<(), VulkanSwapchainError> {
        if present_queue.device_id() != self.device_id {
            return Err(VulkanSwapchainError::DeviceMismatch);
        }

        for semaphore in wait_semaphores.iter() {
            if semaphore.device_id() != self.device_id {
                return Err(VulkanSwapchainError::DeviceMismatch);
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
            if err == ash::vk::Result::ERROR_OUT_OF_DATE_KHR {
                return Err(VulkanSwapchainError::OutOfDate);
            } else {
                return Err(VulkanSwapchainError::PresentError(err));
            }
        }

        Ok(())
    }

    pub fn get_swapchain_raw(&self) -> &ash::vk::SwapchainKHR {
        &self.swapchain
    }

    pub fn _get_swapchain_loader_raw(&self) -> &ash::khr::swapchain::Device {
        &self.swapchain_loader
    }

    pub fn device_id(&self) -> u32 { self.device_id }

    pub fn id(&self) -> u32 { self.id }

    pub fn _swapchain_image_ids(&self) -> &[u32] { &self._swapchain_image_ids }
}
