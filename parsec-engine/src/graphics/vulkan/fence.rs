use crate::graphics::vulkan::device::VulkanDevice;

#[derive(Clone)]
pub struct VulkanFence {
    id: u32,
    fence: ash::vk::Fence,
}

#[derive(Debug, thiserror::Error)]
pub enum VulkanFenceError {
    #[error("Failed to create fence: {0}")]
    CreationError(ash::vk::Result),
    #[error("Failed to wait for fence: {0}")]
    WaitError(ash::vk::Result),
    #[error("Failed to reset fence: {0}")]
    ResetError(ash::vk::Result),
}

crate::create_counter! {ID_COUNTER}
impl VulkanFence {
    pub fn new(
        device: &VulkanDevice,
        signaled: bool,
    ) -> Result<VulkanFence, VulkanFenceError> {
        let mut create_info = ash::vk::FenceCreateInfo::default();
        if signaled {
            create_info =
                create_info.flags(ash::vk::FenceCreateFlags::SIGNALED);
        }

        let fence = match unsafe {
            device.raw_device().create_fence(&create_info, None)
        } {
            Ok(val) => val,
            Err(err) => return Err(VulkanFenceError::CreationError(err)),
        };

        Ok(VulkanFence {
            id: ID_COUNTER.next(),
            fence,
        })
    }

    pub fn wait(&self, device: &VulkanDevice) -> Result<(), VulkanFenceError> {
        if let Err(err) = unsafe {
            device
                .raw_device()
                .wait_for_fences(&[self.fence], true, u64::MAX)
        } {
            return Err(VulkanFenceError::WaitError(err));
        }
        Ok(())
    }

    pub fn reset(&self, device: &VulkanDevice) -> Result<(), VulkanFenceError> {
        if let Err(err) =
            unsafe { device.raw_device().reset_fences(&[self.fence]) }
        {
            return Err(VulkanFenceError::ResetError(err));
        }
        Ok(())
    }

    pub fn null() -> VulkanFence {
        VulkanFence {
            id: ID_COUNTER.next(),
            fence: ash::vk::Fence::null(),
        }
    }

    pub fn destroy(self, device: &VulkanDevice) {
        unsafe {
            device
                .raw_device()
                .destroy_fence(*self.get_fence_raw(), None)
        }
    }

    pub fn get_fence_raw(&self) -> &ash::vk::Fence { &self.fence }

    pub fn id(&self) -> u32 { self.id }
}
