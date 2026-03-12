use std::collections::{HashMap, HashSet};

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
    last_pipeline_stage: Vec<VulkanPipelineStage>,
    pub last_layout: VulkanImageLayout,
    last_access: Vec<VulkanAccess>,
}

#[derive(Clone)]
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
pub enum VulkanCommandBufferState {
    NotStarted,
    Normal,
    Renderpass,
    Ended,
}

pub struct VulkanCommandBuffer {
    id: u32,
    device_id: u32,
    image_state: HashMap<u32, VulkanCommandBufferImageState>,
    raw_command_buffer: ash::vk::CommandBuffer,
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
    #[error("Command used in an incorrect command buffer state: {0:?}")]
    IncorrectState(VulkanCommandBufferState),
    #[error("Pipeline not bound")]
    PipelineNotBound,
    #[error("Renderpass not started")]
    RenderpassNotStarted,
    #[error("WONT HAPPEN")]
    IllegalCommandInsiderRenderpass,
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
impl VulkanCommandBuffer {
    pub fn new(
        device: &VulkanDevice,
        command_pool: &VulkanCommandPool,
    ) -> Result<Self, VulkanCommandBufferError> {
        if device.id() != command_pool.device_id() {
            return Err(VulkanCommandBufferError::DeviceMismatch);
        }

        let create_info = ash::vk::CommandBufferAllocateInfo::default()
            .command_buffer_count(1)
            .command_pool(*command_pool.raw_command_pool())
            .level(ash::vk::CommandBufferLevel::PRIMARY);

        let command_buffer = unsafe {
            device
                .raw_device()
                .allocate_command_buffers(&create_info)
                .map_err(|err| VulkanCommandBufferError::CreationError(err))?[0]
        };

        Ok(Self {
            id: COMMAND_BUFFER_ID_COUNTER.next(),
            device_id: device.id(),
            image_state: HashMap::new(),
            raw_command_buffer: command_buffer,
        })
    }

    pub fn reset(
        &mut self,
        device: &VulkanDevice,
    ) -> Result<(), VulkanCommandBufferError> {
        if self.device_id != device.id() {
            return Err(VulkanCommandBufferError::DeviceMismatch);
        }

        self.image_state.clear();

        unsafe {
            device
                .raw_device()
                .reset_command_buffer(
                    self.raw_command_buffer,
                    ash::vk::CommandBufferResetFlags::RELEASE_RESOURCES,
                )
                .map_err(|err| VulkanCommandBufferError::ResetError(err))
        }
    }

    pub fn id(&self) -> u32 { self.id }

    pub fn raw_command_buffer(&self) -> &ash::vk::CommandBuffer {
        &self.raw_command_buffer
    }

    pub fn image_state(&self) -> &HashMap<u32, VulkanCommandBufferImageState> {
        &self.image_state
    }
}

pub struct VulkanCommandBufferBuilder<'a> {
    device: &'a VulkanDevice,
    command_buffer: &'a mut VulkanCommandBuffer,

    state: VulkanCommandBufferState,
    current_renderpass_commands: Vec<VulkanCommand<'a>>,
    bound_pipeline: Option<&'a VulkanGraphicsPipeline>,
    descriptor_set_images: HashSet<u32>,
}

impl<'a> VulkanCommandBufferBuilder<'a> {
    pub fn new<'b: 'a>(
        device: &'b VulkanDevice,
        command_buffer: &'b mut VulkanCommandBuffer,
        image_map: &HashMap<u32, VulkanOwnedImage>,
    ) -> Result<Self, VulkanCommandBufferError> {
        if device.id() != command_buffer.device_id {
            return Err(VulkanCommandBufferError::DeviceMismatch);
        }

        command_buffer.reset(device)?;

        for (img_id, image) in image_map.iter() {
            command_buffer.image_state.insert(
                *img_id,
                VulkanCommandBufferImageState {
                    last_pipeline_stage: vec![
                        VulkanPipelineStage::BottomOfPipe,
                    ],
                    last_layout: image.last_known_layout,
                    last_access: Vec::new(),
                },
            );
        }

        Ok(Self {
            device,
            command_buffer,
            state: VulkanCommandBufferState::NotStarted,
            current_renderpass_commands: Vec::new(),
            bound_pipeline: None,
            descriptor_set_images: HashSet::new(),
        })
    }

