use std::sync::atomic::{AtomicU32, Ordering};

use crate::graphics::vulkan::{VulkanError, device::VulkanDevice};

#[derive(Clone)]
pub struct VulkanSemaphore {
    id: u32,
    device_id: u32,
    semaphore: ash::vk::Semaphore,
}

#[derive(Debug)]
pub enum VulkanSemaphoreError {
    CreationError(ash::vk::Result),
    WaitError(ash::vk::Result),
    DeviceMismatch,
}

impl From<VulkanSemaphoreError> for VulkanError {
    fn from(value: VulkanSemaphoreError) -> Self {
        VulkanError::VulkanSemaphoreError(value)
    }
}

impl VulkanSemaphore {
    const ID_COUNTER: AtomicU32 = AtomicU32::new(0);

    pub fn new(
        device: &VulkanDevice,
    ) -> Result<VulkanSemaphore, VulkanSemaphoreError> {
        let create_info = ash::vk::SemaphoreCreateInfo::default();

        let semaphore = match unsafe {
            device.get_device_raw().create_semaphore(&create_info, None)
        } {
            Ok(val) => val,
            Err(err) => return Err(VulkanSemaphoreError::CreationError(err)),
        };

        let id = Self::ID_COUNTER.load(Ordering::Acquire);
        Self::ID_COUNTER.store(id + 1, Ordering::Release);

        Ok(VulkanSemaphore {
            id,
            device_id: device.id(),
            semaphore,
        })
    }

    pub fn null(device: &VulkanDevice) -> VulkanSemaphore {
        let id = Self::ID_COUNTER.load(Ordering::Acquire);
        Self::ID_COUNTER.store(id + 1, Ordering::Release);

        VulkanSemaphore {
            id,
            device_id: device.id(),
            semaphore: ash::vk::Semaphore::null(),
        }
    }

    pub fn get_semaphore_raw(&self) -> &ash::vk::Semaphore { &self.semaphore }

    pub fn device_id(&self) -> u32 { self.device_id }

    pub fn id(&self) -> u32 { self.id }
}
