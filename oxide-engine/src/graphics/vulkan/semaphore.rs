use super::{context::VulkanError, device::Device};

pub struct Semaphore {
    semaphore: ash::vk::Semaphore
}

#[derive(Debug)]
pub enum SemaphoreError {
    CreationError(ash::vk::Result),
    WaitError(ash::vk::Result)
}

impl From<SemaphoreError> for VulkanError {
    fn from(value: SemaphoreError) -> Self {
        VulkanError::SemaphoreError(value)
    }
}

impl Semaphore {
    pub fn new(device: &Device) -> Result<Semaphore, SemaphoreError> {
        let create_info = ash::vk::SemaphoreCreateInfo::default();

        let semaphore = match unsafe { device.get_device_raw().create_semaphore(&create_info, None) } {
            Ok(val) => val,
            Err(err) => return Err(SemaphoreError::CreationError(err))
        };

        Ok( Semaphore { semaphore } )
    }
    
    pub fn null() -> Semaphore {
        Semaphore { semaphore: ash::vk::Semaphore::null() }
    }

    pub fn get_semaphore_raw(&self) -> &ash::vk::Semaphore {
        &self.semaphore
    }
}
