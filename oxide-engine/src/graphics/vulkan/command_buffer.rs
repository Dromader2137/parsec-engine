use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};

use crate::graphics::vulkan::{
    VulkanError, buffer::Buffer, descriptor_set::DescriptorSet, device::Device,
    framebuffer::Framebuffer, graphics_pipeline::GraphicsPipeline,
    physical_device::PhysicalDevice, renderpass::Renderpass,
};

pub struct CommandPool {
    id: u32,
    device_id: u32,
    command_pool: ash::vk::CommandPool,
}

pub struct CommandBuffer {
    id: u32,
    device_id: u32,
    command_pool_id: u32,
    command_buffer: ash::vk::CommandBuffer,
}

#[derive(Debug)]
pub enum CommandBufferError {
    CreationError(ash::vk::Result),
    BeginError(ash::vk::Result),
    EndError(ash::vk::Result),
    RenderpassBeginError(ash::vk::Result),
    RenderpassEndError(ash::vk::Result),
    ResetError(ash::vk::Result),
    DeviceMismatch,
    PoolMismatch,
    RenderpassMismatch,
}

impl From<CommandBufferError> for VulkanError {
    fn from(value: CommandBufferError) -> Self {
        VulkanError::CommandBufferError(value)
    }
}

#[derive(Debug)]
pub enum CommandPoolError {
    CreationError(ash::vk::Result),
    PhysicalDeviceMismatch,
}

impl From<CommandPoolError> for VulkanError {
    fn from(value: CommandPoolError) -> Self {
        VulkanError::CommandPoolError(value)
    }
}

impl CommandPool {
    const ID_COUNTER: AtomicU32 = AtomicU32::new(0);

    pub fn new(
        physical_device: &PhysicalDevice,
        device: &Device,
    ) -> Result<CommandPool, CommandPoolError> {
        if physical_device.id() != device.physical_device_id() {
            return Err(CommandPoolError::PhysicalDeviceMismatch);
        }

        let pool_info = ash::vk::CommandPoolCreateInfo::default()
            .flags(ash::vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(physical_device.get_queue_family_index());

        let command_pool = match unsafe {
            device
                .get_device_raw()
                .create_command_pool(&pool_info, None)
        } {
            Ok(val) => val,
            Err(err) => return Err(CommandPoolError::CreationError(err)),
        };

        let id = Self::ID_COUNTER.load(Ordering::Acquire);
        Self::ID_COUNTER.store(id + 1, Ordering::Release);

        Ok(CommandPool {
            id,
            device_id: device.id(),
            command_pool,
        })
    }

    pub fn get_command_pool_raw(&self) -> &ash::vk::CommandPool {
        &self.command_pool
    }

    pub fn id(&self) -> u32 { self.id }

    pub fn device_id(&self) -> u32 { self.device_id }
}

impl CommandBuffer {
    const ID_COUNTER: AtomicU32 = AtomicU32::new(0);

    pub fn new(
        device: &Device,
        command_pool: &CommandPool,
    ) -> Result<CommandBuffer, CommandBufferError> {
        if device.id() != command_pool.device_id() {
            return Err(CommandBufferError::DeviceMismatch);
        }

        let create_info = ash::vk::CommandBufferAllocateInfo::default()
            .command_buffer_count(1)
            .command_pool(*command_pool.get_command_pool_raw())
            .level(ash::vk::CommandBufferLevel::PRIMARY);

        let command_buffer = match unsafe {
            device
                .get_device_raw()
                .allocate_command_buffers(&create_info)
        } {
            Ok(val) => val,
            Err(err) => return Err(CommandBufferError::CreationError(err)),
        }[0];

        let id = Self::ID_COUNTER.load(Ordering::Acquire);
        Self::ID_COUNTER.store(id + 1, Ordering::Release);

        Ok(CommandBuffer {
            id,
            device_id: device.id(),
            command_pool_id: command_pool.id(),
            command_buffer,
        })
    }

