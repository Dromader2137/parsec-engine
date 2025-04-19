use super::{context::VulkanError, instance::Instance, physical_device::PhysicalDevice, queue::Queue};

pub struct Device {
    device: ash::Device,
}

#[derive(Debug)]
pub enum DeviceError {
    DeviceCreationError(ash::vk::Result),
}

impl From<DeviceError> for VulkanError {
    fn from(value: DeviceError) -> Self {
        VulkanError::DeviceError(value)
    }
}

impl Device {
    pub fn new(instance: &Instance, physical_device: &PhysicalDevice) -> Result<Device, DeviceError> {
        let device_extension_names_raw = [
            ash::khr::swapchain::NAME.as_ptr(),
        ];

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

        let device = match instance.create_device(physical_device, &device_create_info) {
            Ok(val) => val,
            Err(err) => return Err(DeviceError::DeviceCreationError(err))
        };

        Ok(Device { device })
    }

    pub fn get_present_queue(&self, family_index: u32) -> Queue {
        let raw_queue = unsafe { self.device.get_device_queue(family_index, 0) };
        Queue::new(raw_queue)
    }
}
