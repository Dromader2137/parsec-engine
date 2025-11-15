use std::sync::Arc;

use super::{
    VulkanError, buffer::Buffer, descriptor_set::DescriptorSet, device::Device,
    framebuffer::Framebuffer, graphics_pipeline::GraphicsPipeline,
};

pub struct CommandPool {
    pub device: Arc<Device>,
    command_pool: ash::vk::CommandPool,
}

pub struct CommandBuffer {
    pub command_pool: Arc<CommandPool>,
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
}

impl From<CommandBufferError> for VulkanError {
    fn from(value: CommandBufferError) -> Self {
        VulkanError::CommandBufferError(value)
    }
}

#[derive(Debug)]
pub enum CommandPoolError {
    CreationError(ash::vk::Result),
}

impl From<CommandPoolError> for VulkanError {
    fn from(value: CommandPoolError) -> Self {
        VulkanError::CommandPoolError(value)
    }
}

impl CommandPool {
    pub fn new(device: Arc<Device>) -> Result<Arc<CommandPool>, CommandPoolError> {
        let pool_info = ash::vk::CommandPoolCreateInfo::default()
            .flags(ash::vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(device.physical_device.get_queue_family_index());

        let command_pool = match unsafe {
            device
                .get_device_raw()
                .create_command_pool(&pool_info, None)
        } {
            Ok(val) => val,
            Err(err) => return Err(CommandPoolError::CreationError(err)),
        };

        Ok(Arc::new(CommandPool {
            device,
            command_pool,
        }))
    }

    pub fn get_command_pool_raw(&self) -> &ash::vk::CommandPool {
        &self.command_pool
    }
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        unsafe {
            self.device
                .get_device_raw()
                .destroy_command_pool(self.command_pool, None)
        };
    }
}

impl CommandBuffer {
    pub fn new(command_pool: Arc<CommandPool>) -> Result<Arc<CommandBuffer>, CommandBufferError> {
        let create_info = ash::vk::CommandBufferAllocateInfo::default()
            .command_buffer_count(1)
            .command_pool(*command_pool.get_command_pool_raw())
            .level(ash::vk::CommandBufferLevel::PRIMARY);

        let command_buffer = match unsafe {
            command_pool
                .device
                .get_device_raw()
                .allocate_command_buffers(&create_info)
        } {
            Ok(val) => val,
            Err(err) => return Err(CommandBufferError::CreationError(err)),
        }[0];

        Ok(Arc::new(CommandBuffer {
            command_pool,
            command_buffer,
        }))
    }

    pub fn begin(&self) -> Result<(), CommandBufferError> {
        let begin_info = ash::vk::CommandBufferBeginInfo::default()
            .flags(ash::vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);

        if let Err(err) = unsafe {
            self.command_pool
                .device
                .get_device_raw()
                .begin_command_buffer(self.command_buffer, &begin_info)
        } {
            return Err(CommandBufferError::BeginError(err));
        };

        Ok(())
    }

    pub fn end(&self) -> Result<(), CommandBufferError> {
        if let Err(err) = unsafe {
            self.command_pool
                .device
                .get_device_raw()
                .end_command_buffer(self.command_buffer)
        } {
            return Err(CommandBufferError::EndError(err));
        };

        Ok(())
    }

    pub fn begin_renderpass(&self, framebuffer: Arc<Framebuffer>) {
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
            .render_pass(*framebuffer.renderpass.get_renderpass_raw())
            .framebuffer(*framebuffer.get_framebuffer_raw())
            .render_area(framebuffer.get_extent_raw().into())
            .clear_values(&clear_values);

        unsafe {
            framebuffer
                .renderpass
                .device
                .get_device_raw()
                .cmd_begin_render_pass(
                    self.command_buffer,
                    &begin_info,
                    ash::vk::SubpassContents::INLINE,
                )
        };
    }

    pub fn end_renderpass(&self) {
        unsafe {
            self.command_pool
                .device
                .get_device_raw()
                .cmd_end_render_pass(self.command_buffer)
        };
    }

    pub fn set_viewports(&self, framebuffer: Arc<Framebuffer>) {
        let viewports = [ash::vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: framebuffer.get_extent_raw().width as f32,
            height: framebuffer.get_extent_raw().height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];
        unsafe {
            self.command_pool.device.get_device_raw().cmd_set_viewport(
                self.command_buffer,
                0,
                &viewports,
            )
        };
    }

    pub fn set_scissor(&self, framebuffer: Arc<Framebuffer>) {
        let scissors = [framebuffer.get_extent_raw().into()];
        unsafe {
            self.command_pool.device.get_device_raw().cmd_set_scissor(
                self.command_buffer,
                0,
                &scissors,
            )
        };
    }

    pub fn bind_graphics_pipeline(&self, pipeline: Arc<GraphicsPipeline>) {
        unsafe {
            self.command_pool.device.get_device_raw().cmd_bind_pipeline(
                self.command_buffer,
                ash::vk::PipelineBindPoint::GRAPHICS,
                *pipeline.get_pipeline_raw(),
            )
        };
    }

    pub fn draw(
        &self,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) {
        unsafe {
            self.command_pool.device.get_device_raw().cmd_draw(
                self.command_buffer,
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            )
        };
    }

    pub fn draw_indexed(
        &self,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) {
        unsafe {
            self.command_pool.device.get_device_raw().cmd_draw_indexed(
                self.command_buffer,
                index_count,
                instance_count,
                first_index,
                vertex_offset,
                first_instance,
            );
        };
    }

    pub fn bind_vertex_buffer(&self, buffer: Arc<Buffer>) {
        unsafe {
            self.command_pool
                .device
                .get_device_raw()
                .cmd_bind_vertex_buffers(self.command_buffer, 0, &[*buffer.get_buffer_raw()], &[0])
        };
    }

    pub fn bind_index_buffer(&self, buffer: Arc<Buffer>) {
        unsafe {
            self.command_pool
                .device
                .get_device_raw()
                .cmd_bind_index_buffer(
                    self.command_buffer,
                    *buffer.get_buffer_raw(),
                    0,
                    ash::vk::IndexType::UINT32,
                )
        };
    }

    pub fn bind_descriptor_set(
        &self,
        descriptor_set: Arc<DescriptorSet>,
        pipeline: Arc<GraphicsPipeline>,
        set_index: u32,
    ) {
        unsafe {
            self.command_pool
                .device
                .get_device_raw()
                .cmd_bind_descriptor_sets(
                    self.command_buffer,
                    ash::vk::PipelineBindPoint::GRAPHICS,
                    *pipeline.get_layout_raw(),
                    set_index,
                    &[*descriptor_set.get_set_raw()],
                    &[],
                );
        };
    }

    pub fn reset(&self) -> Result<(), CommandBufferError> {
        if let Err(err) = unsafe {
            self.command_pool
                .device
                .get_device_raw()
                .reset_command_buffer(
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
}
