use crate::{
    error::ParsecError,
    graphics::{
        buffer::{
            Buffer, BufferContent, BufferError, BufferHandle, BufferUsage,
        },
        command_list::{CommandList, CommandListError},
        framebuffer::{Framebuffer, FramebufferError, FramebufferHandle},
        gpu_cpu_fence::{GpuToCpuFence, GpuToCpuFenceError},
        gpu_gpu_fence::{GpuToGpuFence, GpuToGpuFenceError},
        image::{
            Image, ImageAspect, ImageError, ImageFormat, ImageHandle,
            ImageUsage, ImageView, ImageViewHandle,
        },
        pipeline::{
            Pipeline, PipelineError, PipelineHandle, PipelineOptions, PipelineResource, PipelineResourceBindingLayout, PipelineResourceHandle, PipelineResourceLayout, PipelineResourceLayoutHandle
        },
        renderpass::{
            Renderpass, RenderpassAttachment, RenderpassError, RenderpassHandle,
        },
        sampler::{Sampler, SamplerError, SamplerHandle},
        shader::{Shader, ShaderError, ShaderHandle, ShaderType},
        window::Window,
    },
    math::uvec::Vec2u,
};

#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error("failed to initialize graphics backend: {0}")]
    InitError(ParsecError),
    #[error("failed to start frame: {0}")]
    FrameError(ParsecError),
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
    ) -> Result<BufferHandle, BufferError>;
    fn update_buffer(
        &mut self,
        buffer: BufferHandle,
        data: BufferContent<'_>,
    ) -> Result<(), BufferError>;
    fn delete_buffer(&mut self, buffer: Buffer) -> Result<(), BufferError>;

    fn create_shader(
        &mut self,
        code: &[u32],
        shader_type: ShaderType,
    ) -> Result<ShaderHandle, ShaderError>;
    fn delete_shader(&mut self, shader: Shader) -> Result<(), ShaderError>;

    fn create_renderpass(
        &mut self,
        attachments: &[RenderpassAttachment],
    ) -> Result<RenderpassHandle, RenderpassError>;
    fn delete_renderpass(
        &mut self,
        renderpass: Renderpass,
    ) -> Result<(), RenderpassError>;

    // Pipeline

    fn create_pipeline(
        &mut self,
        vertex_shader: ShaderHandle,
        fragment_shader: ShaderHandle,
        renderpass: RenderpassHandle,
        binding_layouts: &[PipelineResourceLayoutHandle],
        options: PipelineOptions,
    ) -> Result<PipelineHandle, PipelineError>;
    fn delete_pipeline(
        &mut self,
        pipeline: Pipeline,
    ) -> Result<(), PipelineError>;

    // Pipeline resource layout

    fn create_pipeline_resource_layout(
        &mut self,
        subbindings: &[PipelineResourceBindingLayout],
    ) -> Result<PipelineResourceLayoutHandle, PipelineError>;
    fn delete_pipeline_resource_layout(
        &mut self,
        layout: PipelineResourceLayout,
    ) -> Result<(), PipelineError>;

    // Pipeline resource

    fn create_pipeline_resource(
        &mut self,
        pipeline_layout: PipelineResourceLayoutHandle,
    ) -> Result<PipelineResourceHandle, PipelineError>;
    fn delete_pipeline_resource(
        &mut self,
        resrouce: PipelineResource,
    ) -> Result<(), PipelineError>;
    fn bind_buffer(
        &mut self,
        pipeline_resource: PipelineResourceHandle,
        buffer: BufferHandle,
        index: u32,
    ) -> Result<(), PipelineError>;
    fn bind_sampler(
        &mut self,
        pipeline_binding: PipelineResourceHandle,
        sampler: SamplerHandle,
        image_view: ImageViewHandle,
        index: u32,
    ) -> Result<(), PipelineError>;

    // Commands

    fn create_command_list(&mut self) -> Result<CommandList, CommandListError>;
    fn submit_commands(
        &mut self,
        command_list: &CommandList,
        wait_fence: &[GpuToGpuFence],
        signal_fence: &[GpuToGpuFence],
        signal_fence: GpuToCpuFence,
    ) -> Result<(), CommandListError>;

    // Frame handling

    fn handle_resize(&mut self, window: &Window) -> Result<(), BackendError>;
    fn present_images(&mut self) -> Vec<ImageHandle>;
    fn start_frame(
        &mut self,
        signal_fence: GpuToGpuFence,
    ) -> Result<u32, BackendError>;
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
    ) -> Result<ImageHandle, ImageError>;
    fn load_image_from_buffer(
        &mut self,
        buffer: BufferHandle,
        image: ImageHandle,
        image_size: Vec2u,
        image_offset: Vec2u,
    ) -> Result<(), ImageError>;
    fn delete_image(&mut self, image: Image) -> Result<(), ImageError>;

    // Image view

    fn create_image_view(
        &mut self,
        image: ImageHandle,
    ) -> Result<ImageViewHandle, ImageError>;
    fn delete_image_view(
        &mut self,
        image_view: ImageView,
    ) -> Result<(), ImageError>;

    // Sampler

    fn create_image_sampler(&mut self) -> Result<SamplerHandle, SamplerError>;
    fn delete_image_sampler(
        &mut self,
        sampler: Sampler,
    ) -> Result<(), SamplerError>;

    // Framebuffer

    fn create_framebuffer(
        &mut self,
        size: Vec2u,
        attachments: &[ImageViewHandle],
        renderpass: RenderpassHandle,
    ) -> Result<FramebufferHandle, FramebufferError>;
    fn delete_framebuffer(
        &mut self,
        framebuffer: Framebuffer,
    ) -> Result<(), FramebufferError>;

    // Sync

    fn create_gpu_to_cpu_fence(
        &mut self,
        signaled: bool,
    ) -> Result<GpuToCpuFence, GpuToCpuFenceError>;
    fn wait_gpu_to_cpu_fence(
        &mut self,
        fence: GpuToCpuFence,
    ) -> Result<(), GpuToCpuFenceError>;
    fn reset_gpu_to_cpu_fence(
        &mut self,
        fence: GpuToCpuFence,
    ) -> Result<(), GpuToCpuFenceError>;
    fn delete_gpu_to_cpu_fence(
        &mut self,
        fence: GpuToCpuFence,
    ) -> Result<(), GpuToCpuFenceError>;

    fn create_gpu_to_gpu_fence(
        &mut self,
    ) -> Result<GpuToGpuFence, GpuToGpuFenceError>;
    fn delete_gpu_to_gpu_fence(
        &mut self,
        fence: GpuToGpuFence,
    ) -> Result<(), GpuToGpuFenceError>;
}
