use std::sync::atomic::{AtomicU32, Ordering};

use crate::graphics::vulkan::{
    VulkanError, instance::Instance, physical_device::PhysicalDevice,
    surface::Surface,
};

pub struct Device {
    id: u32,
    physical_device_id: u32,
    surface_id: u32,
    device: ash::Device,
    memory_properties: ash::vk::PhysicalDeviceMemoryProperties
}

#[derive(Debug)]
pub enum DeviceError {
    DeviceCreationError(ash::vk::Result),
    WaitIdleError(ash::vk::Result),
    PhysicalDeviceMismatch,
}

impl From<DeviceError> for VulkanError {
    fn from(value: DeviceError) -> Self { VulkanError::DeviceError(value) }
}

impl Device {
    const ID_COUNTER: AtomicU32 = AtomicU32::new(0);

    pub fn new(
        instance: &Instance,
        physical_device: &PhysicalDevice,
        surface: &Surface,
    ) -> Result<Device, DeviceError> {
        if surface.physical_device_id() != physical_device.id() {
            return Err(DeviceError::PhysicalDeviceMismatch);
        }

        let device_extension_names_raw = [ash::khr::swapchain::NAME.as_ptr()];

        let features = ash::vk::PhysicalDeviceFeatures {
            shader_clip_distance: 1,
            ..Default::default()
        };

        let priorities = [1.0];

        let queue_info = ash::vk::DeviceQueueCreateInfo::default()
            .queue_family_index(physical_device.get_queue_family_index())
            .queue_priorities(&priorities);

        let device_create_info = ash::vk::DeviceCreateInfo::default()
            .queue_create_infos(std::slice::from_ref(&queue_info))
            .enabled_extension_names(&device_extension_names_raw)
            .enabled_features(&features);

        let device = unsafe {
            instance
                .get_instance_raw()
                .create_device(
                    *physical_device.get_physical_device_raw(),
                    &device_create_info,
                    None,
                )
                .map_err(|err| DeviceError::DeviceCreationError(err))?
        };

        let id = Self::ID_COUNTER.load(Ordering::Acquire);
        Self::ID_COUNTER.store(id + 1, Ordering::Release);

        Ok(Device {
            id,
            physical_device_id: physical_device.id(),
            surface_id: surface.id(),
            device,
            memory_properties: physical_device.physical_memory_properties()
        })
    }

    pub fn wait_idle(&self) -> Result<(), DeviceError> {
        if let Err(err) = unsafe { self.device.device_wait_idle() } {
            return Err(DeviceError::WaitIdleError(err));
        }
        Ok(())
    }

    pub fn get_device_raw(&self) -> &ash::Device { &self.device }

    pub fn id(&self) -> u32 { self.id }

    pub fn physical_device_id(&self) -> u32 { self.physical_device_id }

    pub fn surface_id(&self) -> u32 { self.surface_id }

    pub fn memory_properties(&self) -> ash::vk::PhysicalDeviceMemoryProperties {
        self.memory_properties
    }
}
