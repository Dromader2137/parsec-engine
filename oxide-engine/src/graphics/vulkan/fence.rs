use super::{context::VulkanError, device::Device};

pub struct Fence {
    fence: ash::vk::Fence
}

#[derive(Debug)]
pub enum FenceError {
    CreationError(ash::vk::Result)
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

        let fence = match device.create_fence(create_info) {
            Ok(val) => val,
            Err(err) => return Err(FenceError::CreationError(err))
        };

        Ok( Fence { fence } )
    }

    pub fn get_fence_raw(&self) -> &ash::vk::Fence {
        &self.fence
    }
}
