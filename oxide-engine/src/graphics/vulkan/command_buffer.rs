use super::{context::VulkanError, device::Device, physical_device::PhysicalDevice, renderpass::Renderpass};

pub struct CommandPool {
    command_pool: ash::vk::CommandPool,
}

pub struct CommandBuffer {
    command_buffer: ash::vk::CommandBuffer,
}

#[derive(Debug)]
pub enum CommandBufferError {
    CreationError(ash::vk::Result),
    BeginError(ash::vk::Result),
    EndError(ash::vk::Result),
    RenderpassBeginError(ash::vk::Result),
    RenderpassEndError(ash::vk::Result),
}

impl From<CommandBufferError> for VulkanError {
    fn from(value: CommandBufferError) -> Self {
        VulkanError::CommandBufferError(value)
    }
}

#[derive(Debug)]
pub enum CommandPoolError {
    CreationError(ash::vk::Result)
}

impl From<CommandPoolError> for VulkanError {
    fn from(value: CommandPoolError) -> Self {
        VulkanError::CommandPoolError(value)
    }
}

impl CommandPool {
    pub fn new(physical_device: &PhysicalDevice, device: &Device) -> Result<CommandPool, CommandPoolError> {
        let pool_info = ash::vk::CommandPoolCreateInfo::default()
            .flags(ash::vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(physical_device.get_queue_family_index());

        let command_pool = match device.create_command_pool(pool_info) {
            Ok(val) => val,
            Err(err) => return Err(CommandPoolError::CreationError(err))
        };

        Ok( CommandPool { command_pool } )
    }

    pub fn get_command_pool_raw(&self) -> &ash::vk::CommandPool {
        &self.command_pool
    }
}

impl CommandBuffer {
    pub fn new(device: &Device, pool: &CommandPool) -> Result<CommandBuffer, CommandBufferError> {
        let create_info = ash::vk::CommandBufferAllocateInfo::default()
            .command_buffer_count(1)
            .command_pool(*pool.get_command_pool_raw())
            .level(ash::vk::CommandBufferLevel::PRIMARY);

        let command_buffer = match device.create_command_buffers(create_info) {
            Ok(val) => val,
            Err(err) => return Err(CommandBufferError::CreationError(err))
        }[0];

        Ok(CommandBuffer { command_buffer })
    }

    pub fn begin(&self, device: &Device) -> Result<(), CommandBufferError>{
        let begin_info = ash::vk::CommandBufferBeginInfo::default()
            .flags(ash::vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);

        if let Err(err) = device.begin_command_buffer(self.command_buffer, begin_info) {
            return Err(CommandBufferError::BeginError(err));
        };

        Ok(())
    }
    
    pub fn end(&self, device: &Device) -> Result<(), CommandBufferError>{
        if let Err(err) = device.end_command_buffer(self.command_buffer) {
            return Err(CommandBufferError::EndError(err));
        };

        Ok(())
    }

    pub fn get_command_buffer_raw(&self) -> &ash::vk::CommandBuffer {
        &self.command_buffer
    }
}
