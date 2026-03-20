use crate::graphics::vulkan::{
    instance::VulkanInstance, physical_device::VulkanPhysicalDevice,
};

pub struct VulkanDevice {
    device: ash::Device,
    memory_properties: ash::vk::PhysicalDeviceMemoryProperties,
}

#[derive(Debug, thiserror::Error)]
pub enum VulkanDeviceError {
    #[error("Failed to create device: {0}")]
    DeviceCreationError(ash::vk::Result),
    #[error("Failed to wait for device to be idle: {0}")]
    WaitIdleError(ash::vk::Result),
}

impl VulkanDevice {
    pub fn new(
        instance: &VulkanInstance,
        physical_device: &VulkanPhysicalDevice,
    ) -> Result<VulkanDevice, VulkanDeviceError> {
        let device_extension_names_raw = [ash::khr::swapchain::NAME.as_ptr()];

        let features = ash::vk::PhysicalDeviceFeatures {
            shader_clip_distance: 1,
            ..Default::default()
        };

        let priorities = [1.0];

        let queue_info = ash::vk::DeviceQueueCreateInfo::default()
            .queue_family_index(physical_device.queue_family_index())
            .queue_priorities(&priorities);

        let device_create_info = ash::vk::DeviceCreateInfo::default()
            .queue_create_infos(std::slice::from_ref(&queue_info))
            .enabled_extension_names(&device_extension_names_raw)
            .enabled_features(&features);

        let device = unsafe {
            instance
                .raw_handle()
                .create_device(
                    *physical_device.raw_handle(),
                    &device_create_info,
                    None,
                )
                .map_err(|err| VulkanDeviceError::DeviceCreationError(err))?
        };

        Ok(VulkanDevice {
            device,
            memory_properties: physical_device.raw_physical_memory_properties(),
        })
    }

    pub fn wait_idle(&self) -> Result<(), VulkanDeviceError> {
        if let Err(err) = unsafe { self.device.device_wait_idle() } {
            return Err(VulkanDeviceError::WaitIdleError(err));
        }
        Ok(())
    }

    pub fn destroy(&self) { unsafe { self.device.destroy_device(None) } }

    pub fn raw_device(&self) -> &ash::Device { &self.device }

    pub fn raw_memory_properties(
        &self,
    ) -> ash::vk::PhysicalDeviceMemoryProperties {
        self.memory_properties
    }
}
