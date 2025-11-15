use std::sync::Arc;

use super::{
  VulkanError,
  command_buffer::CommandPool,
  device::Device,
  instance::Instance,
  physical_device::PhysicalDevice,
  surface::{InitialSurface, Surface},
};
use crate::{graphics::window::WindowWrapper, resources::ResourceCollection};

pub fn init_vulkan(resources: &mut ResourceCollection) -> Result<(), VulkanError> {
  let window = resources.get::<Arc<WindowWrapper>>().unwrap();
  let instance = Instance::new(window.clone())?;
  let initial_surface = InitialSurface::new(instance.clone(), window.clone())?;
  drop(window);
  let physical_device = PhysicalDevice::new(instance.clone(), initial_surface.clone())?;
  let surface = Surface::from_initial_surface(initial_surface, physical_device.clone())?;
  let device = Device::new(physical_device.clone())?;
  let command_pool = CommandPool::new(device.clone())?;
  resources.add(instance).unwrap();
  resources.add(physical_device).unwrap();
  resources.add(surface).unwrap();
  resources.add(device).unwrap();
  resources.add(command_pool).unwrap();
  Ok(())
}
