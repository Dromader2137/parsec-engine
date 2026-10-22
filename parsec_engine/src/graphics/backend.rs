use parsec_engine_math::uvec::Vec2u;

use crate::graphics::{
    buffer::{Buffer, BufferContent, BufferHandle, BufferUsage},
    command_list::CommandList,
    framebuffer::{Framebuffer, FramebufferHandle},
    gpu_cpu_fence::GpuToCpuFence,
    gpu_gpu_fence::GpuToGpuFence,
    image::{Image, ImageAspect, ImageFormat, ImageHandle, ImageUsage, ImageView, ImageViewHandle},
    pipeline::{
        Pipeline, PipelineHandle, PipelineOptions, PipelineResource, PipelineResourceBindingLayout,
        PipelineResourceHandle, PipelineResourceLayout, PipelineResourceLayoutHandle,
    },
    renderpass::{Renderpass, RenderpassAttachment, RenderpassHandle},
    sampler::{Sampler, SamplerHandle},
    shader_module::{ShaderHandle, ShaderModule, ShaderType},
    window::Window,
};

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct BackendError(Box<dyn std::error::Error + Send + Sync>);

impl BackendError {
    pub fn new(err: impl std::error::Error + Send + Sync + 'static) -> Self { Self(Box::new(err)) }
}

pub trait GraphicsBackend: Send + Sync + 'static {
    fn init(window: &Window) -> Result<Self, BackendError>
    where
        Self: Sized;
    fn wait_idle(&self);

    fn get_surface_format(&self) -> ImageFormat;

    fn create_buffer(
        &mut self,
        data: BufferContent<'_>,
        buffer_usage: &[BufferUsage],
    ) -> Result<BufferHandle, BackendError>;
    fn update_buffer(
        &mut self,
        buffer: BufferHandle,
        data: BufferContent<'_>,
    ) -> Result<(), BackendError>;
    fn delete_buffer(&mut self, buffer: Buffer) -> Result<(), BackendError>;

    fn create_shader(
        &mut self,
        code: &[u32],
        shader_type: ShaderType,
    ) -> Result<ShaderHandle, BackendError>;
    fn delete_shader(&mut self, shader: ShaderModule) -> Result<(), BackendError>;

    fn create_renderpass(
        &mut self,
        attachments: &[RenderpassAttachment],
    ) -> Result<RenderpassHandle, BackendError>;
    fn delete_renderpass(&mut self, renderpass: Renderpass) -> Result<(), BackendError>;

    // Pipeline

    fn create_pipeline(
        &mut self,
        vertex_shader: ShaderHandle,
        fragment_shader: ShaderHandle,
        renderpass: RenderpassHandle,
        binding_layouts: &[PipelineResourceLayoutHandle],
        options: PipelineOptions,
    ) -> Result<PipelineHandle, BackendError>;
    fn delete_pipeline(&mut self, pipeline: Pipeline) -> Result<(), BackendError>;

    // Pipeline resource layout

    fn create_pipeline_resource_layout(
        &mut self,
        subbindings: &[PipelineResourceBindingLayout],
    ) -> Result<PipelineResourceLayoutHandle, BackendError>;
    fn delete_pipeline_resource_layout(
        &mut self,
        layout: PipelineResourceLayout,
    ) -> Result<(), BackendError>;

    // Pipeline resource

    fn create_pipeline_resource(
        &mut self,
        pipeline_layout: PipelineResourceLayoutHandle,
    ) -> Result<PipelineResourceHandle, BackendError>;
    fn delete_pipeline_resource(&mut self, resrouce: PipelineResource) -> Result<(), BackendError>;
    fn bind_buffer(
        &mut self,
        pipeline_resource: PipelineResourceHandle,
        buffer: BufferHandle,
        index: u32,
    ) -> Result<(), BackendError>;
    fn bind_sampler(
        &mut self,
        pipeline_binding: PipelineResourceHandle,
        sampler: SamplerHandle,
        image_view: ImageViewHandle,
        index: u32,
    ) -> Result<(), BackendError>;

    // Commands

    fn create_command_list(&mut self) -> Result<CommandList, BackendError>;
    fn submit_commands(
        &mut self,
        command_list: &CommandList,
        wait_fence: &[GpuToGpuFence],
        signal_fence: &[GpuToGpuFence],
        signal_fence: GpuToCpuFence,
    ) -> Result<(), BackendError>;

    // Frame handling

    fn handle_resize(&mut self, window: &Window) -> Result<(), BackendError>;
    fn present_images(&mut self) -> Vec<ImageHandle>;
    fn start_frame(&mut self, signal_fence: GpuToGpuFence) -> Result<u32, BackendError>;
    fn end_frame(
        &mut self,
        wait_fence: &[GpuToGpuFence],
        present_image_index: u32,
    ) -> Result<(), BackendError>;

    // Image

    fn create_image(
        &mut self,
        size: Vec2u,
        format: ImageFormat,
        aspect: ImageAspect,
        usage: &[ImageUsage],
    ) -> Result<ImageHandle, BackendError>;
    fn load_image_from_buffer(
        &mut self,
        buffer: BufferHandle,
        image: ImageHandle,
        image_size: Vec2u,
        image_offset: Vec2u,
    ) -> Result<(), BackendError>;
    fn delete_image(&mut self, image: Image) -> Result<(), BackendError>;

    // Image view

    fn create_image_view(&mut self, image: ImageHandle) -> Result<ImageViewHandle, BackendError>;
    fn delete_image_view(&mut self, image_view: ImageView) -> Result<(), BackendError>;

    // Sampler

    fn create_image_sampler(&mut self) -> Result<SamplerHandle, BackendError>;
    fn delete_image_sampler(&mut self, sampler: Sampler) -> Result<(), BackendError>;

    // Framebuffer

    fn create_framebuffer(
        &mut self,
        size: Vec2u,
        attachments: &[ImageViewHandle],
        renderpass: RenderpassHandle,
    ) -> Result<FramebufferHandle, BackendError>;
    fn delete_framebuffer(&mut self, framebuffer: Framebuffer) -> Result<(), BackendError>;

    // Sync

    fn create_gpu_to_cpu_fence(&mut self, signaled: bool) -> Result<GpuToCpuFence, BackendError>;
    fn wait_gpu_to_cpu_fence(&mut self, fence: GpuToCpuFence) -> Result<(), BackendError>;
    fn reset_gpu_to_cpu_fence(&mut self, fence: GpuToCpuFence) -> Result<(), BackendError>;
    fn delete_gpu_to_cpu_fence(&mut self, fence: GpuToCpuFence) -> Result<(), BackendError>;

    fn create_gpu_to_gpu_fence(&mut self) -> Result<GpuToGpuFence, BackendError>;
    fn delete_gpu_to_gpu_fence(&mut self, fence: GpuToGpuFence) -> Result<(), BackendError>;
}
