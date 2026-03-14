use crate::{
    arena::handle::{Handle, WeakHandle},
    graphics::vulkan::{
        VulkanBackend, device::VulkanDevice, instance::VulkanInstance, surface::VulkanInitialSurface
    },
};

pub struct VulkanPhysicalDevice {
    instance: Handle<VulkanInstance>,
    devices: Vec<Handle<VulkanDevice>>,
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

crate::create_counter! {ID_COUNTER}
impl VulkanPhysicalDevice {
    pub fn new(
        arenas: &mut VulkanBackend,
        instance_handle: Handle<VulkanInstance>,
        initial_surface: &VulkanInitialSurface,
    ) -> Result<WeakHandle<VulkanPhysicalDevice>, VulkanPhysicalDeviceError>
    {
        let instance = arenas.instances.get_mut(instance_handle.clone());

        let physical_devices = unsafe {
            instance
                .raw_instance()
                .enumerate_physical_devices()
                .map_err(|err| VulkanPhysicalDeviceError::CreationError(err))?
        };

        let (physical_device, queue_family_index) = match physical_devices
            .iter()
            .find_map(|p| {
                unsafe {
                    instance
                        .raw_instance()
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
                .raw_instance()
                .get_physical_device_memory_properties(physical_device)
        };

        let physical_device = VulkanPhysicalDevice {
            instance: instance_handle.clone(),
            physical_device,
            physical_memory_properties: memory_prop,
            queue_family_index,
        };

        let handle = arenas.physical_devices.add(physical_device);
        instance.physical_devices.push(handle.clone());
        Ok(handle.downgrade())
    }

    pub fn raw_physical_device(&self) -> &ash::vk::PhysicalDevice {
        &self.physical_device
    }

    pub fn queue_family_index(&self) -> u32 { self.queue_family_index }

    pub fn raw_physical_memory_properties(
        &self,
    ) -> ash::vk::PhysicalDeviceMemoryProperties {
        self.physical_memory_properties
    }

    pub fn instance(&self) -> Handle<VulkanInstance> { self.instance.clone() }
}
