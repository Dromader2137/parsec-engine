use crate::graphics::vulkan::device::VulkanDevice;

#[derive(Clone)]
pub struct VulkanSemaphore {
    id: u32,
    device_id: u32,
    semaphore: ash::vk::Semaphore,
}

#[derive(Debug, thiserror::Error)]
pub enum VulkanSemaphoreError {
    #[error("Failed to create semaphore: {0}")]
    CreationError(ash::vk::Result),
    #[error("Semaphore created on different device")]
    DeviceMismatch,
}

crate::create_counter! {ID_COUNTER}
impl VulkanSemaphore {
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

        Ok(VulkanSemaphore {
            id: ID_COUNTER.next(),
            device_id: device.id(),
            semaphore,
        })
    }

    pub fn _null(device: &VulkanDevice) -> VulkanSemaphore {
        VulkanSemaphore {
            id: ID_COUNTER.next(),
            device_id: device.id(),
            semaphore: ash::vk::Semaphore::null(),
        }
    }

    pub fn delete_semaphore(
        self,
        device: &VulkanDevice,
    ) -> Result<(), VulkanSemaphoreError> {
        if self.device_id != device.id() {
            return Err(VulkanSemaphoreError::DeviceMismatch);
        }

        unsafe {
            device
                .get_device_raw()
                .destroy_semaphore(*self.get_semaphore_raw(), None);
        }
        Ok(())
    }

    pub fn get_semaphore_raw(&self) -> &ash::vk::Semaphore { &self.semaphore }

    pub fn device_id(&self) -> u32 { self.device_id }

    pub fn id(&self) -> u32 { self.id }
}
