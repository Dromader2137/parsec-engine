use crate::{
    graphics::vulkan::{
        instance::VulkanInstance, surface::VulkanInitialSurface,
    },
    utils::id_counter::IdCounter,
};

pub struct VulkanPhysicalDevice {
    id: u32,
    instance_id: u32,
    physical_device: ash::vk::PhysicalDevice,
    physical_memory_properties: ash::vk::PhysicalDeviceMemoryProperties,
    queue_family_index: u32,
}

#[derive(Debug)]
pub enum VulkanPhysicalDeviceError {
    CreationError(ash::vk::Result),
    SuitableDeviceNotFound,
}

static ID_COUNTER: once_cell::sync::Lazy<IdCounter> =
    once_cell::sync::Lazy::new(|| IdCounter::new(0));
impl VulkanPhysicalDevice {
    pub fn new(
        instance: &VulkanInstance,
        initial_surface: &VulkanInitialSurface,
    ) -> Result<VulkanPhysicalDevice, VulkanPhysicalDeviceError> {
        let physical_devices = unsafe {
            instance
                .get_instance_raw()
                .enumerate_physical_devices()
                .map_err(|err| VulkanPhysicalDeviceError::CreationError(err))?
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
            None => {
                return Err(VulkanPhysicalDeviceError::SuitableDeviceNotFound);
            },
        };

        let memory_prop = unsafe {
            instance
                .get_instance_raw()
                .get_physical_device_memory_properties(physical_device)
        };

        Ok(VulkanPhysicalDevice {
            id: ID_COUNTER.next(),
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
