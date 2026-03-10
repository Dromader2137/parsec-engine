use std::collections::HashMap;

use crate::{
    graphics::vulkan::{
        access::VulkanAccess,
        barriers::{
            VulkanBufferMemoryBarrier, VulkanImageMemoryBarrier,
            VulkanMemoryBarrier,
        },
        buffer::VulkanBuffer,
        descriptor_set::VulkanDescriptorSet,
        device::VulkanDevice,
        framebuffer::VulkanFramebuffer,
        graphics_pipeline::VulkanGraphicsPipeline,
        image::{VulkanImage, VulkanImageLayout, VulkanOwnedImage},
        physical_device::VulkanPhysicalDevice,
        pipeline_stage::VulkanPipelineStage,
        renderpass::VulkanRenderpass,
        utils::raw_rect_2d,
    },
    math::{ivec::Vec2i, uvec::Vec2u},
};

pub struct VulkanCommandPool {
    id: u32,
    device_id: u32,
    raw_command_pool: ash::vk::CommandPool,
}

pub struct VulkanCommandBufferImageState {
    last_pipeline_stage: VulkanPipelineStage,
    last_layout: VulkanImageLayout,
    last_access: Vec<VulkanAccess>,
}

enum VulkanCommand<'a> {
    Begin,
    End,
    BeginRenderpass(&'a VulkanRenderpass, &'a VulkanFramebuffer),
    EndRenderpass,
    SetViewport(Vec2u),
    SetScissor(Vec2u, Vec2i),
    BindGraphicsPipeline(&'a VulkanGraphicsPipeline),
    Draw(u32, u32, u32, u32),
    DrawIndexed(u32, u32, u32, i32, u32),
    BindVertexBuffer(&'a VulkanBuffer),
    BindIndexBuffer(&'a VulkanBuffer),
    BindDescriptorSet(&'a VulkanDescriptorSet, u32),
    CopyBufferToImage(&'a VulkanBuffer, &'a VulkanOwnedImage),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VulkanCommandBufferState {
    NotStarted,
    Normal,
    Renderpass,
}

pub struct VulkanCommandBuffer<'a> {
    id: u32,
    device_id: u32,
    commands: Vec<VulkanCommand<'a>>,
}

#[derive(Debug, thiserror::Error)]
pub enum VulkanCommandBufferError {
    #[error("Failed to create Command Buffer: {0}")]
    CreationError(ash::vk::Result),
    #[error("Failed to begin Command Buffer: {0}")]
    RawBeginError(ash::vk::Result),
    #[error("Failed to end Command Buffer: {0}")]
    RawEndError(ash::vk::Result),
    #[error("Failed to reset Command Buffer: {0}")]
    ResetError(ash::vk::Result),
    #[error("Command Buffer created on a different Device")]
    DeviceMismatch,
    #[error("Renderpass doesn't match")]
    RenderpassMismatch,
    #[error("Viewport size must be non-zero")]
    InvalidViewportSize,
    #[error("Scissor size must be non-zero")]
    InvalidScissorSize,
    #[error("Used image does not exist in the provided image map")]
    ImageNotFound,
    IncorrectState(VulkanCommandBufferState),
}

#[derive(Debug, thiserror::Error)]
pub enum VulkanCommandPoolError {
    #[error("Failed to create command pool: {0}")]
    CreationError(ash::vk::Result),
    #[error("Device created on different physical device")]
    PhysicalDeviceMismatch,
}

crate::create_counter! {ID_COUNTER_POOL}
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
            .queue_family_index(physical_device.queue_family_index());

        let raw_command_pool = match unsafe {
            device.raw_device().create_command_pool(&pool_info, None)
        } {
            Ok(val) => val,
            Err(err) => return Err(VulkanCommandPoolError::CreationError(err)),
        };

        Ok(VulkanCommandPool {
            id: ID_COUNTER_POOL.next(),
            device_id: device.id(),
            raw_command_pool,
        })
    }

    pub fn raw_command_pool(&self) -> &ash::vk::CommandPool {
        &self.raw_command_pool
    }

    pub fn id(&self) -> u32 { self.id }

    pub fn device_id(&self) -> u32 { self.device_id }
}

crate::create_counter! {COMMAND_BUFFER_ID_COUNTER}
impl VulkanCommandBuffer<'_> {
    pub fn new<'a>(
        device: &'a VulkanDevice,
        command_pool: &'a VulkanCommandPool,
    ) -> Result<VulkanCommandBuffer<'a>, VulkanCommandBufferError> {
        if device.id() != command_pool.device_id() {
            return Err(VulkanCommandBufferError::DeviceMismatch);
        }

        let create_info = ash::vk::CommandBufferAllocateInfo::default()
            .command_buffer_count(1)
            .command_pool(*command_pool.raw_command_pool())
            .level(ash::vk::CommandBufferLevel::PRIMARY);

        let command_buffer = match unsafe {
            device.raw_device().allocate_command_buffers(&create_info)
        } {
            Ok(val) => val,
            Err(err) => {
                return Err(VulkanCommandBufferError::CreationError(err));
            },
        }[0];

        Ok(VulkanCommandBuffer {
            id: COMMAND_BUFFER_ID_COUNTER.next(),
            device_id: device.id(),
            commands: Vec::new(),
        })
    }

    pub fn begin(&mut self) { self.commands.push(VulkanCommand::Begin); }

    pub fn end(&mut self) { self.commands.push(VulkanCommand::End); }

    pub fn begin_renderpass<'a: 'b, 'b>(
        &'a mut self,
        renderpass: &'b VulkanRenderpass,
        framebuffer: &'b VulkanFramebuffer,
    ) {
        self.commands.push(VulkanCommand::BeginRenderpass(renderpass, framebuffer));
    }

    pub fn end_renderpass(&mut self) -> Result<(), VulkanCommandBufferError> {
        self.commands.push(VulkanCommand::EndRenderpass);

        if self.device_id != device.id() {
            return Err(VulkanCommandBufferError::DeviceMismatch);
        }

        unsafe {
            device
                .raw_device()
                .cmd_end_render_pass(self.raw_command_buffer)
        };

        Ok(())
    }

    pub fn set_viewports(
        &mut self,
        dimensions: Vec2u,
    ) -> Result<(), VulkanCommandBufferError> {
        if dimensions.x == 0 && dimensions.y == 0 {
            return Err(VulkanCommandBufferError::InvalidViewportSize);
        }

        self.commands.push(VulkanCommand::SetViewport(dimensions));

        if self.device_id != device.id()
            || self.device_id != renderpass.device_id()
        {
            return Err(VulkanCommandBufferError::DeviceMismatch);
        }

        let viewports = [ash::vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: dimensions.x as f32,
            height: dimensions.y as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];
        unsafe {
            device.raw_device().cmd_set_viewport(
                self.raw_command_buffer,
                0,
                &viewports,
            )
        };

        Ok(())
    }

    pub fn set_scissor(
        &mut self,
        dimensions: Vec2u,
        offset: Vec2i,
    ) -> Result<(), VulkanCommandBufferError> {
        if dimensions.x == 0 && dimensions.y == 0 {
            return Err(VulkanCommandBufferError::InvalidScissorSize);
        }

        self.commands
            .push(VulkanCommand::SetScissor(dimensions, offset));

        if self.device_id != device.id()
            || self.device_id != renderpass.device_id()
        {
            return Err(VulkanCommandBufferError::DeviceMismatch);
        }

        let scissors = [raw_rect_2d(dimensions, offset)];
        unsafe {
            device.raw_device().cmd_set_scissor(
                self.raw_command_buffer,
                0,
                &scissors,
            )
        };

        Ok(())
    }

    pub fn bind_graphics_pipeline(
        &mut self,
        pipeline: &VulkanGraphicsPipeline,
    ) -> Result<(), VulkanCommandBufferError> {
        self.commands
            .push(VulkanCommand::BindGraphicsPipeline(pipeline));

        if self.device_id != device.id()
            || self.device_id != pipeline.device_id()
        {
            return Err(VulkanCommandBufferError::DeviceMismatch);
        }

        unsafe {
            device.raw_device().cmd_bind_pipeline(
                self.raw_command_buffer,
                ash::vk::PipelineBindPoint::GRAPHICS,
                *pipeline.get_pipeline_raw(),
            )
        };

        Ok(())
    }

    pub fn draw(
        &mut self,
        device: &VulkanDevice,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) -> Result<(), VulkanCommandBufferError> {
        if self.device_id != device.id() {
            return Err(VulkanCommandBufferError::DeviceMismatch);
        }

        self.commands.push(VulkanCommand::Draw(
            vertex_count,
            instance_count,
            first_vertex,
            first_instance,
        ));

        unsafe {
            device.raw_device().cmd_draw(
                self.raw_command_buffer,
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            )
        };

        Ok(())
    }

    pub fn draw_indexed(
        &mut self,
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

        self.commands.push(VulkanCommand::DrawIndexed(
            instance_count,
            instance_count,
            first_index,
            vertex_offset,
            first_instance,
        ));

        unsafe {
            device.raw_device().cmd_draw_indexed(
                self.raw_command_buffer,
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
        &mut self,
        buffer: &VulkanBuffer,
    ) -> Result<(), VulkanCommandBufferError> {
        self.commands.push(VulkanCommand::BindVertexBuffer(buffer));

        if self.device_id != device.id() {
            return Err(VulkanCommandBufferError::DeviceMismatch);
        }
        unsafe {
            device.raw_device().cmd_bind_vertex_buffers(
                self.raw_command_buffer,
                0,
                &[*buffer.get_buffer_raw()],
                &[0],
            )
        };

        Ok(())
    }

    pub fn bind_index_buffer(
        &mut self,
        buffer: &VulkanBuffer,
    ) -> Result<(), VulkanCommandBufferError> {
        self.commands.push(VulkanCommand::BindIndexBuffer(buffer));

        if self.device_id != device.id() {
            return Err(VulkanCommandBufferError::DeviceMismatch);
        }

        unsafe {
            device.raw_device().cmd_bind_index_buffer(
                self.raw_command_buffer,
                *buffer.get_buffer_raw(),
                0,
                ash::vk::IndexType::UINT32,
            )
        };

        Ok(())
    }

    pub fn bind_descriptor_set(
        &self,
        descriptor_set: &VulkanDescriptorSet,
        set_index: u32,
    ) -> Result<(), VulkanCommandBufferError> {
        self.commands
            .push(VulkanCommand::BindDescriptorSet(descriptor_set, set_index));

        if self.device_id != device.id()
            || self.device_id != pipeline.device_id()
            || self.device_id != descriptor_set.device_id()
        {
            return Err(VulkanCommandBufferError::DeviceMismatch);
        }

        let image_ids = descriptor_set.bound_image_view_ids();

        for image_id in image_ids.iter() {
            let (access, layout, stage) = self.get_image_state(*image_id);
            let image = images
                .get(image_id)
                .ok_or(VulkanCommandBufferError::ImageNotFound)?;
            self.pipeline_barrier(
                device,
                stage,
                VulkanPipelineStage::VertexShader,
                &[],
                &[],
                &[VulkanImageMemoryBarrier::new(
                    access,
                    &[VulkanAccess::ShaderRead],
                    layout,
                    VulkanImageLayout::ShaderReadOnlyOptimal,
                    image,
                )],
            )?;
        }

        unsafe {
            device.raw_device().cmd_bind_descriptor_sets(
                self.raw_command_buffer,
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
        &mut self,
        buffer: &VulkanBuffer,
        image: &VulkanOwnedImage,
    ) -> Result<(), VulkanCommandBufferError> {
        self.commands
            .push(VulkanCommand::CopyBufferToImage(buffer, image));

        if self.device_id != device.id()
            || buffer.device_id() != self.device_id
            || image.device_id() != self.device_id
        {
            return Err(VulkanCommandBufferError::DeviceMismatch);
        }

        let (access, layout, stage) = self.get_image_state(image.id());

        self.pipeline_barrier(
            device,
            stage,
            VulkanPipelineStage::Transfer,
            &[],
            &[],
            &[VulkanImageMemoryBarrier::new(
                access,
                &[VulkanAccess::TransferWrite],
                layout,
                VulkanImageLayout::TransferDstOptimal,
                image,
            )],
        )?;

        let buffer_image_copy = ash::vk::BufferImageCopy::default()
            .image_extent(image.extent())
            .image_subresource(
                ash::vk::ImageSubresourceLayers::default()
                    .layer_count(1)
                    .aspect_mask(image.aspect().raw_image_aspect()),
            );

        unsafe {
            device.raw_device().cmd_copy_buffer_to_image(
                self.raw_command_buffer,
                *buffer.get_buffer_raw(),
                *image.raw_image(),
                ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[buffer_image_copy],
            )
        };

        self.image_state
            .insert(image.id(), VulkanCommandBufferImageState {
                last_pipeline_stage: VulkanPipelineStage::Transfer,
                last_layout: VulkanImageLayout::TransferDstOptimal,
                last_access: vec![VulkanAccess::TransferWrite],
            });

        Ok(())
    }

    fn pipeline_barrier(
        &self,
        device: &VulkanDevice,
        src_stage: VulkanPipelineStage,
        dst_stage: VulkanPipelineStage,
        memory_barriers: &[VulkanMemoryBarrier],
        buffer_memory_barriers: &[VulkanBufferMemoryBarrier],
        image_memory_barriers: &[VulkanImageMemoryBarrier],
    ) -> Result<(), VulkanCommandBufferError> {
        if self.device_id != device.id() {
            return Err(VulkanCommandBufferError::DeviceMismatch);
        }

        unsafe {
            device.raw_device().cmd_pipeline_barrier(
                self.raw_command_buffer,
                src_stage.raw_pipeline_stage(),
                dst_stage.raw_pipeline_stage(),
                ash::vk::DependencyFlags::empty(),
                &memory_barriers
                    .iter()
                    .map(|x| x.raw_memory_barrier())
                    .collect::<Vec<_>>(),
                &buffer_memory_barriers
                    .iter()
                    .map(|x| x.raw_buffer_memory_barrier())
                    .collect::<Vec<_>>(),
                &image_memory_barriers
                    .iter()
                    .map(|x| x.raw_image_memory_barrier())
                    .collect::<Vec<_>>(),
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
            device.raw_device().reset_command_buffer(
                self.raw_command_buffer,
                ash::vk::CommandBufferResetFlags::RELEASE_RESOURCES,
            )
        } {
            return Err(VulkanCommandBufferError::ResetError(err));
        }

        Ok(())
    }

    pub fn build(
        &mut self,
        device: &VulkanDevice,
        command_pool: &VulkanCommandPool,
    ) -> Result<(), VulkanCommandBufferError> {
        let mut build =
            VulkanCommandBufferBuildData::new(device, command_pool)?;

        for command in self.commands.iter() {
            match command {
                VulkanCommand::Begin => build.begin()?,
                VulkanCommand::End => build.end()?,
                VulkanCommand::BeginRenderpass(renderpass, framebuffer) => {
                    todo!()
                },
                VulkanCommand::EndRenderpass => todo!(),
                VulkanCommand::SetViewport(vec2u) => todo!(),
                VulkanCommand::SetScissor(vec2u, vec2i) => todo!(),
                VulkanCommand::BindGraphicsPipeline(
                    vulkan_graphics_pipeline,
                ) => todo!(),
                VulkanCommand::Draw(_, _, _, _) => todo!(),
                VulkanCommand::DrawIndexed(_, _, _, _, _) => todo!(),
                VulkanCommand::BindVertexBuffer(vulkan_buffer) => todo!(),
                VulkanCommand::BindIndexBuffer(vulkan_buffer) => todo!(),
                VulkanCommand::BindDescriptorSet(vulkan_descriptor_set, _) => {
                    todo!()
                },
                VulkanCommand::CopyBufferToImage(
                    vulkan_buffer,
                    vulkan_owned_image,
                ) => todo!(),
            };
        }

        Ok(())
    }

    pub fn id(&self) -> u32 { self.id }

    fn get_image_state(
        &self,
        image_id: u32,
    ) -> (&[VulkanAccess], VulkanImageLayout, VulkanPipelineStage) {
        match self.image_state.get(&image_id) {
            Some(image_state) => (
                image_state.last_access.as_slice(),
                image_state.last_layout,
                image_state.last_pipeline_stage,
            ),
            None => (
                &[] as &[VulkanAccess],
                VulkanImageLayout::Undefined,
                VulkanPipelineStage::BottomOfPipe,
            ),
        }
    }
}

struct VulkanCommandBufferBuildData<'a> {
    device: &'a VulkanDevice,
    state: VulkanCommandBufferState,
    command_buffer: ash::vk::CommandBuffer,
}

impl VulkanCommandBufferBuildData<'_> {
    fn new<'a>(
        device: &'a VulkanDevice,
        command_pool: &VulkanCommandPool,
    ) -> Result<VulkanCommandBufferBuildData<'a>, VulkanCommandBufferError>
    {
        let create_info = ash::vk::CommandBufferAllocateInfo::default()
            .command_buffer_count(1)
            .command_pool(*command_pool.raw_command_pool())
            .level(ash::vk::CommandBufferLevel::PRIMARY);

        let command_buffer = match unsafe {
            device.raw_device().allocate_command_buffers(&create_info)
        } {
            Ok(val) => val,
            Err(err) => {
                return Err(VulkanCommandBufferError::CreationError(err));
            },
        }[0];

        Ok(VulkanCommandBufferBuildData {
            device,
            state: VulkanCommandBufferState::NotStarted,
            command_buffer,
        })
    }

    fn begin(&mut self) -> Result<(), VulkanCommandBufferError> {
        if self.state != VulkanCommandBufferState::NotStarted {
            return Err(VulkanCommandBufferError::IncorrectState(self.state));
        }

        let begin_info = ash::vk::CommandBufferBeginInfo::default()
            .flags(ash::vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);

        unsafe {
            self.device
                .raw_device()
                .begin_command_buffer(self.command_buffer, &begin_info)
                .map_err(|err| VulkanCommandBufferError::RawBeginError(err))
        }
    }

    fn end(&mut self) -> Result<(), VulkanCommandBufferError> {
        if self.state == VulkanCommandBufferState::NotStarted {
            return Err(VulkanCommandBufferError::IncorrectState(self.state));
        }

        unsafe {
            self.device
                .raw_device()
                .end_command_buffer(self.command_buffer)
                .map_err(|err| VulkanCommandBufferError::RawEndError(err))
        }
    }

    fn begin_renderpass(
        &mut self,
        renderpass: &VulkanRenderpass,
        framebuffer: &VulkanFramebuffer,
    ) -> Result<(), VulkanCommandBufferError> {
        if self.device.id() != renderpass.device_id() {
            return Err(VulkanCommandBufferError::DeviceMismatch);
        }

        if renderpass.id() != framebuffer.renderpass_id() {
            return Err(VulkanCommandBufferError::RenderpassMismatch);
        }

        if self.state != VulkanCommandBufferState::Normal {
            return Err(VulkanCommandBufferError::IncorrectState(self.state));
        }

        let clear_values = renderpass
            .clear_values()
            .iter()
            .map(|v| v.raw_clear_value())
            .collect::<Vec<_>>();

        let begin_info = ash::vk::RenderPassBeginInfo::default()
            .render_pass(*renderpass.get_renderpass_raw())
            .framebuffer(*framebuffer.raw_framebuffer())
            .render_area(raw_rect_2d(framebuffer.dimensions(), Vec2i::ZERO))
            .clear_values(&clear_values);

        unsafe {
            self.device.raw_device().cmd_begin_render_pass(
                self.command_buffer,
                &begin_info,
                ash::vk::SubpassContents::INLINE,
            )
        };
            
        Ok(())
    }
}
