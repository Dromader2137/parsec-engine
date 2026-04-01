use crate::{
    graphics::{
        buffer::{Buffer, BufferContent, BufferError, BufferUsage},
        command_list::{CommandList, CommandListError},
        framebuffer::{Framebuffer, FramebufferError},
        gpu_cpu_fence::{GpuToCpuFence, GpuToCpuFenceError},
        gpu_gpu_fence::{GpuToGpuFence, GpuToGpuFenceError},
        image::{
            Image, ImageAspect, ImageError, ImageFormat, ImageUsage, ImageView,
        },
        pipeline::{
            Pipeline, PipelineError, PipelineOptions, PipelineResource,
            PipelineResourceBindingLayout, PipelineResourceLayout,
        },
        renderpass::{Renderpass, RenderpassAttachment, RenderpassError},
        sampler::{Sampler, SamplerError},
        shader::{Shader, ShaderError, ShaderType},
        window::Window,
    },
    math::uvec::Vec2u, error::ParsecError,
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
    ) -> Result<Buffer, BufferError>;
    fn update_buffer(
        &mut self,
        buffer: Buffer,
        data: BufferContent<'_>,
    ) -> Result<(), BufferError>;
    fn delete_buffer(&mut self, buffer: Buffer) -> Result<(), BufferError>;

    fn create_shader(
        &mut self,
        code: &[u32],
        shader_type: ShaderType,
    ) -> Result<Shader, ShaderError>;
    fn delete_shader(&mut self, shader: Shader) -> Result<(), ShaderError>;

    fn create_renderpass(
        &mut self,
        attachments: &[RenderpassAttachment],
    ) -> Result<Renderpass, RenderpassError>;
    fn delete_renderpass(
        &mut self,
        renderpass: Renderpass,
    ) -> Result<(), RenderpassError>;

    fn create_pipeline_resource_layout(
        &mut self,
        subbindings: &[PipelineResourceBindingLayout],
    ) -> Result<PipelineResourceLayout, PipelineError>;
    fn create_pipeline(
        &mut self,
        vertex_shader: Shader,
        fragment_shader: Shader,
        renderpass: Renderpass,
        binding_layouts: &[PipelineResourceLayout],
        options: PipelineOptions,
    ) -> Result<Pipeline, PipelineError>;
    fn create_pipeline_resource(
        &mut self,
        pipeline_layout: PipelineResourceLayout,
    ) -> Result<PipelineResource, PipelineError>;
    fn bind_buffer(
        &mut self,
        pipeline_binding: PipelineResource,
        buffer: Buffer,
        index: u32,
    ) -> Result<(), BufferError>;
    fn bind_sampler(
        &mut self,
        pipeline_binding: PipelineResource,
        sampler: Sampler,
        image_view: ImageView,
        index: u32,
    ) -> Result<(), SamplerError>;

    fn create_command_list(&mut self) -> Result<CommandList, CommandListError>;
    fn submit_commands(
        &mut self,
        command_list: &CommandList,
        wait_fence: &[GpuToGpuFence],
        signal_fence: &[GpuToGpuFence],
        signal_fence: GpuToCpuFence,
    ) -> Result<(), CommandListError>;

    fn handle_resize(&mut self, window: &Window) -> Result<(), BackendError>;
    fn present_images(&mut self) -> Vec<Image>;
    fn start_frame(
        &mut self,
        signal_fence: GpuToGpuFence,
    ) -> Result<u32, BackendError>;
    fn end_frame(
        &mut self,
        wait_fence: &[GpuToGpuFence],
        present_image_index: u32,
    ) -> Result<(), BackendError>;

    fn create_image(
        &mut self,
        size: Vec2u,
        format: ImageFormat,
        aspect: ImageAspect,
        usage: &[ImageUsage],
    ) -> Result<Image, ImageError>;
    fn load_image_from_buffer(
        &mut self,
        buffer: Buffer,
        image: Image,
        image_size: Vec2u,
        image_offset: Vec2u,
    ) -> Result<(), ImageError>;
    fn delete_image(&mut self, image: Image) -> Result<(), ImageError>;
    fn create_image_view(
        &mut self,
        image: Image,
    ) -> Result<ImageView, ImageError>;
    fn delete_image_view(
        &mut self,
        image_view: ImageView,
    ) -> Result<(), ImageError>;
    fn create_image_sampler(&mut self) -> Result<Sampler, SamplerError>;
    fn delete_image_sampler(
        &mut self,
        sampler: Sampler,
    ) -> Result<(), SamplerError>;

    fn create_framebuffer(
        &mut self,
        size: Vec2u,
        attachments: &[ImageView],
        renderpass: Renderpass,
    ) -> Result<Framebuffer, FramebufferError>;
    fn delete_framebuffer(
        &mut self,
        framebuffer: Framebuffer,
    ) -> Result<(), FramebufferError>;

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
