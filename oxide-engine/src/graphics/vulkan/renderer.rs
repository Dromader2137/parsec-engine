use std::{fmt::Debug, sync::Arc, time::Duration};

use vulkano::{
    pipeline::graphics::vertex_input::{Vertex, VertexDefinition},
    sync::GpuFuture,
};

use crate::{error::EngineError, graphics::{renderer::Renderer, window::WindowWrapper}};

use super::context::VulkanContext;

type Fence = Option<
    Arc<
        vulkano::sync::future::FenceSignalFuture<
            vulkano::swapchain::PresentFuture<
                vulkano::command_buffer::CommandBufferExecFuture<
                    vulkano::sync::future::JoinFuture<
                        Box<dyn GpuFuture>,
                        vulkano::swapchain::SwapchainAcquireFuture,
                    >,
                >,
            >,
        >,
    >,
>;

pub struct VulkanRenderer {
    render_pass: Arc<vulkano::render_pass::RenderPass>,
    swapchain: Arc<vulkano::swapchain::Swapchain>,
    images: Vec<Arc<vulkano::image::Image>>,
    framebuffers: Vec<Arc<vulkano::render_pass::Framebuffer>>,
    viewport: vulkano::pipeline::graphics::viewport::Viewport,
    vertex_buffer: vulkano::buffer::Subbuffer<[VertexData]>,
    pipeline: Arc<vulkano::pipeline::GraphicsPipeline>,
    command_buffers: Vec<Arc<vulkano::command_buffer::PrimaryAutoCommandBuffer>>,
    fences: Vec<Fence>,
    previous_fence: u32,
    recreate: bool
}

#[derive(Debug, Clone)]
pub enum VulkanRendererError {
    SurfaceCapabilitiesError(String),
    SurfaceFormatsError(String),
    SurfaceFormatNotFound,
    SwapchainError(String),
    RenderpassError(String),
    ImageViewError(String),
    FramebufferError(String),
    BufferError(String),
    ShaderModuleLoadError(String),
    ShaderEntryPointNotFound,
    VertexInputStateError(String),
    PipelineError(String),
    SubpassNotFound,
    CommandBufferError(String)
}

impl Debug for VulkanRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f
            .debug_struct("VulkanRenderer")
            .field("render_pass", &self.render_pass)
            .field("swapchain", &self.swapchain)
            .field("images", &self.images)
            .field("framebuffers", &self.framebuffers)
            .field("viewport", &self.viewport)
            .field("vertex_buffer", &self.vertex_buffer)
            .field("pipeline", &self.pipeline)
            .field("previous_fence", &self.previous_fence)
            .field("recreate", &self.recreate)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, vulkano::pipeline::graphics::vertex_input::Vertex, vulkano::buffer::BufferContents)]
#[repr(C)]
struct VertexData {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2],
}

mod test_vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r"
            #version 450

            layout(location = 0) in vec2 position;

            void main() {
                gl_Position = vec4(position, 0.5, 1.0);
            }
        ",
    }
}

mod test_fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: r"
            #version 450

            layout(location = 0) out vec4 color;

            void main() {
                color = vec4(1.0, 1.0, 1.0, 1.0);
            }
        ",
    }
}

