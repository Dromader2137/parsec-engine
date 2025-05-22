use std::sync::Arc;

use super::{VulkanError, device::Device};

#[derive(Clone)]
pub struct Fence {
    pub device: Arc<Device>,
    fence: ash::vk::Fence,
}

#[derive(Debug)]
pub enum FenceError {
    CreationError(ash::vk::Result),
    WaitError(ash::vk::Result),
    ResetError(ash::vk::Result),
    StatusError(ash::vk::Result),
}

impl From<FenceError> for VulkanError {
    fn from(value: FenceError) -> Self {
        VulkanError::FenceError(value)
    }
}

impl Fence {
    pub fn new(device: Arc<Device>, signaled: bool) -> Result<Arc<Fence>, FenceError> {
        let mut create_info = ash::vk::FenceCreateInfo::default();
        if signaled {
            create_info = create_info.flags(ash::vk::FenceCreateFlags::SIGNALED);
        }

        let fence = match unsafe { device.get_device_raw().create_fence(&create_info, None) } {
            Ok(val) => val,
            Err(err) => return Err(FenceError::CreationError(err)),
        };

        Ok(Arc::new(Fence { device, fence }))
    }

    pub fn wait(&self) -> Result<(), FenceError> {
        if let Err(err) = unsafe {
            self
                .device
                .get_device_raw()
                .wait_for_fences(&[self.fence], true, u64::MAX)
        } {
            return Err(FenceError::WaitError(err));
        }
        Ok(())
    }

    pub fn reset(&self) -> Result<(), FenceError> {
        if let Err(err) = unsafe { self.device.get_device_raw().reset_fences(&[self.fence]) } {
            return Err(FenceError::ResetError(err));
        }
        Ok(())
    }

    pub fn get_state(&self) -> Result<bool, FenceError> {
        match unsafe { self.device.get_device_raw().get_fence_status(self.fence) } {
            Ok(val) => Ok(val),
            Err(err) => Err(FenceError::StatusError(err)),
        }
    }

    pub fn null(device: Arc<Device>) -> Arc<Fence> {
        Arc::new(Fence {
            device,
            fence: ash::vk::Fence::null(),
        })
    }

    pub fn get_fence_raw(&self) -> &ash::vk::Fence {
        &self.fence
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        unsafe { self.device.get_device_raw().destroy_fence(self.fence, None) };
    }
}
