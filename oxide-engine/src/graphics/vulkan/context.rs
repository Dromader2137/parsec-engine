use std::sync::Arc;

use crate::{
    graphics::{
        vulkan::{
            VulkanError,
            command_buffer::CommandPool,
            device::Device,
            instance::Instance,
            physical_device::PhysicalDevice,
            surface::{InitialSurface, Surface},
        },
        window::WindowWrapper,
    }, resources::Rsc,
};

pub fn init_vulkan() -> Result<(), VulkanError> {
    let window = Rsc::<Arc<WindowWrapper>>::get().unwrap();
    let instance = Instance::new(window.clone())?;
    let initial_surface = InitialSurface::new(instance.clone(), window.clone())?;
    drop(window);
    let physical_device = PhysicalDevice::new(instance.clone(), initial_surface.clone())?;
    let surface = Surface::from_initial_surface(initial_surface, physical_device.clone())?;
    let device = Device::new(physical_device.clone())?;
    let command_pool = CommandPool::new(device.clone())?;
    Rsc::add(instance).unwrap();
    Rsc::add(physical_device).unwrap();
    Rsc::add(surface).unwrap();
    Rsc::add(device).unwrap();
    Rsc::add(command_pool).unwrap();
    Ok(())
}
