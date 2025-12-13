use crate::{
    graphics::vulkan::{VulkanError, device::VulkanDevice},
    utils::id_counter::IdCounter,
};

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

static ID_COUNTER: once_cell::sync::Lazy<IdCounter> =
    once_cell::sync::Lazy::new(|| IdCounter::new(0));
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

    pub fn null(device: &VulkanDevice) -> VulkanSemaphore {
        VulkanSemaphore {
            id: ID_COUNTER.next(),
            device_id: device.id(),
            semaphore: ash::vk::Semaphore::null(),
        }
    }

    pub fn get_semaphore_raw(&self) -> &ash::vk::Semaphore { &self.semaphore }

    pub fn device_id(&self) -> u32 { self.device_id }

    pub fn id(&self) -> u32 { self.id }
}
