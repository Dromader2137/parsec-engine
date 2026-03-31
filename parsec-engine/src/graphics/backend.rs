use crate::{
    graphics::{
        buffer::{Buffer, BufferContent, BufferError, BufferUsage},
        command_list::{CommandList, CommandListError},
        fence::{Fence, FenceError},
        framebuffer::{Framebuffer, FramebufferError},
        image::{
            Image, ImageAspect, ImageError, ImageFormat, ImageUsage, ImageView,
        },
        pipeline::{
            Pipeline, PipelineError, PipelineOptions, PipelineSubbindingLayout,
            ResourceBinding, ResourceBindingLayout,
        },
        renderpass::{Renderpass, RenderpassAttachment, RenderpassError},
        sampler::{Sampler, SamplerError},
        semaphore::{Semaphore, SemaphoreError},
        shader::{Shader, ShaderError, ShaderType},
        swapchain::{Swapchain, SwapchainError},
        window::Window,
    },
    math::uvec::Vec2u,
};

#[derive(Debug)]
pub enum BackendInitError {
    InitError(anyhow::Error),
}

#[derive(Debug, Default)]
pub struct FrameBeginInfo {
    pub resized: bool,
    pub present_images: Vec<Image>,
    pub current_image: u32,
}

pub trait GraphicsBackend: Send + Sync + 'static {
    fn init() -> Result<Self, BackendInitError>
    where
        Self: Sized;
    fn wait_idle(&self);

    fn request_redraw(&self);
    fn handle_resize(&mut self);
    fn resized(&self) -> bool;

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

    fn create_resource_binding_layout(
        &mut self,
        subbindings: &[PipelineSubbindingLayout],
    ) -> Result<ResourceBindingLayout, PipelineError>;
    fn create_pipeline(
        &mut self,
        vertex_shader: Shader,
        fragment_shader: Shader,
        renderpass: Renderpass,
        resource_binding_layouts: &[ResourceBindingLayout],
        options: PipelineOptions,
    ) -> Result<Pipeline, PipelineError>;
    fn create_resource_binding(
        &mut self,
        resource_binding_layout: ResourceBindingLayout,
    ) -> Result<ResourceBinding, PipelineError>;
    fn bind_buffer(
        &mut self,
        resource_binding: ResourceBinding,
        buffer: Buffer,
        index: u32,
    ) -> Result<(), BufferError>;
    fn bind_sampler(
        &mut self,
        resource_binding: ResourceBinding,
        sampler: Sampler,
        image_view: ImageView,
        index: u32,
    ) -> Result<(), SamplerError>;

    fn create_command_list(&mut self) -> Result<CommandList, CommandListError>;
    fn submit_commands(
        &mut self,
        command_list: &CommandList,
        wait_semaphores: &[Semaphore],
        signal_semaphores: &[Semaphore],
        signal_fence: Fence,
    ) -> Result<(), CommandListError>;

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

    fn create_fence(&mut self, signaled: bool) -> Result<Fence, FenceError>;
    fn wait_fence(&mut self, fence: Fence) -> Result<(), FenceError>;
    fn reset_fence(&mut self, fence: Fence) -> Result<(), FenceError>;
    fn delete_fence(&mut self, fence: Fence) -> Result<(), FenceError>;

    fn begin_frame(&mut self) -> Result<FrameBeginInfo, SwapchainError>;
    fn end_frame(&mut self);
}
