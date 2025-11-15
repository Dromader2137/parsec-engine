use std::sync::Arc;

use super::{
  VulkanError, device::Device, fence::Fence, image::SwapchainImage, queue::Queue,
  semaphore::Semaphore, surface::Surface,
};

pub struct Swapchain {
  pub device: Arc<Device>,
  pub surface: Arc<Surface>,
  pub swapchain_images: Vec<Arc<SwapchainImage>>,
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
    surface: Arc<Surface>,
    device: Arc<Device>,
    old_swapchain: Option<Arc<Swapchain>>,
  ) -> Result<Arc<Swapchain>, SwapchainError> {
    let mut desired_image_count = surface.min_image_count() + 1;
    if surface.max_image_count() > 0 && desired_image_count > surface.max_image_count() {
      desired_image_count = surface.max_image_count()
    }

    let surface_resolution = surface.current_extent();

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
          *device.physical_device.get_physical_device_raw(),
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
      device.physical_device.instance.get_instance_raw(),
      device.get_device_raw(),
    );

    let mut swapchain_create_info = ash::vk::SwapchainCreateInfoKHR::default()
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
      swapchain_create_info = swapchain_create_info.old_swapchain(*val.get_swapchain_raw());
    }

    let swapchain = match unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None) }
    {
      Ok(val) => val,
      Err(err) => return Err(SwapchainError::CreationError(err)),
    };

    let swapchain_images = match unsafe { swapchain_loader.get_swapchain_images(swapchain) } {
      Ok(val) => val
        .into_iter()
        .map(|x| SwapchainImage::from_raw_image(device.clone(), x))
        .collect::<Vec<_>>(),
      Err(err) => return Err(SwapchainError::ImageAcquisitionError(err)),
    };

    Ok(Arc::new(Swapchain {
      device,
      surface,
      swapchain_images,
      swapchain,
      swapchain_loader,
    }))
  }

  pub fn acquire_next_image(
    &self,
    semaphore: Arc<Semaphore>,
    fence: Arc<Fence>,
  ) -> Result<(u32, bool), SwapchainError> {
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
    present_queue: Arc<Queue>,
    wait_semaphores: &[Arc<Semaphore>],
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
      self
        .swapchain_loader
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
}

impl Drop for Swapchain {
  fn drop(&mut self) {
    unsafe {
      self
        .swapchain_loader
        .destroy_swapchain(self.swapchain, None)
    };
  }
}
