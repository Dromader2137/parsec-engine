use std::{
    fs::File,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
};

use crate::graphics::vulkan::{VulkanError, device::Device};

pub struct ShaderModule {
    id: u32,
    device_id: u32,
    shader_type: ShaderType,
    shader_module: ash::vk::ShaderModule,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderType {
    Fragment,
    Vertex,
}

#[derive(Debug)]
pub enum ShaderError {
    CreationError(ash::vk::Result),
    ShaderFileError(std::io::Error),
}

impl From<ShaderError> for VulkanError {
    fn from(value: ShaderError) -> Self { VulkanError::ShaderError(value) }
}

pub fn read_shader_code(path: &str) -> Result<Vec<u32>, ShaderError> {
    let mut file = match File::open(path) {
        Ok(val) => val,
        Err(err) => return Err(ShaderError::ShaderFileError(err)),
    };

    match ash::util::read_spv(&mut file) {
        Ok(val) => Ok(val),
        Err(err) => Err(ShaderError::ShaderFileError(err)),
    }
}

impl ShaderModule {
    const ID_COUNTER: AtomicU32 = AtomicU32::new(0);

    pub fn new(
        device: &Device,
        code: &[u32],
        shader_type: ShaderType,
    ) -> Result<ShaderModule, ShaderError> {
        let create_info = ash::vk::ShaderModuleCreateInfo::default().code(code);

        let shader_module = match unsafe {
            device
                .get_device_raw()
                .create_shader_module(&create_info, None)
        } {
            Ok(val) => val,
            Err(err) => return Err(ShaderError::CreationError(err)),
        };

        let id = Self::ID_COUNTER.load(Ordering::Acquire);
        Self::ID_COUNTER.store(id + 1, Ordering::Release);

        Ok(ShaderModule {
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
}
