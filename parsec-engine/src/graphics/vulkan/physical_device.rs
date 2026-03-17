use crate::graphics::vulkan::{
    instance::VulkanInstance, surface::VulkanInitialSurface,
};

pub struct VulkanPhysicalDevice {
    physical_device: ash::vk::PhysicalDevice,
    physical_memory_properties: ash::vk::PhysicalDeviceMemoryProperties,
    queue_family_index: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum VulkanPhysicalDeviceError {
    #[error("Failed to create physical device: {0}")]
    CreationError(ash::vk::Result),
    #[error("Failed to find a suitable physical device")]
    SuitableDeviceNotFound,
}

impl VulkanPhysicalDevice {
    pub fn new(
        instance: &VulkanInstance,
        initial_surface: &VulkanInitialSurface,
    ) -> Result<VulkanPhysicalDevice, VulkanPhysicalDeviceError> {
        let physical_devices = unsafe {
            instance
                .raw_handle()
                .enumerate_physical_devices()
                .map_err(|err| VulkanPhysicalDeviceError::CreationError(err))?
        };

        let (physical_device, queue_family_index) = match physical_devices
            .iter()
            .find_map(|p| {
                unsafe {
                    instance
                        .raw_handle()
                        .get_physical_device_queue_family_properties(*p)
                }
                .iter()
                .enumerate()
                .find_map(|(index, info)| {
                    let supports_graphic_and_surface = info
                        .queue_flags
                        .contains(ash::vk::QueueFlags::GRAPHICS)
                        && initial_surface
                            .check_surface_support(*p, index as u32)
                            .unwrap_or(false);

                    if supports_graphic_and_surface {
                        Some((*p, index as u32))
                    } else {
                        None
                    }
                })
            }) {
            Some(val) => val,
            None => {
                return Err(VulkanPhysicalDeviceError::SuitableDeviceNotFound);
            },
        };

        let memory_prop = unsafe {
            instance
                .raw_handle()
                .get_physical_device_memory_properties(physical_device)
        };

        Ok(VulkanPhysicalDevice {
            physical_device,
            physical_memory_properties: memory_prop,
            queue_family_index,
        })
    }

    pub fn raw_handle(&self) -> &ash::vk::PhysicalDevice {
        &self.physical_device
    }

    pub fn queue_family_index(&self) -> u32 { self.queue_family_index }

    pub fn raw_physical_memory_properties(
        &self,
    ) -> ash::vk::PhysicalDeviceMemoryProperties {
        self.physical_memory_properties
    }
}