    pub fn begin(&mut self) -> Result<(), VulkanCommandBufferError> {
        if self.state != VulkanCommandBufferState::NotStarted {
            return Err(VulkanCommandBufferError::IncorrectState(self.state));
        }

        let begin_info = ash::vk::CommandBufferBeginInfo::default()
            .flags(ash::vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);

        unsafe {
            self.device
                .raw_device()
                .begin_command_buffer(
                    self.command_buffer.raw_command_buffer,
                    &begin_info,
                )
                .map_err(|err| VulkanCommandBufferError::RawBeginError(err))?;
        }

        self.state = VulkanCommandBufferState::Normal;

        Ok(())
    }

    pub fn end(&mut self) -> Result<(), VulkanCommandBufferError> {
        if self.state == VulkanCommandBufferState::NotStarted {
            return Err(VulkanCommandBufferError::IncorrectState(self.state));
        }

        unsafe {
            self.device
                .raw_device()
                .end_command_buffer(self.command_buffer.raw_command_buffer)
                .map_err(|err| VulkanCommandBufferError::RawEndError(err))?
        };

        self.state = VulkanCommandBufferState::Ended;

        Ok(())
    }

    pub fn begin_renderpass<'b: 'a>(
        &mut self,
        renderpass: &'b VulkanRenderpass,
        framebuffer: &'b VulkanFramebuffer,
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

        self.current_renderpass_commands =
            vec![VulkanCommand::BeginRenderpass(renderpass, framebuffer)];

        self.state = VulkanCommandBufferState::Renderpass;

        Ok(())
    }

