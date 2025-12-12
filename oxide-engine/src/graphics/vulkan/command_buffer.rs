use std::sync::atomic::{AtomicU32, Ordering};

use crate::graphics::vulkan::{
    VulkanError, buffer::VulkanBuffer, descriptor_set::VulkanDescriptorSet,
    device::VulkanDevice, framebuffer::VulkanFramebuffer,
    graphics_pipeline::VulkanGraphicsPipeline,
    physical_device::VulkanPhysicalDevice, renderpass::VulkanRenderpass,
};

pub struct VulkanCommandPool {
    id: u32,
    device_id: u32,
    command_pool: ash::vk::CommandPool,
}

pub struct VulkanCommandBuffer {
    id: u32,
    device_id: u32,
    command_pool_id: u32,
    command_buffer: ash::vk::CommandBuffer,
}

#[derive(Debug)]
pub enum VulkanCommandBufferError {
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

impl From<VulkanCommandBufferError> for VulkanError {
    fn from(value: VulkanCommandBufferError) -> Self {
        VulkanError::VulkanCommandBufferError(value)
    }
}

#[derive(Debug)]
pub enum VulkanCommandPoolError {
    CreationError(ash::vk::Result),
    PhysicalDeviceMismatch,
}

impl From<VulkanCommandPoolError> for VulkanError {
    fn from(value: VulkanCommandPoolError) -> Self {
        VulkanError::VulkanCommandPoolError(value)
    }
}

impl VulkanCommandPool {
    const ID_COUNTER: AtomicU32 = AtomicU32::new(0);

