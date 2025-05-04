use super::{buffer::Buffer, VulkanError, device::Device, framebuffer::Framebuffer, graphics_pipeline::{GraphicsPipeline, Vertex}, physical_device::PhysicalDevice, renderpass::Renderpass};

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
    ResetError(ash::vk::Result)
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

        let command_pool = match unsafe { device.get_device_raw().create_command_pool(&pool_info, None) } {
            Ok(val) => val,
            Err(err) => return Err(CommandPoolError::CreationError(err))
        };

        Ok( CommandPool { command_pool } )
    }

    pub fn get_command_pool_raw(&self) -> &ash::vk::CommandPool {
        &self.command_pool
    }

    pub fn cleanup(&self, device: &Device) {
        unsafe { device.get_device_raw().destroy_command_pool(self.command_pool, None) };
    }
}

impl CommandBuffer {
    pub fn new(device: &Device, pool: &CommandPool) -> Result<CommandBuffer, CommandBufferError> {
        let create_info = ash::vk::CommandBufferAllocateInfo::default()
            .command_buffer_count(1)
            .command_pool(*pool.get_command_pool_raw())
            .level(ash::vk::CommandBufferLevel::PRIMARY);

        let command_buffer = match unsafe { device.get_device_raw().allocate_command_buffers(&create_info) } {
            Ok(val) => val,
            Err(err) => return Err(CommandBufferError::CreationError(err))
        }[0];

        Ok(CommandBuffer { command_buffer })
    }

    pub fn begin(&self, device: &Device) -> Result<(), CommandBufferError>{
        let begin_info = ash::vk::CommandBufferBeginInfo::default()
            .flags(ash::vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);

        if let Err(err) = unsafe { device.get_device_raw().begin_command_buffer(self.command_buffer, &begin_info) } {
            return Err(CommandBufferError::BeginError(err));
        };

        Ok(())
    }
    
    pub fn end(&self, device: &Device) -> Result<(), CommandBufferError>{
        if let Err(err) = unsafe { device.get_device_raw().end_command_buffer(self.command_buffer) } {
            return Err(CommandBufferError::EndError(err));
        };

        Ok(())
    }

    pub fn begin_renderpass(&self, device: &Device, renderpass: &Renderpass, framebuffer: &Framebuffer) {
        let clear_values = [
            ash::vk::ClearValue {
                color: ash::vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 0.0],
                },
            },
        ];

        let begin_info = ash::vk::RenderPassBeginInfo::default()
            .render_pass(*renderpass.get_renderpass_raw())
            .framebuffer(*framebuffer.get_framebuffer_raw())
            .render_area(framebuffer.get_extent_raw().into())
            .clear_values(&clear_values);
        
        unsafe { device.get_device_raw().cmd_begin_render_pass(self.command_buffer, &begin_info, ash::vk::SubpassContents::INLINE) };
    }

    pub fn end_renderpass(&self, device: &Device) {
        unsafe { device.get_device_raw().cmd_end_render_pass(self.command_buffer) };
    }

    pub fn set_viewports(&self, device: &Device, framebuffer: &Framebuffer) {
        let viewports = [ash::vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: framebuffer.get_extent_raw().width as f32,
            height: framebuffer.get_extent_raw().height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];
        unsafe { device.get_device_raw().cmd_set_viewport(self.command_buffer, 0, &viewports) };
    }
    
    pub fn set_scissor(&self, device: &Device, framebuffer: &Framebuffer) {
        let scissors = [framebuffer.get_extent_raw().into()];
        unsafe { device.get_device_raw().cmd_set_scissor(self.command_buffer, 0, &scissors) };
    }

    pub fn bind_graphics_pipeline(&self, device: &Device, pipeline: &GraphicsPipeline) {
        unsafe { device.get_device_raw().cmd_bind_pipeline(self.command_buffer, ash::vk::PipelineBindPoint::GRAPHICS, *pipeline.get_pipeline_raw()) };
    }

    pub fn draw(&self, device: &Device, vertex_count: u32, instance_count: u32, first_vertex: u32, first_instance: u32) {
        unsafe { device.get_device_raw().cmd_draw(self.command_buffer, vertex_count, instance_count, first_vertex, first_instance) };
    }

    pub fn bind_vertex_buffer(&self, device: &Device, buffer: &Buffer<impl Vertex>) {
        unsafe { device.get_device_raw().cmd_bind_vertex_buffers(self.command_buffer, 0, &[*buffer.get_buffer_raw()], &[0]) };
    }

    pub fn reset(&self, device: &Device) -> Result<(), CommandBufferError> {
        if let Err(err) = unsafe { device.get_device_raw().reset_command_buffer(self.command_buffer, ash::vk::CommandBufferResetFlags::RELEASE_RESOURCES) } {
            return Err(CommandBufferError::ResetError(err));
        }

        Ok(())
    }

    pub fn get_command_buffer_raw(&self) -> &ash::vk::CommandBuffer {
        &self.command_buffer
    }
}
