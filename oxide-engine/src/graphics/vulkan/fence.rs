use std::sync::atomic::{AtomicU32, Ordering};

use crate::graphics::vulkan::{VulkanError, device::VulkanDevice};

#[derive(Clone)]
pub struct VulkanFence {
    id: u32,
    device_id: u32,
    fence: ash::vk::Fence,
}

#[derive(Debug)]
pub enum VulkanFenceError {
    CreationError(ash::vk::Result),
    WaitError(ash::vk::Result),
    ResetError(ash::vk::Result),
    StatusError(ash::vk::Result),
    DeviceMismatch,
}

impl From<VulkanFenceError> for VulkanError {
    fn from(value: VulkanFenceError) -> Self {
        VulkanError::VulkanFenceError(value)
    }
}

impl VulkanFence {
    const ID_COUNTER: AtomicU32 = AtomicU32::new(0);

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

        let id = Self::ID_COUNTER.load(Ordering::Acquire);
        Self::ID_COUNTER.store(id + 1, Ordering::Release);

        Ok(VulkanFence {
            id,
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
        let id = Self::ID_COUNTER.load(Ordering::Acquire);
        Self::ID_COUNTER.store(id + 1, Ordering::Release);

        VulkanFence {
            id,
            device_id: device.id(),
            fence: ash::vk::Fence::null(),
        }
    }

    pub fn get_fence_raw(&self) -> &ash::vk::Fence { &self.fence }

    pub fn device_id(&self) -> u32 { self.device_id }

    pub fn id(&self) -> u32 { self.id }
}
