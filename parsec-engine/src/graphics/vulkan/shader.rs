use std::fs::File;

use crate::graphics::{shader::ShaderType, vulkan::device::VulkanDevice};

#[derive(Debug)]
pub struct VulkanShaderModule {
    id: u32,
    device_id: u32,
    shader_type: ShaderType,
    shader_module: ash::vk::ShaderModule,
}

#[derive(Debug, thiserror::Error)]
pub enum VulkanShaderError {
    #[error("Failed to create a shader: {0}")]
    CreationError(ash::vk::Result),
    #[error("Failed to read a SpirV shader file: {0}")]
    ShaderFileError(std::io::Error),
    #[error("Shader created on another device")]
    DeviceMismatch,
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

crate::create_counter! {ID_COUNTER}
impl VulkanShaderModule {
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

        Ok(VulkanShaderModule {
            id: ID_COUNTER.next(),
            device_id: device.id(),
            shader_module,
            shader_type,
        })
    }

    pub fn delete_shader(
        self,
        device: &VulkanDevice,
    ) -> Result<(), VulkanShaderError> {
        if self.device_id != device.id() {
            return Err(VulkanShaderError::DeviceMismatch);
        }

        unsafe {
            device
                .get_device_raw()
                .destroy_shader_module(*self.get_shader_module_raw(), None);
        }
        Ok(())
    }

    pub fn get_shader_module_raw(&self) -> &ash::vk::ShaderModule {
        &self.shader_module
    }

    pub fn id(&self) -> u32 { self.id }

    pub fn device_id(&self) -> u32 { self.device_id }

    pub fn shader_type(&self) -> ShaderType { self.shader_type }
}
