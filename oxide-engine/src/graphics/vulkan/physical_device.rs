use std::sync::Arc;

use crate::graphics::vulkan::{VulkanError, instance::Instance, surface::InitialSurface};

pub struct PhysicalDevice {
    pub instance: Arc<Instance>,
    physical_device: ash::vk::PhysicalDevice,
    queue_family_index: u32,
}

#[derive(Debug)]
pub enum PhysicalDeviceError {
    CreationError(ash::vk::Result),
    SuitableDeviceNotFound,
}

impl From<PhysicalDeviceError> for VulkanError {
    fn from(value: PhysicalDeviceError) -> Self {
        VulkanError::PhysicalDeviceError(value)
    }
}

impl PhysicalDevice {
    pub fn new(
        instance: Arc<Instance>,
        initial_surface: Arc<InitialSurface>,
    ) -> Result<Arc<PhysicalDevice>, PhysicalDeviceError> {
        let physical_devices =
            match unsafe { instance.get_instance_raw().enumerate_physical_devices() } {
                Ok(val) => val,
                Err(err) => return Err(PhysicalDeviceError::CreationError(err)),
            };

        let (physical_device, queue_family_index) = match physical_devices.iter().find_map(|p| {
            unsafe {
                instance
                    .get_instance_raw()
                    .get_physical_device_queue_family_properties(*p)
            }
            .iter()
            .enumerate()
            .find_map(|(index, info)| {
                let supports_graphic_and_surface =
                    info.queue_flags.contains(ash::vk::QueueFlags::GRAPHICS)
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
            None => return Err(PhysicalDeviceError::SuitableDeviceNotFound),
        };

        Ok(Arc::new(PhysicalDevice {
            instance,
            physical_device,
            queue_family_index,
        }))
    }

    pub fn get_physical_device_raw(&self) -> &ash::vk::PhysicalDevice {
        &self.physical_device
    }

    pub fn get_queue_family_index(&self) -> u32 {
        self.queue_family_index
    }
}
