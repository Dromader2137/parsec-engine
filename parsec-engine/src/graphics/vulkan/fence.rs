use crate::{
    graphics::vulkan::device::VulkanDevice, utils::id_counter::IdCounter,
};

#[derive(Clone)]
pub struct VulkanFence {
    id: u32,
    device_id: u32,
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
    #[error("Fence created on different device")]
    DeviceMismatch,
}

static ID_COUNTER: once_cell::sync::Lazy<IdCounter> =
    once_cell::sync::Lazy::new(|| IdCounter::new(0));
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
            device.get_device_raw().create_fence(&create_info, None)
        } {
            Ok(val) => val,
            Err(err) => return Err(VulkanFenceError::CreationError(err)),
        };

        Ok(VulkanFence {
            id: ID_COUNTER.next(),
            device_id: device.id(),
            fence,
        })
    }

    pub fn wait(&self, device: &VulkanDevice) -> Result<(), VulkanFenceError> {
        if device.id() != self.device_id {
            return Err(VulkanFenceError::DeviceMismatch);
        }

        if let Err(err) = unsafe {
            device.get_device_raw().wait_for_fences(
                &[self.fence],
                true,
                u64::MAX,
            )
        } {
            return Err(VulkanFenceError::WaitError(err));
        }
        Ok(())
    }

    pub fn reset(&self, device: &VulkanDevice) -> Result<(), VulkanFenceError> {
        if device.id() != self.device_id {
            return Err(VulkanFenceError::DeviceMismatch);
        }

        if let Err(err) =
            unsafe { device.get_device_raw().reset_fences(&[self.fence]) }
        {
            return Err(VulkanFenceError::ResetError(err));
        }
        Ok(())
    }

    pub fn null(device: &VulkanDevice) -> VulkanFence {
        VulkanFence {
            id: ID_COUNTER.next(),
            device_id: device.id(),
            fence: ash::vk::Fence::null(),
        }
    }

    pub fn delete_fence(
        self,
        device: &VulkanDevice,
    ) -> Result<(), VulkanFenceError> {
        if self.device_id != device.id() {
            return Err(VulkanFenceError::DeviceMismatch);
        }

        unsafe {
            device
                .get_device_raw()
                .destroy_fence(*self.get_fence_raw(), None);
        }
        Ok(())
    }

    pub fn get_fence_raw(&self) -> &ash::vk::Fence { &self.fence }

    pub fn device_id(&self) -> u32 { self.device_id }

    pub fn id(&self) -> u32 { self.id }
}
