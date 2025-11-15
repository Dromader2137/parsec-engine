use std::sync::Arc;

use super::{VulkanError, device::Device};

#[derive(Clone)]
pub struct Semaphore {
    pub device: Arc<Device>,
    semaphore: ash::vk::Semaphore,
}

#[derive(Debug)]
pub enum SemaphoreError {
    CreationError(ash::vk::Result),
    WaitError(ash::vk::Result),
}

impl From<SemaphoreError> for VulkanError {
    fn from(value: SemaphoreError) -> Self {
        VulkanError::SemaphoreError(value)
    }
}

impl Semaphore {
    pub fn new(device: Arc<Device>) -> Result<Arc<Semaphore>, SemaphoreError> {
        let create_info = ash::vk::SemaphoreCreateInfo::default();

        let semaphore =
            match unsafe { device.get_device_raw().create_semaphore(&create_info, None) } {
                Ok(val) => val,
                Err(err) => return Err(SemaphoreError::CreationError(err)),
            };

        Ok(Arc::new(Semaphore { device, semaphore }))
    }

    pub fn null(device: Arc<Device>) -> Semaphore {
        Semaphore {
            device,
            semaphore: ash::vk::Semaphore::null(),
        }
    }

    pub fn get_semaphore_raw(&self) -> &ash::vk::Semaphore {
        &self.semaphore
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        unsafe {
            self.device
                .get_device_raw()
                .destroy_semaphore(self.semaphore, None)
        };
    }
}