    fn begin_renderpass_backtrack<'b: 'a>(
        &mut self,
        renderpass: &'b VulkanRenderpass,
        framebuffer: &'b VulkanFramebuffer,
    ) {
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
                self.command_buffer.raw_command_buffer,
                &begin_info,
                ash::vk::SubpassContents::INLINE,
            )
        };
    }

    pub fn end_renderpass(
        &mut self,
        image_map: &HashMap<u32, VulkanOwnedImage>,
    ) -> Result<(), VulkanCommandBufferError> {
        if self.state != VulkanCommandBufferState::Renderpass {
            return Err(VulkanCommandBufferError::IncorrectState(self.state));
        }

        for image_id in self.descriptor_set_images.iter() {
            let (access, layout, stage) = self.get_image_state(*image_id);
            let image = image_map
                .get(image_id)
                .ok_or(VulkanCommandBufferError::ImageNotFound)?;
            self.pipeline_barrier(
                stage,
                &[
                    VulkanPipelineStage::FragmentShader,
                    VulkanPipelineStage::VertexShader,
                ],
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
            self.command_buffer.image_state.insert(
                *image_id,
                VulkanCommandBufferImageState {
                    last_pipeline_stage: vec![
                        VulkanPipelineStage::FragmentShader,
                        VulkanPipelineStage::VertexShader,
                    ],
                    last_layout: VulkanImageLayout::ShaderReadOnlyOptimal,
                    last_access: vec![VulkanAccess::ShaderRead],
                },
            );
        }

        self.pipeline_barrier(
            &[VulkanPipelineStage::BottomOfPipe],
            &[VulkanPipelineStage::TopOfPipe],
            &[],
            &[],
            &[],
        )?;

        let commands = self.current_renderpass_commands.clone();
        for command in commands.into_iter() {
            match command {
                VulkanCommand::BeginRenderpass(renderpass, framebuffer) => {
                    self.begin_renderpass_backtrack(renderpass, framebuffer);
                },
                VulkanCommand::SetViewport(dimensions) => {
                    self.set_viewports_backtrack(dimensions);
                },
                VulkanCommand::SetScissor(dimensions, offset) => {
                    self.set_scissor_backtrack(dimensions, offset);
                },
                VulkanCommand::BindGraphicsPipeline(pipeline) => {
                    self.bind_graphics_pipeline_backtrack(pipeline);
                },
                VulkanCommand::Draw(vc, ic, fv, fi) => {
                    self.draw_backtrack(vc, ic, fv, fi);
                },
                VulkanCommand::DrawIndexed(idc, inc, fid, vo, fin) => {
                    self.draw_indexed_backtrack(idc, inc, fid, vo, fin);
                },
                VulkanCommand::BindVertexBuffer(buffer) => {
                    self.bind_vertex_buffer_backtrack(buffer);
                },
                VulkanCommand::BindIndexBuffer(buffer) => {
                    self.bind_index_buffer_backtrack(buffer);
                },
                VulkanCommand::BindDescriptorSet(set, idx) => {
                    self.bind_descriptor_set_backtrack(set, idx)?;
                },
                _ => return Err(
                    VulkanCommandBufferError::IllegalCommandInsiderRenderpass,
                ),
            };
        }

        unsafe {
            self.device
                .raw_device()
                .cmd_end_render_pass(self.command_buffer.raw_command_buffer);
        }

        self.descriptor_set_images.clear();
        self.state = VulkanCommandBufferState::Normal;

        Ok(())
    }

    pub fn set_viewports(
        &mut self,
        dimensions: Vec2u,
    ) -> Result<(), VulkanCommandBufferError> {
        if self.state != VulkanCommandBufferState::Renderpass {
            return Err(VulkanCommandBufferError::IncorrectState(self.state));
        }

        self.current_renderpass_commands
            .push(VulkanCommand::SetViewport(dimensions));

        Ok(())
    }

    fn set_viewports_backtrack(&mut self, dimensions: Vec2u) {
        let viewports = [ash::vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: dimensions.x as f32,
            height: dimensions.y as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];

        unsafe {
            self.device.raw_device().cmd_set_viewport(
                self.command_buffer.raw_command_buffer,
                0,
                &viewports,
            )
        };
    }

    pub fn set_scissor(
        &mut self,
        dimensions: Vec2u,
        offset: Vec2i,
    ) -> Result<(), VulkanCommandBufferError> {
        if self.state != VulkanCommandBufferState::Renderpass {
            return Err(VulkanCommandBufferError::IncorrectState(self.state));
        }

        self.current_renderpass_commands
            .push(VulkanCommand::SetScissor(dimensions, offset));

        Ok(())
    }

    fn set_scissor_backtrack(&mut self, dimensions: Vec2u, offset: Vec2i) {
        let scissors = [raw_rect_2d(dimensions, offset)];
        unsafe {
            self.device.raw_device().cmd_set_scissor(
                self.command_buffer.raw_command_buffer,
                0,
                &scissors,
            )
        };
    }

    pub fn bind_graphics_pipeline<'b: 'a>(
        &mut self,
        pipeline: &'b VulkanGraphicsPipeline,
    ) -> Result<(), VulkanCommandBufferError> {
        if self.state != VulkanCommandBufferState::Renderpass {
            return Err(VulkanCommandBufferError::IncorrectState(self.state));
        }

        if self.device.id() != pipeline.device_id() {
            return Err(VulkanCommandBufferError::DeviceMismatch);
        }

        self.current_renderpass_commands
            .push(VulkanCommand::BindGraphicsPipeline(pipeline));

        Ok(())
    }

    fn bind_graphics_pipeline_backtrack<'b: 'a>(
        &mut self,
        pipeline: &'b VulkanGraphicsPipeline,
    ) {
        unsafe {
            self.device.raw_device().cmd_bind_pipeline(
                self.command_buffer.raw_command_buffer,
                ash::vk::PipelineBindPoint::GRAPHICS,
                *pipeline.get_pipeline_raw(),
            )
        };

        self.bound_pipeline = Some(pipeline);
    }

    pub fn draw(
        &mut self,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) -> Result<(), VulkanCommandBufferError> {
        if self.state != VulkanCommandBufferState::Renderpass {
            return Err(VulkanCommandBufferError::IncorrectState(self.state));
        }

        self.current_renderpass_commands.push(VulkanCommand::Draw(
            vertex_count,
            instance_count,
            first_vertex,
            first_instance,
        ));

        Ok(())
    }

    fn draw_backtrack(
        &mut self,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) {
        unsafe {
            self.device.raw_device().cmd_draw(
                self.command_buffer.raw_command_buffer,
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            )
        };
    }

    pub fn draw_indexed(
        &mut self,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) -> Result<(), VulkanCommandBufferError> {
        if self.state != VulkanCommandBufferState::Renderpass {
            return Err(VulkanCommandBufferError::IncorrectState(self.state));
        }

        self.current_renderpass_commands
            .push(VulkanCommand::DrawIndexed(
                index_count,
                instance_count,
                first_index,
                vertex_offset,
                first_instance,
            ));

        Ok(())
    }

    fn draw_indexed_backtrack(
        &mut self,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) {
        unsafe {
            self.device.raw_device().cmd_draw_indexed(
                self.command_buffer.raw_command_buffer,
                index_count,
                instance_count,
                first_index,
                vertex_offset,
                first_instance,
            );
        };
    }

    pub fn bind_vertex_buffer<'b: 'a>(
        &mut self,
        buffer: &'b VulkanBuffer,
    ) -> Result<(), VulkanCommandBufferError> {
        if self.state != VulkanCommandBufferState::Renderpass {
            return Err(VulkanCommandBufferError::IncorrectState(self.state));
        }

        self.current_renderpass_commands
            .push(VulkanCommand::BindVertexBuffer(buffer));

        Ok(())
    }

    fn bind_vertex_buffer_backtrack(&mut self, buffer: &VulkanBuffer) {
        unsafe {
            self.device.raw_device().cmd_bind_vertex_buffers(
                self.command_buffer.raw_command_buffer,
                0,
                &[*buffer.get_buffer_raw()],
                &[0],
            )
        };
    }

    pub fn bind_index_buffer<'b: 'a>(
        &mut self,
        buffer: &'b VulkanBuffer,
    ) -> Result<(), VulkanCommandBufferError> {
        if self.state != VulkanCommandBufferState::Renderpass {
            return Err(VulkanCommandBufferError::IncorrectState(self.state));
        }

        self.current_renderpass_commands
            .push(VulkanCommand::BindIndexBuffer(buffer));

        Ok(())
    }

    fn bind_index_buffer_backtrack<'b: 'a>(
        &mut self,
        buffer: &'b VulkanBuffer,
    ) {
        unsafe {
            self.device.raw_device().cmd_bind_index_buffer(
                self.command_buffer.raw_command_buffer,
                *buffer.get_buffer_raw(),
                0,
                ash::vk::IndexType::UINT32,
            )
        };
    }

    pub fn bind_descriptor_set<'b: 'a>(
        &mut self,
        descriptor_set: &'b VulkanDescriptorSet,
        set_index: u32,
    ) -> Result<(), VulkanCommandBufferError> {
        if self.state != VulkanCommandBufferState::Renderpass {
            return Err(VulkanCommandBufferError::IncorrectState(self.state));
        }

        if self.device.id() != descriptor_set.device_id() {
            return Err(VulkanCommandBufferError::DeviceMismatch);
        }

        for image_id in descriptor_set.bound_image_ids().iter() {
            self.descriptor_set_images.insert(*image_id);
        }

        self.current_renderpass_commands
            .push(VulkanCommand::BindDescriptorSet(descriptor_set, set_index));

        Ok(())
    }

    fn bind_descriptor_set_backtrack(
        &mut self,
        descriptor_set: &VulkanDescriptorSet,
        set_index: u32,
    ) -> Result<(), VulkanCommandBufferError> {
        let pipeline = self
            .bound_pipeline
            .ok_or(VulkanCommandBufferError::PipelineNotBound)?;

        unsafe {
            self.device.raw_device().cmd_bind_descriptor_sets(
                self.command_buffer.raw_command_buffer,
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
        if self.state != VulkanCommandBufferState::Normal {
            return Err(VulkanCommandBufferError::IncorrectState(self.state));
        }

        if self.device.id() != buffer.device_id()
            || self.device.id() != image.device_id()
        {
            return Err(VulkanCommandBufferError::DeviceMismatch);
        }

        let (access, layout, stage) = self.get_image_state(image.id());

        self.pipeline_barrier(
            stage,
            &[VulkanPipelineStage::Transfer],
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
            self.device.raw_device().cmd_copy_buffer_to_image(
                self.command_buffer.raw_command_buffer,
                *buffer.get_buffer_raw(),
                *image.raw_image(),
                ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[buffer_image_copy],
            )
        };

        self.command_buffer.image_state.insert(
            image.id(),
            VulkanCommandBufferImageState {
                last_pipeline_stage: vec![VulkanPipelineStage::Transfer],
                last_layout: VulkanImageLayout::TransferDstOptimal,
                last_access: vec![VulkanAccess::TransferWrite],
            },
        );

        Ok(())
    }

    fn pipeline_barrier(
        &self,
        src_stage: &[VulkanPipelineStage],
        dst_stage: &[VulkanPipelineStage],
        memory_barriers: &[VulkanMemoryBarrier],
        buffer_memory_barriers: &[VulkanBufferMemoryBarrier],
        image_memory_barriers: &[VulkanImageMemoryBarrier],
    ) -> Result<(), VulkanCommandBufferError> {
        unsafe {
            self.device.raw_device().cmd_pipeline_barrier(
                self.command_buffer.raw_command_buffer,
                VulkanPipelineStage::raw_combined_stage(src_stage),
                VulkanPipelineStage::raw_combined_stage(dst_stage),
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

    fn get_image_state(
        &self,
        image_id: u32,
    ) -> (&[VulkanAccess], VulkanImageLayout, &[VulkanPipelineStage]) {
        match self.command_buffer.image_state.get(&image_id) {
            Some(image_state) => (
                image_state.last_access.as_slice(),
                image_state.last_layout,
                &image_state.last_pipeline_stage,
            ),
            None => (&[] as &[VulkanAccess], VulkanImageLayout::Undefined, &[
                VulkanPipelineStage::BottomOfPipe,
            ]),
        }
    }
}