impl VulkanRenderer {
    pub fn new(vulkan_context: &VulkanContext, window: &WindowWrapper) -> Result<VulkanRenderer, VulkanRendererError> {
        let caps = match vulkan_context
            .get_physical_device()
            .surface_capabilities(&vulkan_context.get_surface(), Default::default()) {
                Ok(val) => val,
                Err(err) => return Err(VulkanRendererError::SurfaceCapabilitiesError(err.to_string()))
        };

        let dimensions = window.get_physical_size();

        let image_formats = match vulkan_context
            .get_physical_device()
            .surface_formats(&vulkan_context.get_surface(), Default::default()) {
                Ok(val) => val,
                Err(err) => return Err(VulkanRendererError::SurfaceFormatsError(err.to_string()))
        };

        let image_format = match image_formats.first() {
            Some((format, _)) => *format,
            None => return Err(VulkanRendererError::SurfaceFormatNotFound)
        };

        let (swapchain, images) = match vulkano::swapchain::Swapchain::new(
            vulkan_context.get_device(),
            vulkan_context.get_surface(),
            vulkano::swapchain::SwapchainCreateInfo {
                min_image_count: caps.min_image_count,
                image_format,
                image_extent: dimensions.into(),
                image_usage: vulkano::image::ImageUsage::COLOR_ATTACHMENT
                    | vulkano::image::ImageUsage::TRANSFER_DST,
                ..Default::default()
            },
        ) {
            Ok(val) => val,
            Err(err) => return Err(VulkanRendererError::SwapchainError(err.to_string()))
        };

        let render_pass = match vulkano::single_pass_renderpass!(
            vulkan_context.get_device(),
            attachments: {
                color: {
                    format: swapchain.image_format(),
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
            },
            pass: {
                color: [color],
                depth_stencil: {},
            },
        ) {
            Ok(val) => val,
            Err(err) => return Err(VulkanRendererError::RenderpassError(err.to_string()))
        };

        let viewport = vulkano::pipeline::graphics::viewport::Viewport {
            offset: [0.0, 0.0],
            extent: dimensions.into(),
            depth_range: 0.0..=1.0,
        };

        let framebuffers = images
            .iter()
            .map(|image| {
                let view = match vulkano::image::view::ImageView::new_default(image.clone()) {
                    Ok(val) => val,
                    Err(err) => return Err(VulkanRendererError::ImageViewError(err.to_string()))
                };

                match vulkano::render_pass::Framebuffer::new(
                    render_pass.clone(),
                    vulkano::render_pass::FramebufferCreateInfo {
                        attachments: vec![view],
                        ..Default::default()
                    },
                ) {
                    Ok(val) => Ok(val),
                    Err(err) => Err(VulkanRendererError::FramebufferError(err.to_string()))
                }
            })
            .collect::<Result<Vec<_>, VulkanRendererError>>()?;

        let vertices: [VertexData; 3] = [
            VertexData {
                position: [0.0, 0.8],
            },
            VertexData {
                position: [-0.8, -0.8],
            },
            VertexData {
                position: [0.8, -0.8],
            },
        ];

        let vertex_buffer = match vulkano::buffer::Buffer::from_iter(
            vulkan_context.get_allocator(),
            vulkano::buffer::BufferCreateInfo {
                usage: vulkano::buffer::BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            vulkano::memory::allocator::AllocationCreateInfo {
                memory_type_filter: vulkano::memory::allocator::MemoryTypeFilter::PREFER_DEVICE
                    | vulkano::memory::allocator::MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices,
        ) {
            Ok(val) => val,
            Err(err) => return Err(VulkanRendererError::BufferError(err.to_string()))
        };

        let pipeline = {
            let vertex_shader = match test_vertex_shader::load(vulkan_context.get_device()) {
                Ok(shader_module) => {
                    match shader_module.entry_point("main") {
                        Some(val) => val,
                        None => return Err(VulkanRendererError::ShaderEntryPointNotFound)
                    }
                },
                Err(err) => return Err(VulkanRendererError::ShaderModuleLoadError(err.to_string()))
            };
            let fragment_shader = match test_fragment_shader::load(vulkan_context.get_device()) {
                Ok(shader_module) => {
                    match shader_module.entry_point("main") {
                        Some(val) => val,
                        None => return Err(VulkanRendererError::ShaderEntryPointNotFound)
                    }
                },
                Err(err) => return Err(VulkanRendererError::ShaderModuleLoadError(err.to_string()))
            };

            let vertex_input = match VertexData::per_vertex()
                .definition(&vertex_shader.info().input_interface) {
                    Ok(val) => val,
                    Err(err) => return Err(VulkanRendererError::VertexInputStateError(err.to_string()))
            };

            let stages = [
                vulkano::pipeline::PipelineShaderStageCreateInfo::new(vertex_shader),
                vulkano::pipeline::PipelineShaderStageCreateInfo::new(fragment_shader),
            ];

            let layout_create_info = match vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(vulkan_context.get_device()) {
                    Ok(val) => val,
                    Err(err) => return Err(VulkanRendererError::PipelineError(err.to_string()))
            };

            let layout = match vulkano::pipeline::PipelineLayout::new(
                vulkan_context.get_device(),
                layout_create_info
            ) {
                Ok(val) => val,
                Err(err) => return Err(VulkanRendererError::PipelineError(err.to_string()))
            };

            let subpass = match vulkano::render_pass::Subpass::from(render_pass.clone(), 0) {
                Some(val) => val,
                None => return Err(VulkanRendererError::SubpassNotFound)
            };

            match vulkano::pipeline::GraphicsPipeline::new(
                vulkan_context.get_device(),
                None,
                vulkano::pipeline::graphics::GraphicsPipelineCreateInfo {
                    stages: stages.into_iter().collect(),
                    vertex_input_state: Some(vertex_input),
                    input_assembly_state: Some(vulkano::pipeline::graphics::input_assembly::InputAssemblyState::default()),
                    viewport_state: Some(vulkano::pipeline::graphics::viewport::ViewportState { 
                        viewports: [viewport.clone()].into_iter().collect(), 
                        ..Default::default()
                    }),
                    multisample_state: Some(vulkano::pipeline::graphics::multisample::MultisampleState::default()),
                    color_blend_state: Some(vulkano::pipeline::graphics::color_blend::ColorBlendState::with_attachment_states(
                        subpass.num_color_attachments(), 
                        vulkano::pipeline::graphics::color_blend::ColorBlendAttachmentState::default()
                    )),
                    rasterization_state: Some(vulkano::pipeline::graphics::rasterization::RasterizationState::default()),
                    subpass: Some(subpass.into()),
                    ..vulkano::pipeline::graphics::GraphicsPipelineCreateInfo::layout(layout)
                },
            ) {
                Ok(val) => Ok(val),
                Err(err) => Err(VulkanRendererError::PipelineError(err.to_string()))
            }
        }?;

        let command_buffers = framebuffers.iter().map(|framebuffer| {
                let mut builder = match vulkano::command_buffer::AutoCommandBufferBuilder::primary(
                    vulkan_context.get_command_buffer_allocator().as_ref(),
                    vulkan_context.get_queue().queue_family_index(),
                    vulkano::command_buffer::CommandBufferUsage::MultipleSubmit,
                ) {
                    Ok(val) => val,
                    Err(err) => return Err(VulkanRendererError::CommandBufferError(err.to_string()))
                };

                if let Err(err) = builder
                    .begin_render_pass(
                        vulkano::command_buffer::RenderPassBeginInfo { 
                            clear_values: vec![
                                Some([0.0, 0.0, 0.0, 1.0].into())
                            ],
                            ..vulkano::command_buffer::RenderPassBeginInfo::framebuffer(framebuffer.clone())
                        },
                        vulkano::command_buffer::SubpassBeginInfo { 
                            contents: vulkano::command_buffer::SubpassContents::Inline, 
                            ..Default::default() 
                        }
                    ) {
                    return Err(VulkanRendererError::CommandBufferError(err.to_string()))
                };

                if let Err(err) = builder.bind_pipeline_graphics(pipeline.clone()) {
                    return Err(VulkanRendererError::CommandBufferError(err.to_string()))
                };
                
                if let Err(err) = builder.bind_vertex_buffers(0, vertex_buffer.clone()) {
                    return Err(VulkanRendererError::CommandBufferError(err.to_string()))
                };
                
                if let Err(err) = builder.draw(3, 1, 0, 0) {
                    return Err(VulkanRendererError::CommandBufferError(err.to_string()))
                };
                
                if let Err(err) = builder.end_render_pass(Default::default()) {
                    return Err(VulkanRendererError::CommandBufferError(err.to_string()))
                };

                match builder.build() {
                    Ok(val) => Ok(val),
                    Err(err) => Err(VulkanRendererError::CommandBufferError(err.to_string()))
                }
            }).collect::<Result<Vec<Arc<vulkano::command_buffer::PrimaryAutoCommandBuffer>>, VulkanRendererError>>()?;

        let fences = vec![None; images.len()];

        Ok(VulkanRenderer {
            render_pass,
            swapchain,
            images,
            framebuffers,
            viewport,
            vertex_buffer,
            pipeline,
            command_buffers,
            fences,
            previous_fence: 0,
            recreate: false
        })
    }

    fn resize(&mut self, vulkan_context: &VulkanContext, new_size: [u32; 2]) -> Result<(), VulkanRendererError> {
        let (swapchain, images) = match self.swapchain.recreate(
            vulkano::swapchain::SwapchainCreateInfo {
                image_extent: new_size,
                ..self.swapchain.create_info()
            }
        ) {
            Ok(val) => val,
            Err(err) => return Err(VulkanRendererError::SwapchainError(err.to_string()))
        };

        let framebuffers = images
            .iter()
            .map(|image| {
                let view = match vulkano::image::view::ImageView::new_default(image.clone()) {
                    Ok(val) => val,
                    Err(err) => return Err(VulkanRendererError::ImageViewError(err.to_string()))
                };

                match vulkano::render_pass::Framebuffer::new(
                    self.render_pass.clone(),
                    vulkano::render_pass::FramebufferCreateInfo {
                        attachments: vec![view],
                        ..Default::default()
                    },
                ) {
                    Ok(val) => Ok(val),
                    Err(err) => Err(VulkanRendererError::FramebufferError(err.to_string()))
                }
            })
            .collect::<Result<Vec<_>, VulkanRendererError>>()?;

        
        self.viewport.extent = [new_size[0] as f32, new_size[1] as f32];
        
        let pipeline = {
            let vertex_shader = match test_vertex_shader::load(vulkan_context.get_device()) {
                Ok(shader_module) => {
                    match shader_module.entry_point("main") {
                        Some(val) => val,
                        None => return Err(VulkanRendererError::ShaderEntryPointNotFound)
                    }
                },
                Err(err) => return Err(VulkanRendererError::ShaderModuleLoadError(err.to_string()))
            };
            let fragment_shader = match test_fragment_shader::load(vulkan_context.get_device()) {
                Ok(shader_module) => {
                    match shader_module.entry_point("main") {
                        Some(val) => val,
                        None => return Err(VulkanRendererError::ShaderEntryPointNotFound)
                    }
                },
                Err(err) => return Err(VulkanRendererError::ShaderModuleLoadError(err.to_string()))
            };

            let vertex_input = match VertexData::per_vertex()
                .definition(&vertex_shader.info().input_interface) {
                    Ok(val) => val,
                    Err(err) => return Err(VulkanRendererError::VertexInputStateError(err.to_string()))
            };

            let stages = [
                vulkano::pipeline::PipelineShaderStageCreateInfo::new(vertex_shader),
                vulkano::pipeline::PipelineShaderStageCreateInfo::new(fragment_shader),
            ];

            let layout_create_info = match vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(vulkan_context.get_device()) {
                    Ok(val) => val,
                    Err(err) => return Err(VulkanRendererError::PipelineError(err.to_string()))
            };

            let layout = match vulkano::pipeline::PipelineLayout::new(
                vulkan_context.get_device(),
                layout_create_info
            ) {
                Ok(val) => val,
                Err(err) => return Err(VulkanRendererError::PipelineError(err.to_string()))
            };

            let subpass = match vulkano::render_pass::Subpass::from(self.render_pass.clone(), 0) {
                Some(val) => val,
                None => return Err(VulkanRendererError::SubpassNotFound)
            };

            match vulkano::pipeline::GraphicsPipeline::new(
                vulkan_context.get_device(),
                None,
                vulkano::pipeline::graphics::GraphicsPipelineCreateInfo {
                    stages: stages.into_iter().collect(),
                    vertex_input_state: Some(vertex_input),
                    input_assembly_state: Some(vulkano::pipeline::graphics::input_assembly::InputAssemblyState::default()),
                    viewport_state: Some(vulkano::pipeline::graphics::viewport::ViewportState { 
                        viewports: [self.viewport.clone()].into_iter().collect(), 
                        ..Default::default()
                    }),
                    multisample_state: Some(vulkano::pipeline::graphics::multisample::MultisampleState::default()),
                    color_blend_state: Some(vulkano::pipeline::graphics::color_blend::ColorBlendState::with_attachment_states(
                        subpass.num_color_attachments(), 
                        vulkano::pipeline::graphics::color_blend::ColorBlendAttachmentState::default()
                    )),
                    rasterization_state: Some(vulkano::pipeline::graphics::rasterization::RasterizationState::default()),
                    subpass: Some(subpass.into()),
                    ..vulkano::pipeline::graphics::GraphicsPipelineCreateInfo::layout(layout)
                },
            ) {
                Ok(val) => Ok(val),
                Err(err) => Err(VulkanRendererError::PipelineError(err.to_string()))
            }
        }?;

        let command_buffers = framebuffers.iter().map(|framebuffer| {
                let mut builder = match vulkano::command_buffer::AutoCommandBufferBuilder::primary(
                    vulkan_context.get_command_buffer_allocator().as_ref(),
                    vulkan_context.get_queue().queue_family_index(),
                    vulkano::command_buffer::CommandBufferUsage::MultipleSubmit,
                ) {
                    Ok(val) => val,
                    Err(err) => return Err(VulkanRendererError::CommandBufferError(err.to_string()))
                };

                if let Err(err) = builder
                    .begin_render_pass(
                        vulkano::command_buffer::RenderPassBeginInfo { 
                            clear_values: vec![
                                Some([0.0, 0.0, 0.0, 1.0].into())
                            ],
                            ..vulkano::command_buffer::RenderPassBeginInfo::framebuffer(framebuffer.clone())
                        },
                        vulkano::command_buffer::SubpassBeginInfo { 
                            contents: vulkano::command_buffer::SubpassContents::Inline, 
                            ..Default::default() 
                        }
                    ) {
                    return Err(VulkanRendererError::CommandBufferError(err.to_string()))
                };

                if let Err(err) = builder.bind_pipeline_graphics(pipeline.clone()) {
                    return Err(VulkanRendererError::CommandBufferError(err.to_string()))
                };
                
                if let Err(err) = builder.bind_vertex_buffers(0, self.vertex_buffer.clone()) {
                    return Err(VulkanRendererError::CommandBufferError(err.to_string()))
                };
                
                if let Err(err) = builder.draw(3, 1, 0, 0) {
                    return Err(VulkanRendererError::CommandBufferError(err.to_string()))
                };
                
                if let Err(err) = builder.end_render_pass(Default::default()) {
                    return Err(VulkanRendererError::CommandBufferError(err.to_string()))
                };

                match builder.build() {
                    Ok(val) => Ok(val),
                    Err(err) => Err(VulkanRendererError::CommandBufferError(err.to_string()))
                }
            }).collect::<Result<Vec<Arc<vulkano::command_buffer::PrimaryAutoCommandBuffer>>, VulkanRendererError>>()?;

        self.swapchain = swapchain;
        self.command_buffers = command_buffers;
        self.images = images;
        self.framebuffers = framebuffers;
        self.pipeline = pipeline;
        self.recreate = false;

        Ok(())
    }
}

impl Renderer for VulkanRenderer {
    #[allow(clippy::arc_with_non_send_sync)]
    fn render(&mut self, vulkan_context: &VulkanContext, window: &WindowWrapper) -> Result<(), EngineError> {
        if self.recreate {
            if let Err(err) = self.resize(vulkan_context, window.get_physical_size().into()) {
                return Err(EngineError::Graphics(format!("{:?}", err)));
            };
            return Ok(());
        }

        let (image_i, suboptimal, acquire_future) =
            match vulkano::swapchain::acquire_next_image(self.swapchain.clone(), Some(Duration::from_secs(1)))
                .map_err(vulkano::Validated::unwrap)
            {
                Ok(r) => r,
                Err(vulkano::VulkanError::OutOfDate) => {
                    self.recreate = true;
                    return Ok(());
                }
                Err(err) => return Err(EngineError::Graphics(format!("{:?}", err))),
            };

        if suboptimal {
            self.recreate = true;
            return Ok(());
        }

        let prev_future = match self.fences[self.previous_fence as usize].clone() {
            Some(fence) => fence.boxed(),
            None => {
                let mut now = vulkano::sync::now(vulkan_context.get_device());
                now.cleanup_finished();
                now.boxed()
            }
        };

        if let Some(image_fence) = &self.fences[image_i as usize] {
            match image_fence.wait(None) {
                Ok(_) => {}
                Err(err) => {
                    return Err(EngineError::Graphics(format!("{:?}", err)));
                }
            }
        }

        let future = match prev_future.join(acquire_future)
            .then_execute(vulkan_context.get_queue(), self.command_buffers[image_i as usize].clone()) {
                Ok(val) => val,
                Err(err) => return Err(EngineError::Graphics(format!("{:?}", err))),
            }
            .then_swapchain_present(
                vulkan_context.get_queue(),
                vulkano::swapchain::SwapchainPresentInfo::swapchain_image_index(
                    self.swapchain.clone(),
                    image_i,
                ),
            )
            .then_signal_fence_and_flush();

        self.fences[image_i as usize] = match future.map_err(vulkano::Validated::unwrap) {
            Ok(value) => Some(Arc::new(value)),
            Err(vulkano::VulkanError::OutOfDate) => {
                self.recreate = true;
                None
            },
            Err(_) => {
                None
            }
        };

        self.previous_fence = image_i;

        Ok(())
    }

    fn handle_resize(&mut self) -> Result<(), EngineError> {
        self.recreate = true;

        Ok(())
    }
}
