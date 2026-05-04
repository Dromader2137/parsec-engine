use parsec_engine_utils::create_counter;

use crate::device::VulkanDevice;

#[derive(Clone)]
pub struct VulkanSemaphore {
    id: u32,
    semaphore: ash::vk::Semaphore,
}

#[derive(Debug, thiserror::Error)]
pub enum VulkanSemaphoreError {
    #[error("Failed to create semaphore: {0}")]
    CreationError(ash::vk::Result),
}

create_counter! {ID_COUNTER}
impl VulkanSemaphore {
    pub fn new(
        device: &VulkanDevice,
    ) -> Result<VulkanSemaphore, VulkanSemaphoreError> {
        let create_info = ash::vk::SemaphoreCreateInfo::default();

        let semaphore = match unsafe {
            device.raw_device().create_semaphore(&create_info, None)
        } {
            Ok(val) => val,
            Err(err) => return Err(VulkanSemaphoreError::CreationError(err)),
        };

        Ok(VulkanSemaphore {
            id: ID_COUNTER.next(),
            semaphore,
        })
    }

    pub fn _null() -> VulkanSemaphore {
        VulkanSemaphore {
            id: ID_COUNTER.next(),
            semaphore: ash::vk::Semaphore::null(),
        }
    }

    pub fn destroy(self, device: &VulkanDevice) {
        unsafe {
            device
                .raw_device()
                .destroy_semaphore(*self.get_semaphore_raw(), None)
        }
    }

    pub fn get_semaphore_raw(&self) -> &ash::vk::Semaphore { &self.semaphore }

    pub fn id(&self) -> u32 { self.id }
}
