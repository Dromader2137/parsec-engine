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
        utils::{raw_extent_2d, raw_rect_2d},
    },
    math::{ivec::Vec2i, uvec::Vec2u},
};

pub struct VulkanCommandPool {
    raw_command_pool: ash::vk::CommandPool,
}

#[derive(Debug, thiserror::Error)]
pub enum VulkanCommandPoolError {
    #[error("Failed to create command pool: {0}")]
    CreationError(ash::vk::Result),
}

impl VulkanCommandPool {
    pub fn new(
        physical_device: &VulkanPhysicalDevice,
        device: &VulkanDevice,
    ) -> Result<VulkanCommandPool, VulkanCommandPoolError> {
        let pool_info = ash::vk::CommandPoolCreateInfo::default()
            .flags(ash::vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(physical_device.queue_family_index());

        let raw_command_pool = unsafe {
            device
                .raw_device()
                .create_command_pool(&pool_info, None)
                .map_err(|err| VulkanCommandPoolError::CreationError(err))?
        };

        Ok(VulkanCommandPool { raw_command_pool })
    }

    pub fn destroy(&self, device: &VulkanDevice) {
        unsafe {
            device
                .raw_device()
                .destroy_command_pool(self.raw_command_pool, None)
        }
    }

    pub fn raw_command_pool(&self) -> &ash::vk::CommandPool {
        &self.raw_command_pool
    }
}

pub struct VulkanCommandBuffer {
    id: u32,
    image_state_change: Vec<(u32, VulkanCommandBufferImageState)>,
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
    #[error("Renderpass doesn't match")]
    RenderpassMismatch,
    #[error("Used image does not exist in the provided image map")]
    ImageNotFound,
    #[error("Command used in an incorrect command buffer state: {0:?}")]
    IncorrectPhase(VulkanCommandBufferPhase),
    #[error("Pipeline not bound")]
    PipelineNotBound,
    #[error("Invalid viewport dimensions: {0:?}")]
    InvalidViewportDimensions(Vec2u),
    #[error("Invalid scissor dimensions: {0:?}")]
    InvalidScissorDimensions(Vec2u),
    #[error("Vertex buffer not bound")]
    VertexBufferNotBound,
    #[error("Drawcall vertex count can't be zero")]
    InvalidVertexCount,
    #[error(
        "Drawcall vertex count + first vertex can't escape the vertex buffer"
    )]
    VertexBufferOverflow,
    #[error("Index buffer not bound")]
    IndexBufferNotBound,
    #[error("Drawcall index count + first index can't escape the index buffer")]
    IndexBufferOverflow,
    #[error("Buffers have to be the same size to copy: {0} -> {1}")]
    BufferSizeMismatch(u64, u64),
}

crate::create_counter! {COMMAND_BUFFER_ID_COUNTER}
impl VulkanCommandBuffer {
    pub fn new(
        device: &VulkanDevice,
        command_pool: &VulkanCommandPool,
    ) -> Result<Self, VulkanCommandBufferError> {
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
            image_state_change: Vec::new(),
            raw_command_buffer: command_buffer,
        })
    }

    pub fn reset(
        &mut self,
        device: &VulkanDevice,
    ) -> Result<(), VulkanCommandBufferError> {
        self.image_state_change.clear();

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

    pub fn image_state(&self) -> &[(u32, VulkanCommandBufferImageState)] {
        &self.image_state_change
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum VulkanCommandBufferPhase {
    #[default]
    NotStarted = 0,
    Normal = 1,
    Renderpass = 2,
    Ended = 3,
}

#[derive(Default)]
struct CommandBufferState<'a> {
    phase: VulkanCommandBufferPhase,
    bound_pipeline: Option<&'a VulkanGraphicsPipeline>,
    bound_index_buffer: Option<&'a VulkanBuffer>,
    bound_vertex_buffer: Option<&'a VulkanBuffer>,
    bound_descriptor_sets: HashMap<u32, &'a VulkanDescriptorSet>,
    current_renderpass: Option<&'a VulkanRenderpass>,
    current_framebuffer: Option<&'a VulkanFramebuffer>,
    image_properties: HashMap<u32, VulkanCommandBufferImageState>,
}

impl CommandBufferState<'_> {
    fn reset(&mut self) {
        self.phase = VulkanCommandBufferPhase::NotStarted;
        self.current_renderpass = None;
        self.current_framebuffer = None;
        self.bound_pipeline = None;
        self.bound_vertex_buffer = None;
        self.bound_index_buffer = None;
        self.bound_descriptor_sets.clear();
    }

    fn reset_renderpass(&mut self) {
        self.phase = VulkanCommandBufferPhase::Normal;
        self.current_renderpass = None;
        self.current_framebuffer = None;
        self.bound_pipeline = None;
        self.bound_vertex_buffer = None;
        self.bound_index_buffer = None;
        self.bound_descriptor_sets.clear();
    }
}

