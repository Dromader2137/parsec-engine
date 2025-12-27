use crate::{
    graphics::vulkan::{
        buffer::VulkanBuffer,
        descriptor_set::VulkanDescriptorSet,
        device::VulkanDevice,
        framebuffer::VulkanFramebuffer,
        graphics_pipeline::VulkanGraphicsPipeline,
        image::{VulkanImage, VulkanOwnedImage},
        physical_device::VulkanPhysicalDevice,
        renderpass::VulkanRenderpass,
    },
};

pub struct VulkanCommandPool {
    id: u32,
    device_id: u32,
    command_pool: ash::vk::CommandPool,
}

pub struct VulkanCommandBuffer {
    id: u32,
    device_id: u32,
    _command_pool_id: u32,
    command_buffer: ash::vk::CommandBuffer,
}

#[derive(Debug, thiserror::Error)]
pub enum VulkanCommandBufferError {
    #[error("Failed to create Command Buffer: {0}")]
    CreationError(ash::vk::Result),
    #[error("Failed to begin Command Buffer: {0}")]
    BeginError(ash::vk::Result),
    #[error("Failed to end Command Buffer: {0}")]
    EndError(ash::vk::Result),
    #[error("Failed to reset Command Buffer: {0}")]
    ResetError(ash::vk::Result),
    #[error("Command Buffer created on a different Device")]
    DeviceMismatch,
    #[error("Renderpass doesn't match")]
    RenderpassMismatch,
}

#[derive(Debug, thiserror::Error)]
pub enum VulkanCommandPoolError {
    #[error("Failed to create command pool: {0}")]
    CreationError(ash::vk::Result),
    #[error("Device created on different physical device")]
    PhysicalDeviceMismatch,
}

crate::create_counter!{ID_COUNTER_POOL}
impl VulkanCommandPool {
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

        Ok(VulkanCommandPool {
            id: ID_COUNTER_POOL.next(),
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

crate::create_counter!{ID_COUNTER_BUFFER}
impl VulkanCommandBuffer {
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

        Ok(VulkanCommandBuffer {
            id: ID_COUNTER_BUFFER.next(),
            device_id: device.id(),
            _command_pool_id: command_pool.id(),
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

        let clear_values = renderpass.clear_values();

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
        dimensions: (u32, u32),
        renderpass: &VulkanRenderpass,
    ) -> Result<(), VulkanCommandBufferError> {
        if self.device_id != device.id()
            || self.device_id != renderpass.device_id()
        {
            return Err(VulkanCommandBufferError::DeviceMismatch);
        }

        let viewports = [ash::vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: dimensions.0 as f32,
            height: dimensions.1 as f32,
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
        dimensions: (u32, u32),
        renderpass: &VulkanRenderpass,
    ) -> Result<(), VulkanCommandBufferError> {
        if self.device_id != device.id()
            || self.device_id != renderpass.device_id()
        {
            return Err(VulkanCommandBufferError::DeviceMismatch);
        }

        let scissors = [
            ash::vk::Rect2D {
                extent: ash::vk::Extent2D { width: dimensions.0, height: dimensions.1 },
                offset: ash::vk::Offset2D::default()
            }.into()
        ];
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

    pub fn _draw(
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

    pub fn copy_buffer_to_image(
        &self,
        device: &VulkanDevice,
        buffer: &VulkanBuffer,
        image: &VulkanOwnedImage,
    ) -> Result<(), VulkanCommandBufferError> {
        if self.device_id != device.id()
            || buffer.device_id() != self.device_id
            || image.device_id() != self.device_id
        {
            return Err(VulkanCommandBufferError::DeviceMismatch);
        }

        let buffer_image_copy = ash::vk::BufferImageCopy::default()
            .image_extent(image.extent())
            .image_subresource(
                ash::vk::ImageSubresourceLayers::default()
                    .layer_count(1)
                    .aspect_mask(image.aspect_flags()),
            );

        unsafe {
            device.get_device_raw().cmd_copy_buffer_to_image(
                self.command_buffer,
                *buffer.get_buffer_raw(),
                *image.get_image_raw(),
                ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[buffer_image_copy],
            )
        };

        Ok(())
    }

    pub fn pipeline_barrier(
        &self,
        device: &VulkanDevice,
        src_stage: PipelineStage,
        dst_stage: PipelineStage,
        memory_barriers: &[MemoryBarrier],
        buffer_memory_barriers: &[BufferMemoryBarrier],
        image_memory_barriers: &[ImageMemoryBarrier],
    ) -> Result<(), VulkanCommandBufferError> {
        if self.device_id != device.id() {
            return Err(VulkanCommandBufferError::DeviceMismatch);
        }

        unsafe {
            device.get_device_raw().cmd_pipeline_barrier(
                self.command_buffer,
                src_stage,
                dst_stage,
                ash::vk::DependencyFlags::empty(),
                memory_barriers,
                buffer_memory_barriers,
                image_memory_barriers,
            );
        }
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

    pub fn _command_pool_id(&self) -> u32 { self._command_pool_id }
}

pub type MemoryBarrier = ash::vk::MemoryBarrier<'static>;
pub type BufferMemoryBarrier = ash::vk::BufferMemoryBarrier<'static>;
pub type ImageMemoryBarrier = ash::vk::ImageMemoryBarrier<'static>;
pub type PipelineStage = ash::vk::PipelineStageFlags;
