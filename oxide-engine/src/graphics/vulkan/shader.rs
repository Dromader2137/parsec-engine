use std::{
    fs::File,
    sync::atomic::{AtomicU32, Ordering},
};

use crate::graphics::{
    shader::ShaderType,
    vulkan::{VulkanError, device::VulkanDevice},
};

pub struct VulkanShaderModule {
    id: u32,
    device_id: u32,
    shader_type: ShaderType,
    shader_module: ash::vk::ShaderModule,
}

#[derive(Debug)]
pub enum VulkanShaderError {
    CreationError(ash::vk::Result),
    ShaderFileError(std::io::Error),
}

impl From<VulkanShaderError> for VulkanError {
    fn from(value: VulkanShaderError) -> Self {
        VulkanError::VulkanShaderError(value)
    }
}

pub fn read_shader_code(path: &str) -> Result<Vec<u32>, VulkanShaderError> {
    let mut file = match File::open(path) {
        Ok(val) => val,
        Err(err) => return Err(VulkanShaderError::ShaderFileError(err)),
    };

    match ash::util::read_spv(&mut file) {
        Ok(val) => Ok(val),
        Err(err) => Err(VulkanShaderError::ShaderFileError(err)),
    }
}

impl VulkanShaderModule {
    const ID_COUNTER: AtomicU32 = AtomicU32::new(0);

    pub fn new(
        device: &VulkanDevice,
        code: &[u32],
        shader_type: ShaderType,
    ) -> Result<VulkanShaderModule, VulkanShaderError> {
        let create_info = ash::vk::ShaderModuleCreateInfo::default().code(code);

        let shader_module = match unsafe {
            device
                .get_device_raw()
                .create_shader_module(&create_info, None)
        } {
            Ok(val) => val,
            Err(err) => return Err(VulkanShaderError::CreationError(err)),
        };

        let id = Self::ID_COUNTER.load(Ordering::Acquire);
        Self::ID_COUNTER.store(id + 1, Ordering::Release);

        Ok(VulkanShaderModule {
            id,
            device_id: device.id(),
            shader_module,
            shader_type,
        })
    }

    pub fn get_shader_module_raw(&self) -> &ash::vk::ShaderModule {
        &self.shader_module
    }

    pub fn id(&self) -> u32 { self.id }

    pub fn device_id(&self) -> u32 { self.device_id }

    pub fn shader_type(&self) -> ShaderType { self.shader_type }
}
