use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};

use crate::graphics::vulkan::{VulkanError, device::Device};

#[derive(Clone)]
pub struct Fence {
    id: u32,
    device_id: u32,
    fence: ash::vk::Fence,
}

#[derive(Debug)]
pub enum FenceError {
    CreationError(ash::vk::Result),
    WaitError(ash::vk::Result),
    ResetError(ash::vk::Result),
    StatusError(ash::vk::Result),
    DeviceMismatch,
}

impl From<FenceError> for VulkanError {
    fn from(value: FenceError) -> Self { VulkanError::FenceError(value) }
}

impl Fence {
    const ID_COUNTER: AtomicU32 = AtomicU32::new(0);

    pub fn new(device: &Device, signaled: bool) -> Result<Fence, FenceError> {
        let mut create_info = ash::vk::FenceCreateInfo::default();
        if signaled {
            create_info =
                create_info.flags(ash::vk::FenceCreateFlags::SIGNALED);
        }

        let fence = match unsafe {
            device.get_device_raw().create_fence(&create_info, None)
        } {
            Ok(val) => val,
            Err(err) => return Err(FenceError::CreationError(err)),
        };

        let id = Self::ID_COUNTER.load(Ordering::Acquire);
        Self::ID_COUNTER.store(id + 1, Ordering::Release);

        Ok(Fence {
            id,
            device_id: device.id(),
            fence,
        })
    }

    pub fn wait(&self, device: &Device) -> Result<(), FenceError> {
        if device.id() != self.device_id {
            return Err(FenceError::DeviceMismatch);
        }

        if let Err(err) = unsafe {
            device.get_device_raw().wait_for_fences(
                &[self.fence],
                true,
                u64::MAX,
            )
        } {
            return Err(FenceError::WaitError(err));
        }
        Ok(())
    }

    pub fn reset(&self, device: &Device) -> Result<(), FenceError> {
        if device.id() != self.device_id {
            return Err(FenceError::DeviceMismatch);
        }

        if let Err(err) =
            unsafe { device.get_device_raw().reset_fences(&[self.fence]) }
        {
            return Err(FenceError::ResetError(err));
        }
        Ok(())
    }

    pub fn null(device: &Device) -> Fence {
        let id = Self::ID_COUNTER.load(Ordering::Acquire);
        Self::ID_COUNTER.store(id + 1, Ordering::Release);

        Fence {
            id,
            device_id: device.id(),
            fence: ash::vk::Fence::null(),
        }
    }

    pub fn get_fence_raw(&self) -> &ash::vk::Fence { &self.fence }

    pub fn device_id(&self) -> u32 { self.device_id }
}
