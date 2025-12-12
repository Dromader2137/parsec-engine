use crate::{
    ecs::system::system,
    graphics::{
        backend::GraphicsBackend, vulkan::
            VulkanBackend
        , window::Window
    },
    resources::{Resource, Resources},
};

#[system]
pub fn init_vulkan(window: Resource<Window>) {
    let context = VulkanBackend::init(&window);
    Resources::add(context).unwrap();
}
