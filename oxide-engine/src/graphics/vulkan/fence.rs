use super::{context::VulkanError, device::Device};

pub struct Fence {
    fence: ash::vk::Fence
}

#[derive(Debug)]
pub enum FenceError {
    CreationError(ash::vk::Result),
    WaitError(ash::vk::Result),
    ResetError(ash::vk::Result),
}

impl From<FenceError> for VulkanError {
    fn from(value: FenceError) -> Self {
        VulkanError::FenceError(value)
    }
}

impl Fence {
    pub fn new(device: &Device, signaled: bool) -> Result<Fence, FenceError> {
        let mut create_info = ash::vk::FenceCreateInfo::default();
        if signaled {
            create_info = create_info.flags(ash::vk::FenceCreateFlags::SIGNALED);
        }

        let fence = match unsafe { device.get_device_raw().create_fence(&create_info, None) } {
            Ok(val) => val,
            Err(err) => return Err(FenceError::CreationError(err))
        };

        Ok( Fence { fence } )
    }

    pub fn wait(&self, device: &Device) -> Result<(), FenceError> {
        if let Err(err) = unsafe { device.get_device_raw().wait_for_fences(&[self.fence], true, u64::MAX) } {
            return Err(FenceError::WaitError(err))
        }
        Ok(())
    }
    
    pub fn reset(&self, device: &Device) -> Result<(), FenceError> {
        if let Err(err) = unsafe { device.get_device_raw().reset_fences(&[self.fence]) } {
            return Err(FenceError::ResetError(err))
        }
        Ok(())
    }
    
    pub fn null() -> Fence {
        Fence { fence: ash::vk::Fence::null() }
    }

    pub fn get_fence_raw(&self) -> &ash::vk::Fence {
        &self.fence
    }
}
