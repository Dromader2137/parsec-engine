use std::sync::atomic::{AtomicU32, Ordering};

use crate::graphics::vulkan::{
    VulkanError, instance::Instance, surface::InitialSurface,
};

pub struct PhysicalDevice {
    id: u32,
    instance_id: u32,
    physical_device: ash::vk::PhysicalDevice,
    physical_memory_properties: ash::vk::PhysicalDeviceMemoryProperties,
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
    const ID_COUNTER: AtomicU32 = AtomicU32::new(0);

    pub fn new(
        instance: &Instance,
        initial_surface: &InitialSurface,
    ) -> Result<PhysicalDevice, PhysicalDeviceError> {
        let physical_devices = unsafe {
            instance
                .get_instance_raw()
                .enumerate_physical_devices()
                .map_err(|err| PhysicalDeviceError::CreationError(err))?
        };

        let (physical_device, queue_family_index) = match physical_devices
            .iter()
            .find_map(|p| {
                unsafe {
                    instance
                        .get_instance_raw()
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
            None => return Err(PhysicalDeviceError::SuitableDeviceNotFound),
        };

        let memory_prop = unsafe {
            instance
                .get_instance_raw()
                .get_physical_device_memory_properties(physical_device)
        };

        let id = Self::ID_COUNTER.load(Ordering::Acquire);
        Self::ID_COUNTER.store(id + 1, Ordering::Release);

        Ok(PhysicalDevice {
            id,
            instance_id: instance.id(),
            physical_device,
            physical_memory_properties: memory_prop,
            queue_family_index,
        })
    }

    pub fn get_physical_device_raw(&self) -> &ash::vk::PhysicalDevice {
        &self.physical_device
    }

    pub fn get_queue_family_index(&self) -> u32 { self.queue_family_index }

    pub fn id(&self) -> u32 { self.id }

    pub fn physical_memory_properties(
        &self,
    ) -> ash::vk::PhysicalDeviceMemoryProperties {
        self.physical_memory_properties
    }

    pub fn instance_id(&self) -> u32 { self.instance_id }
}
