use std::sync::Arc;

use crate::graphics::window::WindowWrapper;

#[derive(Debug)]
pub struct VulkanContext {
    instance: Arc<ash::Instance>,
    physical_device: Arc<ash::vk::PhysicalDevice>,
    device: Arc<ash::vk::Device>,
}

#[derive(Debug, Clone)]
pub enum VulkanContextError {
    LibraryLoadingError,
    InstanceError(String),
    PhysicalDeviceError(String),
    DeviceError(String),
    SurfaceError(String),
}

impl VulkanContext {
    pub fn new(
        event_loop: &winit::event_loop::ActiveEventLoop,
        window: &WindowWrapper,
    ) -> Result<VulkanContext, VulkanContextError> {
        let entry = match ash::Entry::load() {};

        let app_info = ash::vk::ApplicationInfo {
            api_version: ash::vk::make_api_version(0, 1, 0, 0),
            ..Default::default()
        };
        let create_info = ash::vk::InstanceCreateInfo {
            p_application_info: &app_info,
            ..Default::default()
        };
        let instance = unsafe { entry.create_instance(&create_info, None)? };
    }
}
