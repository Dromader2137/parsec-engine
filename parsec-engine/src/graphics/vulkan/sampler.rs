use crate::
    graphics::vulkan::device::VulkanDevice
;

#[derive(Debug)]
pub struct VulkanSampler {
    id: u32,
    device_id: u32,
    sampler: ash::vk::Sampler,
}

#[derive(Debug, thiserror::Error)]
pub enum VulkanSamplerError {
    #[error("Failed to create a Vulkan sampler: {0}")]
    SamplerCreationError(ash::vk::Result),
    #[error("Vulkan sampler created on a different device")]
    DeviceMismatch,
}

crate::create_counter!{ID_COUNTER}
impl VulkanSampler {
    pub fn new(
        device: &VulkanDevice,
    ) -> Result<VulkanSampler, VulkanSamplerError> {
        let sampler_info = ash::vk::SamplerCreateInfo::default();

        let sampler = unsafe {
            device
                .get_device_raw()
                .create_sampler(&sampler_info, None)
                .map_err(|err| VulkanSamplerError::SamplerCreationError(err))?
        };

        Ok(VulkanSampler {
            id: ID_COUNTER.next(),
            device_id: device.id(),
            sampler,
        })
    }

    pub fn delete_sampler(
        self,
        device: &VulkanDevice,
    ) -> Result<(), VulkanSamplerError> {
        if self.device_id != device.id() {
            return Err(VulkanSamplerError::DeviceMismatch);
        }

        unsafe {
            device
                .get_device_raw()
                .destroy_sampler(self.sampler_raw(), None);
        }
        Ok(())
    }

    pub fn sampler_raw(&self) -> ash::vk::Sampler { self.sampler }

    pub fn id(&self) -> u32 { self.id }
}
