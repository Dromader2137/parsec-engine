use std::fs::File;

use parsec_engine_graphics::shader::ShaderType;
use parsec_engine_utils::create_counter;

use crate::device::VulkanDevice;

#[derive(Debug)]
pub struct VulkanShaderModule {
    id: u32,
    shader_type: ShaderType,
    raw_shader_module: ash::vk::ShaderModule,
}

#[derive(Debug, thiserror::Error)]
pub enum VulkanShaderError {
    #[error("Failed to create a shader: {0}")]
    CreationError(ash::vk::Result),
    #[error("Failed to read a SpirV shader file: {0}")]
    ShaderFileError(std::io::Error),
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

create_counter! {ID_COUNTER}
impl VulkanShaderModule {
    pub fn new(
        device: &VulkanDevice,
        code: &[u32],
        shader_type: ShaderType,
    ) -> Result<VulkanShaderModule, VulkanShaderError> {
        let create_info = ash::vk::ShaderModuleCreateInfo::default().code(code);

        let shader_module = match unsafe {
            device.raw_device().create_shader_module(&create_info, None)
        } {
            Ok(val) => val,
            Err(err) => return Err(VulkanShaderError::CreationError(err)),
        };

        Ok(VulkanShaderModule {
            id: ID_COUNTER.next(),
            raw_shader_module: shader_module,
            shader_type,
        })
    }

    pub fn destroy(self, device: &VulkanDevice) {
        unsafe {
            device
                .raw_device()
                .destroy_shader_module(self.raw_shader_module, None)
        }
    }

    pub fn raw_handle(&self) -> &ash::vk::ShaderModule {
        &self.raw_shader_module
    }

    pub fn id(&self) -> u32 { self.id }

    pub fn shader_type(&self) -> ShaderType { self.shader_type }
}
