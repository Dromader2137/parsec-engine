use std::{fs::File, sync::Arc};

use super::{VulkanError, device::Device};

pub struct ShaderModule {
  pub device: Arc<Device>,
  pub shader_type: ShaderType,
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
  fn from(value: ShaderError) -> Self {
    VulkanError::ShaderError(value)
  }
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
  pub fn new(
    device: Arc<Device>,
    code: &[u32],
    shader_type: ShaderType,
  ) -> Result<Arc<ShaderModule>, ShaderError> {
    let create_info = ash::vk::ShaderModuleCreateInfo::default().code(code);

    let shader_module = match unsafe {
      device
        .get_device_raw()
        .create_shader_module(&create_info, None)
    } {
      Ok(val) => val,
      Err(err) => return Err(ShaderError::CreationError(err)),
    };

    Ok(Arc::new(ShaderModule {
      device,
      shader_module,
      shader_type,
    }))
  }

  pub fn get_shader_module_raw(&self) -> &ash::vk::ShaderModule {
    &self.shader_module
  }
}

impl Drop for ShaderModule {
  fn drop(&mut self) {
    unsafe {
      self
        .device
        .get_device_raw()
        .destroy_shader_module(self.shader_module, None)
    };
  }
}