    pub fn begin(&self, device: &Device) -> Result<(), CommandBufferError> {
        if self.device_id != device.id() {
            return Err(CommandBufferError::DeviceMismatch);
        }

        let begin_info = ash::vk::CommandBufferBeginInfo::default()
            .flags(ash::vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);

        if let Err(err) = unsafe {
            device
                .get_device_raw()
                .begin_command_buffer(self.command_buffer, &begin_info)
        } {
            return Err(CommandBufferError::BeginError(err));
        };

        Ok(())
    }

    pub fn end(&self, device: &Device) -> Result<(), CommandBufferError> {
        if self.device_id != device.id() {
            return Err(CommandBufferError::DeviceMismatch);
        }

        if let Err(err) = unsafe {
            device
                .get_device_raw()
                .end_command_buffer(self.command_buffer)
        } {
            return Err(CommandBufferError::EndError(err));
        };

        Ok(())
    }

    pub fn begin_renderpass(
        &self,
        device: &Device,
        framebuffer: &Framebuffer,
        renderpass: &Renderpass,
    ) -> Result<(), CommandBufferError> {
        if self.device_id != device.id()
            || self.device_id != renderpass.device_id()
        {
            return Err(CommandBufferError::DeviceMismatch);
        }

        if renderpass.id() != framebuffer.renderpass_id() {
            return Err(CommandBufferError::RenderpassMismatch);
        }

        let clear_values = [
            ash::vk::ClearValue {
                color: ash::vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 0.0],
                },
            },
            ash::vk::ClearValue {
                depth_stencil: ash::vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
        ];

        let begin_info = ash::vk::RenderPassBeginInfo::default()
            .render_pass(*renderpass.get_renderpass_raw())
            .framebuffer(*framebuffer.get_framebuffer_raw())
            .render_area(framebuffer.get_extent_raw().into())
            .clear_values(&clear_values);

        unsafe {
            device.get_device_raw().cmd_begin_render_pass(
                self.command_buffer,
                &begin_info,
                ash::vk::SubpassContents::INLINE,
            )
        };

        Ok(())
    }

    pub fn end_renderpass(
        &self,
        device: &Device,
    ) -> Result<(), CommandBufferError> {
        if self.device_id != device.id() {
            return Err(CommandBufferError::DeviceMismatch);
        }

        unsafe {
            device
                .get_device_raw()
                .cmd_end_render_pass(self.command_buffer)
        };

        Ok(())
    }

    pub fn set_viewports(
        &self,
        device: &Device,
        framebuffer: &Framebuffer,
        renderpass: &Renderpass,
    ) -> Result<(), CommandBufferError> {
        if self.device_id != device.id()
            || self.device_id != renderpass.device_id()
        {
            return Err(CommandBufferError::DeviceMismatch);
        }

        if renderpass.id() != framebuffer.renderpass_id() {
            return Err(CommandBufferError::RenderpassMismatch);
        }

        let viewports = [ash::vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: framebuffer.get_extent_raw().width as f32,
            height: framebuffer.get_extent_raw().height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];
        unsafe {
            device.get_device_raw().cmd_set_viewport(
                self.command_buffer,
                0,
                &viewports,
            )
        };

        Ok(())
    }

    pub fn set_scissor(
        &self,
        device: &Device,
        framebuffer: &Framebuffer,
        renderpass: &Renderpass,
    ) -> Result<(), CommandBufferError> {
        if self.device_id != device.id()
            || self.device_id != renderpass.device_id()
        {
            return Err(CommandBufferError::DeviceMismatch);
        }

        if renderpass.id() != framebuffer.renderpass_id() {
            return Err(CommandBufferError::RenderpassMismatch);
        }

        let scissors = [framebuffer.get_extent_raw().into()];
        unsafe {
            device.get_device_raw().cmd_set_scissor(
                self.command_buffer,
                0,
                &scissors,
            )
        };

        Ok(())
    }