#[derive(Debug, Clone)]
pub struct VulkanCommandBufferImageState {
    last_pipeline_stage: Vec<VulkanPipelineStage>,
    pub last_layout: VulkanImageLayout,
    last_access: Vec<VulkanAccess>,
}

#[derive(Default)]
pub struct RenderpassDependency {
    images: Vec<u32>,
}

#[derive(Clone)]
#[allow(unused)]
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
    CopyBufferToImage(&'a VulkanBuffer, &'a Box<dyn VulkanImage>),
    CopyBufferToBuffer(&'a VulkanBuffer, &'a VulkanBuffer),
}

pub struct VulkanCommandBufferBuilder<'a> {
    commands: Vec<VulkanCommand<'a>>,
    state: CommandBufferState<'a>,
    renderpass_dependencies: Vec<RenderpassDependency>,
    images: &'a HashMap<u32, Box<dyn VulkanImage>>,
}

impl<'a> VulkanCommandBufferBuilder<'a> {
    pub fn new<'b: 'a>(
        image_map: &'b HashMap<u32, Box<dyn VulkanImage>>,
    ) -> Result<Self, VulkanCommandBufferError> {
        let mut state = CommandBufferState::default();

        for (img_id, image) in image_map.iter() {
            state.image_properties.insert(
                *img_id,
                VulkanCommandBufferImageState {
                    last_pipeline_stage: vec![
                        VulkanPipelineStage::BottomOfPipe,
                    ],
                    last_layout: image.get_layout(),
                    last_access: Vec::new(),
                },
            );
        }

        Ok(Self {
            commands: Vec::new(),
            state,
            renderpass_dependencies: Vec::new(),
            images: image_map,
        })
    }

