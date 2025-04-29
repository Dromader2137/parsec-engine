use super::context::VulkanError;

pub struct ShaderModule {
    shader_module: ash::vk::ShaderModule
}

#[derive(Debug)]
pub enum ShaderError {
}

impl From<ShaderError> for VulkanError {
    fn from(value: ShaderError) -> Self {
        VulkanError::ShaderError(value)
    }
}

impl ShaderModule {
    pub fn get_shader_module_raw(&self) -> &ash::vk::ShaderModule {
        &self.shader_module
    }
}
