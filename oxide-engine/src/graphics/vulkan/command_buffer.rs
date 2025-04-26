use super::context::VulkanError;

pub struct CommandBuffer {
    command_buffer: ash::vk::CommandBuffer,
}

#[derive(Debug)]
pub enum CommandBufferError {
}

impl From<CommandBufferError> for VulkanError {
    fn from(value: CommandBufferError) -> Self {
        VulkanError::CommandBufferError(value)
    }
}

impl CommandBuffer {
    pub fn new() -> Result<CommandBuffer, CommandBufferError> {
        let 
    }
}
