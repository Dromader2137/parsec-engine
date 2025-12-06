use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};

use crate::graphics::vulkan::{VulkanError, device::Device};

#[derive(Clone)]
pub struct Semaphore {
    id: u32,
    device_id: u32,
    semaphore: ash::vk::Semaphore,
}

#[derive(Debug)]
pub enum SemaphoreError {
    CreationError(ash::vk::Result),
    WaitError(ash::vk::Result),
    DeviceMismatch,
}

impl From<SemaphoreError> for VulkanError {
    fn from(value: SemaphoreError) -> Self {
        VulkanError::SemaphoreError(value)
    }
}

impl Semaphore {
    const ID_COUNTER: AtomicU32 = AtomicU32::new(0);

    pub fn new(device: &Device) -> Result<Semaphore, SemaphoreError> {
        let create_info = ash::vk::SemaphoreCreateInfo::default();

        let semaphore = match unsafe {
            device.get_device_raw().create_semaphore(&create_info, None)
        } {
            Ok(val) => val,
            Err(err) => return Err(SemaphoreError::CreationError(err)),
        };

        let id = Self::ID_COUNTER.load(Ordering::Acquire);
        Self::ID_COUNTER.store(id + 1, Ordering::Release);

        Ok(Semaphore {
            id,
            device_id: device.id(),
            semaphore,
        })
    }

    pub fn null(device: &Device) -> Semaphore {
        let id = Self::ID_COUNTER.load(Ordering::Acquire);
        Self::ID_COUNTER.store(id + 1, Ordering::Release);

        Semaphore {
            id,
            device_id: device.id(),
            semaphore: ash::vk::Semaphore::null(),
        }
    }

    pub fn get_semaphore_raw(&self) -> &ash::vk::Semaphore { &self.semaphore }

    pub fn device_id(&self) -> u32 { self.device_id }
}