    pub fn build(
        &mut self,
        device: &VulkanDevice,
        command_buffer: &mut VulkanCommandBuffer,
    ) -> Result<(), VulkanCommandBufferError> {
        let mut renderpass_counter = 0;
        command_buffer.reset(device)?;
        self.state.reset();

        for command in self.commands.iter().cloned() {
            match command {
                VulkanCommand::Begin => {
                    let begin_info = ash::vk::CommandBufferBeginInfo::default()
                        .flags(
                            ash::vk::CommandBufferUsageFlags::SIMULTANEOUS_USE,
                        );

                    unsafe {
                        device
                            .raw_device()
                            .begin_command_buffer(
                                command_buffer.raw_command_buffer,
                                &begin_info,
                            )
                            .map_err(|err| {
                                VulkanCommandBufferError::RawBeginError(err)
                            })?;
                    }
                },
                VulkanCommand::End => {
                    unsafe {
                        device
                            .raw_device()
                            .end_command_buffer(
                                command_buffer.raw_command_buffer,
                            )
                            .map_err(|err| {
                                VulkanCommandBufferError::RawEndError(err)
                            })?
                    };
                },
                VulkanCommand::BeginRenderpass(renderpass, framebuffer) => {
                    let renderpass_dependencies =
                        &self.renderpass_dependencies[renderpass_counter];

                    for image_id in renderpass_dependencies.images.iter() {
                        let (access, layout, stage) =
                            self.get_image_state(*image_id);
                        let image = self
                            .images
                            .get(image_id)
                            .ok_or(VulkanCommandBufferError::ImageNotFound)?;
                        self.pipeline_barrier(
                            device,
                            command_buffer,
                            stage,
                            &[
                                VulkanPipelineStage::VertexShader,
                                VulkanPipelineStage::FragmentShader,
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
                        self.state.image_properties.insert(
                            *image_id,
                            VulkanCommandBufferImageState {
                                last_pipeline_stage: vec![
                                    VulkanPipelineStage::FragmentShader,
                                ],
                                last_layout:
                                    VulkanImageLayout::ShaderReadOnlyOptimal,
                                last_access: vec![VulkanAccess::ShaderRead],
                            },
                        );
                    }

                    let clear_values = renderpass
                        .clear_values()
                        .iter()
                        .map(|v| v.raw_clear_value())
                        .collect::<Vec<_>>();

                    let begin_info = ash::vk::RenderPassBeginInfo::default()
                        .render_pass(*renderpass.raw_handle())
                        .framebuffer(*framebuffer.raw_handle())
                        .render_area(raw_rect_2d(
                            framebuffer.dimensions(),
                            Vec2i::ZERO,
                        ))
                        .clear_values(&clear_values);

                    unsafe {
                        device.raw_device().cmd_begin_render_pass(
                            command_buffer.raw_command_buffer,
                            &begin_info,
                            ash::vk::SubpassContents::INLINE,
                        )
                    };

                    self.state.current_renderpass = Some(renderpass);
                    self.state.current_framebuffer = Some(framebuffer);
                },
                VulkanCommand::EndRenderpass => {
                    unsafe {
                        device.raw_device().cmd_end_render_pass(
                            command_buffer.raw_command_buffer,
                        );
                    }

                    let framebuffer = self.state.current_framebuffer.expect(
                        "Framebuffer has to be bound at end_renderpass",
                    );
                    let renderpass = self
                        .state
                        .current_renderpass
                        .expect("Renderpass has to be bound at end_renderpass");

                    let depth_idx =
                        renderpass.depth_attachment_id().unwrap_or(u32::MAX)
                            as usize;
                    for (idx, image_id) in
                        framebuffer.attached_images_id().iter().enumerate()
                    {
                        if !self.images.contains_key(image_id) {
                            return Err(
                                VulkanCommandBufferError::ImageNotFound,
                            );
                        }
                        if idx == depth_idx {
                            self.state.image_properties.insert(
                    *image_id,
                    VulkanCommandBufferImageState {
                        last_pipeline_stage: vec![
                            VulkanPipelineStage::LateFragmentTests,
                        ],
                        last_layout:
                            VulkanImageLayout::DepthStencilAttachmentOptimal,
                        last_access: vec![VulkanAccess::DepthAttachmentWrite],
                    },
                );
                        } else {
                            self.state.image_properties.insert(
                                *image_id,
                                VulkanCommandBufferImageState {
                                    last_pipeline_stage: vec![
                            VulkanPipelineStage::ColorAttachmentOutput,
                        ],
                                    last_layout:
                                        VulkanImageLayout::PresentSrcKHR,
                                    last_access: vec![
                                        VulkanAccess::ColorAttachmentWrite,
                                    ],
                                },
                            );
                        }
                    }

                    self.state.reset();
                    renderpass_counter += 1;
                },
                VulkanCommand::SetViewport(dimensions) => {
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
                            command_buffer.raw_command_buffer,
                            0,
                            &viewports,
                        )
                    };
                },
                VulkanCommand::SetScissor(dimensions, offset) => {
                    let scissors = [raw_rect_2d(dimensions, offset)];
                    unsafe {
                        device.raw_device().cmd_set_scissor(
                            command_buffer.raw_command_buffer,
                            0,
                            &scissors,
                        )
                    };
                },
                VulkanCommand::BindGraphicsPipeline(pipeline) => {
                    unsafe {
                        device.raw_device().cmd_bind_pipeline(
                            command_buffer.raw_command_buffer,
                            ash::vk::PipelineBindPoint::GRAPHICS,
                            *pipeline.get_pipeline_raw(),
                        )
                    };

                    self.state.bound_pipeline = Some(pipeline);
                },
                VulkanCommand::Draw(
                    vertex_count,
                    instance_count,
                    first_vertex,
                    first_instance,
                ) => {
                    unsafe {
                        device.raw_device().cmd_draw(
                            command_buffer.raw_command_buffer,
                            vertex_count,
                            instance_count,
                            first_vertex,
                            first_instance,
                        )
                    };
                },
                VulkanCommand::DrawIndexed(
                    index_count,
                    instance_count,
                    first_index,
                    vertex_offset,
                    first_instance,
                ) => {
                    unsafe {
                        device.raw_device().cmd_draw_indexed(
                            command_buffer.raw_command_buffer,
                            index_count,
                            instance_count,
                            first_index,
                            vertex_offset,
                            first_instance,
                        );
                    };
                },
                VulkanCommand::BindVertexBuffer(buffer) => {
                    unsafe {
                        device.raw_device().cmd_bind_vertex_buffers(
                            command_buffer.raw_command_buffer,
                            0,
                            &[*buffer.get_buffer_raw()],
                            &[0],
                        )
                    };
                },
                VulkanCommand::BindIndexBuffer(buffer) => {
                    unsafe {
                        device.raw_device().cmd_bind_index_buffer(
                            command_buffer.raw_command_buffer,
                            *buffer.get_buffer_raw(),
                            0,
                            ash::vk::IndexType::UINT32,
                        )
                    };
                },
                VulkanCommand::BindDescriptorSet(set, binding_idx) => {
                    let pipeline = self
                        .state
                        .bound_pipeline
                        .ok_or(VulkanCommandBufferError::PipelineNotBound)?;

                    unsafe {
                        device.raw_device().cmd_bind_descriptor_sets(
                            command_buffer.raw_command_buffer,
                            ash::vk::PipelineBindPoint::GRAPHICS,
                            *pipeline.get_layout_raw(),
                            binding_idx,
                            &[*set.raw_descriptor_set()],
                            &[],
                        );
                    };
                },
                VulkanCommand::CopyBufferToImage(buffer, image) => {
                    let (access, layout, stage) =
                        self.get_image_state(image.id());

                    self.pipeline_barrier(
                        device,
                        command_buffer,
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
                        .image_extent(
                            raw_extent_2d(image.extent().raw_size()).into(),
                        )
                        .image_subresource(
                            ash::vk::ImageSubresourceLayers::default()
                                .layer_count(1)
                                .aspect_mask(image.aspect().raw_image_aspect()),
                        );

                    unsafe {
                        device.raw_device().cmd_copy_buffer_to_image(
                            command_buffer.raw_command_buffer,
                            *buffer.get_buffer_raw(),
                            *image.raw_image(),
                            ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                            &[buffer_image_copy],
                        )
                    };

                    self.state.image_properties.insert(
                        image.id(),
                        VulkanCommandBufferImageState {
                            last_pipeline_stage: vec![
                                VulkanPipelineStage::Transfer,
                            ],
                            last_layout: VulkanImageLayout::TransferDstOptimal,
                            last_access: vec![VulkanAccess::TransferWrite],
                        },
                    );
                },
                VulkanCommand::CopyBufferToBuffer(src_buffer, dst_buffer) => {
                    let buffer_copy =
                        ash::vk::BufferCopy::default().size(src_buffer.size());

                    unsafe {
                        device.raw_device().cmd_copy_buffer(
                            command_buffer.raw_command_buffer,
                            *src_buffer.get_buffer_raw(),
                            *dst_buffer.get_buffer_raw(),
                            &[buffer_copy],
                        )
                    };
                },
            }
        }

        for (id, image_state) in self.state.image_properties.iter() {
            command_buffer
                .image_state_change
                .push((*id, image_state.clone()));
        }

        Ok(())
    }

    pub fn begin(&mut self) -> Result<(), VulkanCommandBufferError> {
        self.check_phase(VulkanCommandBufferPhase::NotStarted)?;

        self.commands.push(VulkanCommand::Begin);
        self.state.phase = VulkanCommandBufferPhase::Normal;

        Ok(())
    }

    pub fn end(&mut self) -> Result<(), VulkanCommandBufferError> {
        self.check_phase(VulkanCommandBufferPhase::Normal)?;

        self.commands.push(VulkanCommand::End);
        self.state.phase = VulkanCommandBufferPhase::Ended;

        Ok(())
    }

    pub fn begin_renderpass<'b: 'a>(
        &mut self,
        renderpass: &'b VulkanRenderpass,
        framebuffer: &'b VulkanFramebuffer,
    ) -> Result<(), VulkanCommandBufferError> {
        self.check_phase(VulkanCommandBufferPhase::Normal)?;

        if renderpass.id() != framebuffer.renderpass_id() {
            return Err(VulkanCommandBufferError::RenderpassMismatch);
        }

        self.commands
            .push(VulkanCommand::BeginRenderpass(renderpass, framebuffer));
        self.renderpass_dependencies
            .push(RenderpassDependency::default());
        self.state.phase = VulkanCommandBufferPhase::Renderpass;

        Ok(())
    }

    pub fn end_renderpass(&mut self) -> Result<(), VulkanCommandBufferError> {
        self.check_phase(VulkanCommandBufferPhase::Renderpass)?;

        self.commands.push(VulkanCommand::EndRenderpass);

        self.state.reset_renderpass();

        Ok(())
    }

    pub fn set_viewports(
        &mut self,
        dimensions: Vec2u,
    ) -> Result<(), VulkanCommandBufferError> {
        self.check_phase(VulkanCommandBufferPhase::Renderpass)?;
        if dimensions.x == 0 || dimensions.y == 0 {
            return Err(VulkanCommandBufferError::InvalidViewportDimensions(
                dimensions,
            ));
        }

        self.commands.push(VulkanCommand::SetViewport(dimensions));

        Ok(())
    }

    pub fn set_scissor(
        &mut self,
        dimensions: Vec2u,
        offset: Vec2i,
    ) -> Result<(), VulkanCommandBufferError> {
        self.check_phase(VulkanCommandBufferPhase::Renderpass)?;
        if dimensions.x == 0 || dimensions.y == 0 {
            return Err(VulkanCommandBufferError::InvalidScissorDimensions(
                dimensions,
            ));
        }

        self.commands
            .push(VulkanCommand::SetScissor(dimensions, offset));

        Ok(())
    }

    pub fn bind_graphics_pipeline<'b: 'a>(
        &mut self,
        pipeline: &'b VulkanGraphicsPipeline,
    ) -> Result<(), VulkanCommandBufferError> {
        self.check_phase(VulkanCommandBufferPhase::Renderpass)?;

        self.commands
            .push(VulkanCommand::BindGraphicsPipeline(pipeline));
        self.state.bound_pipeline = Some(pipeline);
        self.state.bound_descriptor_sets.clear();

        Ok(())
    }

    pub fn draw(
        &mut self,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) -> Result<(), VulkanCommandBufferError> {
        self.check_phase(VulkanCommandBufferPhase::Renderpass)?;
        let _ = self
            .state
            .bound_pipeline
            .ok_or(VulkanCommandBufferError::PipelineNotBound)?;
        let vb = self
            .state
            .bound_vertex_buffer
            .ok_or(VulkanCommandBufferError::VertexBufferNotBound)?;
        if vertex_count == 0 {
            return Err(VulkanCommandBufferError::InvalidVertexCount);
        }
        if first_vertex + vertex_count > vb.len() {
            return Err(VulkanCommandBufferError::VertexBufferOverflow);
        }

        self.commands.push(VulkanCommand::Draw(
            vertex_count,
            instance_count,
            first_vertex,
            first_instance,
        ));

        Ok(())
    }

    pub fn draw_indexed(
        &mut self,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) -> Result<(), VulkanCommandBufferError> {
        self.check_phase(VulkanCommandBufferPhase::Renderpass)?;
        let _ = self
            .state
            .bound_pipeline
            .ok_or(VulkanCommandBufferError::PipelineNotBound)?;
        let _ = self
            .state
            .bound_vertex_buffer
            .ok_or(VulkanCommandBufferError::VertexBufferNotBound)?;
        let ib = self
            .state
            .bound_index_buffer
            .ok_or(VulkanCommandBufferError::IndexBufferNotBound)?;
        if first_index + index_count > ib.len() {
            return Err(VulkanCommandBufferError::IndexBufferOverflow);
        }

        self.commands.push(VulkanCommand::DrawIndexed(
            index_count,
            instance_count,
            first_index,
            vertex_offset,
            first_instance,
        ));

        Ok(())
    }

    pub fn bind_vertex_buffer<'b: 'a>(
        &mut self,
        buffer: &'b VulkanBuffer,
    ) -> Result<(), VulkanCommandBufferError> {
        self.check_phase(VulkanCommandBufferPhase::Renderpass)?;

        self.commands.push(VulkanCommand::BindVertexBuffer(buffer));
        self.state.bound_vertex_buffer = Some(buffer);

        Ok(())
    }

    pub fn bind_index_buffer<'b: 'a>(
        &mut self,
        buffer: &'b VulkanBuffer,
    ) -> Result<(), VulkanCommandBufferError> {
        self.check_phase(VulkanCommandBufferPhase::Renderpass)?;

        self.commands.push(VulkanCommand::BindIndexBuffer(buffer));
        self.state.bound_index_buffer = Some(buffer);

        Ok(())
    }

    pub fn bind_descriptor_set<'b: 'a>(
        &mut self,
        descriptor_set: &'b VulkanDescriptorSet,
        set_index: u32,
    ) -> Result<(), VulkanCommandBufferError> {
        self.check_phase(VulkanCommandBufferPhase::Renderpass)?;

        let _ = self
            .state
            .bound_pipeline
            .ok_or(VulkanCommandBufferError::PipelineNotBound)?;

        for image_id in descriptor_set.bound_image_ids().iter() {
            self.renderpass_dependencies
                .last_mut()
                .expect("There has to be a renderpass dependency here!")
                .images
                .push(*image_id);
        }

        self.commands
            .push(VulkanCommand::BindDescriptorSet(descriptor_set, set_index));
        self.state
            .bound_descriptor_sets
            .insert(set_index, descriptor_set);

        Ok(())
    }

    pub fn copy_buffer_to_image<'b: 'a>(
        &mut self,
        buffer: &'b VulkanBuffer,
        image: &'b Box<dyn VulkanImage>,
    ) -> Result<(), VulkanCommandBufferError> {
        self.check_phase(VulkanCommandBufferPhase::Normal)?;

        self.commands
            .push(VulkanCommand::CopyBufferToImage(buffer, image));

        Ok(())
    }

    pub fn copy_buffer_to_buffer<'b: 'a>(
        &mut self,
        src: &'b VulkanBuffer,
        dst: &'b VulkanBuffer,
    ) -> Result<(), VulkanCommandBufferError> {
        self.check_phase(VulkanCommandBufferPhase::Normal)?;

        if src.size() != dst.size() {
            return Err(VulkanCommandBufferError::BufferSizeMismatch(
                src.size(),
                dst.size(),
            ));
        }

        self.commands
            .push(VulkanCommand::CopyBufferToBuffer(src, dst));

        Ok(())
    }

    fn pipeline_barrier(
        &self,
        device: &VulkanDevice,
        command_buffer: &VulkanCommandBuffer,
        src_stage: &[VulkanPipelineStage],
        dst_stage: &[VulkanPipelineStage],
        memory_barriers: &[VulkanMemoryBarrier],
        buffer_memory_barriers: &[VulkanBufferMemoryBarrier],
        image_memory_barriers: &[VulkanImageMemoryBarrier],
    ) -> Result<(), VulkanCommandBufferError> {
        unsafe {
            device.raw_device().cmd_pipeline_barrier(
                command_buffer.raw_command_buffer,
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
        match self.state.image_properties.get(&image_id) {
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

    fn check_phase(
        &self,
        correct_phase: VulkanCommandBufferPhase,
    ) -> Result<(), VulkanCommandBufferError> {
        if correct_phase != self.state.phase {
            return Err(VulkanCommandBufferError::IncorrectPhase(
                self.state.phase,
            ));
        }
        Ok(())
    }
}