    pub fn new(
        physical_device: &VulkanPhysicalDevice,
        device: &VulkanDevice,
    ) -> Result<VulkanCommandPool, VulkanCommandPoolError> {
        if physical_device.id() != device.physical_device_id() {
            return Err(VulkanCommandPoolError::PhysicalDeviceMismatch);
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
            Err(err) => return Err(VulkanCommandPoolError::CreationError(err)),
        };

        let id = Self::ID_COUNTER.load(Ordering::Acquire);
        Self::ID_COUNTER.store(id + 1, Ordering::Release);

        Ok(VulkanCommandPool {
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

impl VulkanCommandBuffer {
    const ID_COUNTER: AtomicU32 = AtomicU32::new(0);

    pub fn new(
        device: &VulkanDevice,
        command_pool: &VulkanCommandPool,
    ) -> Result<VulkanCommandBuffer, VulkanCommandBufferError> {
        if device.id() != command_pool.device_id() {
            return Err(VulkanCommandBufferError::DeviceMismatch);
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
            Err(err) => {
                return Err(VulkanCommandBufferError::CreationError(err));
            },
        }[0];

        let id = Self::ID_COUNTER.load(Ordering::Acquire);
        Self::ID_COUNTER.store(id + 1, Ordering::Release);

        Ok(VulkanCommandBuffer {
            id,
            device_id: device.id(),
            command_pool_id: command_pool.id(),
            command_buffer,
        })
    }

    pub fn begin(
        &self,
        device: &VulkanDevice,
    ) -> Result<(), VulkanCommandBufferError> {
        if self.device_id != device.id() {
            return Err(VulkanCommandBufferError::DeviceMismatch);
        }

        let begin_info = ash::vk::CommandBufferBeginInfo::default()
            .flags(ash::vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);

        if let Err(err) = unsafe {
            device
                .get_device_raw()
                .begin_command_buffer(self.command_buffer, &begin_info)
        } {
            return Err(VulkanCommandBufferError::BeginError(err));
        };

        Ok(())
    }

    pub fn end(
        &self,
        device: &VulkanDevice,
    ) -> Result<(), VulkanCommandBufferError> {
        if self.device_id != device.id() {
            return Err(VulkanCommandBufferError::DeviceMismatch);
        }

        if let Err(err) = unsafe {
            device
                .get_device_raw()
                .end_command_buffer(self.command_buffer)
        } {
            return Err(VulkanCommandBufferError::EndError(err));
        };

        Ok(())
    }

    pub fn begin_renderpass(
        &self,
        device: &VulkanDevice,
        framebuffer: &VulkanFramebuffer,
        renderpass: &VulkanRenderpass,
    ) -> Result<(), VulkanCommandBufferError> {
        if self.device_id != device.id()
            || self.device_id != renderpass.device_id()
        {
            return Err(VulkanCommandBufferError::DeviceMismatch);
        }

        if renderpass.id() != framebuffer.renderpass_id() {
            return Err(VulkanCommandBufferError::RenderpassMismatch);
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
        device: &VulkanDevice,
    ) -> Result<(), VulkanCommandBufferError> {
        if self.device_id != device.id() {
            return Err(VulkanCommandBufferError::DeviceMismatch);
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
        device: &VulkanDevice,
        framebuffer: &VulkanFramebuffer,
        renderpass: &VulkanRenderpass,
    ) -> Result<(), VulkanCommandBufferError> {
        if self.device_id != device.id()
            || self.device_id != renderpass.device_id()
        {
            return Err(VulkanCommandBufferError::DeviceMismatch);
        }

        if renderpass.id() != framebuffer.renderpass_id() {
            return Err(VulkanCommandBufferError::RenderpassMismatch);
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
        device: &VulkanDevice,
        framebuffer: &VulkanFramebuffer,
        renderpass: &VulkanRenderpass,
    ) -> Result<(), VulkanCommandBufferError> {
        if self.device_id != device.id()
            || self.device_id != renderpass.device_id()
        {
            return Err(VulkanCommandBufferError::DeviceMismatch);
        }

        if renderpass.id() != framebuffer.renderpass_id() {
            return Err(VulkanCommandBufferError::RenderpassMismatch);
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
        device: &VulkanDevice,
        pipeline: &VulkanGraphicsPipeline,
    ) -> Result<(), VulkanCommandBufferError> {
        if self.device_id != device.id()
            || self.device_id != pipeline.device_id()
        {
            return Err(VulkanCommandBufferError::DeviceMismatch);
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
        device: &VulkanDevice,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) -> Result<(), VulkanCommandBufferError> {
        if self.device_id != device.id() {
            return Err(VulkanCommandBufferError::DeviceMismatch);
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
        device: &VulkanDevice,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) -> Result<(), VulkanCommandBufferError> {
        if self.device_id != device.id() {
            return Err(VulkanCommandBufferError::DeviceMismatch);
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
        device: &VulkanDevice,
        buffer: &VulkanBuffer,
    ) -> Result<(), VulkanCommandBufferError> {
        if self.device_id != device.id() {
            return Err(VulkanCommandBufferError::DeviceMismatch);
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
        device: &VulkanDevice,
        buffer: &VulkanBuffer,
    ) -> Result<(), VulkanCommandBufferError> {
        if self.device_id != device.id() {
            return Err(VulkanCommandBufferError::DeviceMismatch);
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
        device: &VulkanDevice,
        descriptor_set: &VulkanDescriptorSet,
        pipeline: &VulkanGraphicsPipeline,
        set_index: u32,
    ) -> Result<(), VulkanCommandBufferError> {
        if self.device_id != device.id()
            || self.device_id != pipeline.device_id()
            || self.device_id != descriptor_set.device_id()
        {
            return Err(VulkanCommandBufferError::DeviceMismatch);
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

    pub fn reset(
        &self,
        device: &VulkanDevice,
    ) -> Result<(), VulkanCommandBufferError> {
        if self.device_id != device.id() {
            return Err(VulkanCommandBufferError::DeviceMismatch);
        }

        if let Err(err) = unsafe {
            device.get_device_raw().reset_command_buffer(
                self.command_buffer,
                ash::vk::CommandBufferResetFlags::RELEASE_RESOURCES,
            )
        } {
            return Err(VulkanCommandBufferError::ResetError(err));
        }

        Ok(())
    }

    pub fn get_command_buffer_raw(&self) -> &ash::vk::CommandBuffer {
        &self.command_buffer
    }

    pub fn id(&self) -> u32 { self.id }

    pub fn command_pool_id(&self) -> u32 { self.command_pool_id }
}
