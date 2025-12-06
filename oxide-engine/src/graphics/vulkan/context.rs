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
pub fn init_vulkan(window: Resource<WindowWrapper>) {
    let instance = Instance::new(&window).unwrap();
    let initial_surface = InitialSurface::new(&instance, &window).unwrap();
    let physical_device =
        PhysicalDevice::new(&instance, &initial_surface).unwrap();
    let surface =
        Surface::from_initial_surface(initial_surface, &physical_device)
            .unwrap();
    let device = Device::new(&instance, &physical_device, &surface).unwrap();
    let command_pool = CommandPool::new(&physical_device, &device).unwrap();
    Resources::add(instance).unwrap();
    Resources::add(surface).unwrap();
    Resources::add(device).unwrap();
    Resources::add(physical_device).unwrap();
    Resources::add(command_pool).unwrap();
}