    pub fn bind_graphics_pipeline(
        &self,
        device: &Device,
        pipeline: &GraphicsPipeline,
    ) -> Result<(), CommandBufferError> {
        if self.device_id != device.id()
            || self.device_id != pipeline.device_id()
        {
            return Err(CommandBufferError::DeviceMismatch);
        }

        unsafe {
            device.get_device_raw().cmd_bind_pipeline(
                self.command_buffer,
                ash::vk::PipelineBindPoint::GRAPHICS,
                *pipeline.get_pipeline_raw(),
            )
        };

        Ok(())
    }

    pub fn draw(
        &self,
        device: &Device,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) -> Result<(), CommandBufferError> {
        if self.device_id != device.id() {
            return Err(CommandBufferError::DeviceMismatch);
        }

        unsafe {
            device.get_device_raw().cmd_draw(
                self.command_buffer,
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            )
        };

        Ok(())
    }

    pub fn draw_indexed(
        &self,
        device: &Device,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) -> Result<(), CommandBufferError> {
        if self.device_id != device.id() {
            return Err(CommandBufferError::DeviceMismatch);
        }

        unsafe {
            device.get_device_raw().cmd_draw_indexed(
                self.command_buffer,
                index_count,
                instance_count,
                first_index,
                vertex_offset,
                first_instance,
            );
        };

        Ok(())
    }

    pub fn bind_vertex_buffer(
        &self,
        device: &Device,
        buffer: &Buffer,
    ) -> Result<(), CommandBufferError> {
        if self.device_id != device.id() {
            return Err(CommandBufferError::DeviceMismatch);
        }
        unsafe {
            device.get_device_raw().cmd_bind_vertex_buffers(
                self.command_buffer,
                0,
                &[*buffer.get_buffer_raw()],
                &[0],
            )
        };

        Ok(())
    }

    pub fn bind_index_buffer(
        &self,
        device: &Device,
        buffer: &Buffer,
    ) -> Result<(), CommandBufferError> {
        if self.device_id != device.id() {
            return Err(CommandBufferError::DeviceMismatch);
        }

        unsafe {
            device.get_device_raw().cmd_bind_index_buffer(
                self.command_buffer,
                *buffer.get_buffer_raw(),
                0,
                ash::vk::IndexType::UINT32,
            )
        };

        Ok(())
    }

    pub fn bind_descriptor_set(
        &self,
        device: &Device,
        descriptor_set: &DescriptorSet,
        pipeline: &GraphicsPipeline,
        set_index: u32,
    ) -> Result<(), CommandBufferError> {
        if self.device_id != device.id()
            || self.device_id != pipeline.device_id()
            || self.device_id != descriptor_set.device_id()
        {
            return Err(CommandBufferError::DeviceMismatch);
        }

        unsafe {
            device.get_device_raw().cmd_bind_descriptor_sets(
                self.command_buffer,
                ash::vk::PipelineBindPoint::GRAPHICS,
                *pipeline.get_layout_raw(),
                set_index,
                &[*descriptor_set.get_set_raw()],
                &[],
            );
        };

        Ok(())
    }

    pub fn reset(&self, device: &Device) -> Result<(), CommandBufferError> {
        if self.device_id != device.id() {
            return Err(CommandBufferError::DeviceMismatch);
        }

        if let Err(err) = unsafe {
            device.get_device_raw().reset_command_buffer(
                self.command_buffer,
                ash::vk::CommandBufferResetFlags::RELEASE_RESOURCES,
            )
        } {
            return Err(CommandBufferError::ResetError(err));
        }

        Ok(())
    }

    pub fn get_command_buffer_raw(&self) -> &ash::vk::CommandBuffer {
        &self.command_buffer
    }

    pub fn id(&self) -> u32 { self.id }
}
