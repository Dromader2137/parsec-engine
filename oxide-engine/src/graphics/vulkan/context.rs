use std::sync::Arc;

use crate::{
    ecs::system::system,
    graphics::{
        vulkan::{
            command_buffer::CommandPool,
            device::Device,
            instance::Instance,
            physical_device::PhysicalDevice,
            surface::{InitialSurface, Surface},
        },
        window::WindowWrapper,
    },
    resources::{Resource, Resources},
};

#[system]
pub fn init_vulkan(window: Resource<Arc<WindowWrapper>>) {
    let instance = Instance::new(window.clone()).unwrap();
    let initial_surface = InitialSurface::new(instance.clone(), window.clone()).unwrap();
    let physical_device = PhysicalDevice::new(instance.clone(), initial_surface.clone()).unwrap();
    let surface = Surface::from_initial_surface(initial_surface, physical_device.clone()).unwrap();
    let device = Device::new(physical_device.clone()).unwrap();
    let command_pool = CommandPool::new(device.clone()).unwrap();
    Resources::add(instance).unwrap();
    Resources::add(surface).unwrap();
    Resources::add(device).unwrap();
    Resources::add(physical_device).unwrap();
    Resources::add(command_pool).unwrap();
}
