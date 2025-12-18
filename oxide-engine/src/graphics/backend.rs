use crate::graphics::{
    buffer::{Buffer, BufferError, BufferUsage},
    command_list::{CommandList, CommandListError},
    fence::{Fence, FenceError},
    framebuffer::{Framebuffer, FramebufferError},
    image::{Image, ImageError, ImageFormat, ImageUsage, ImageView},
    pipeline::{
        Pipeline, PipelineBinding, PipelineBindingLayout, PipelineError,
        PipelineSubbindingLayout,
    },
    renderpass::{Renderpass, RenderpassError},
    sampler::{Sampler, SamplerError},
    semaphore::{Semaphore, SemaphoreError},
    shader::{Shader, ShaderError, ShaderType},
    swapchain::{Swapchain, SwapchainError},
    window::Window,
};

#[derive(Debug)]
pub enum BackendInitError {
    InitError(anyhow::Error)
}

pub trait GraphicsBackend: Sized {
    fn init(window: &Window) -> Result<Self, BackendInitError>;
    fn wait_idle(&self);

    fn create_buffer<T: Clone + Copy>(
        &mut self,
        data: &[T],
        buffer_usage: &[BufferUsage],
    ) -> Result<Buffer, BufferError>;
    fn update_buffer<T: Clone + Copy>(
        &mut self,
        buffer: Buffer,
        data: &[T],
    ) -> Result<(), BufferError>;
    fn delete_buffer(&mut self, buffer: Buffer) -> Result<(), BufferError>;

    fn create_shader(
        &mut self,
        code: &[u32],
        shader_type: ShaderType,
    ) -> Result<Shader, ShaderError>;
    fn delete_shader(&mut self, shader: Shader) -> Result<(), ShaderError>;

    fn create_renderpass(&mut self) -> Result<Renderpass, RenderpassError>;
    fn delete_renderpass(
        &mut self,
        renderpass: Renderpass,
    ) -> Result<(), RenderpassError>;

    fn create_pipeline_binding_layout(
        &mut self,
        subbindings: &[PipelineSubbindingLayout],
    ) -> Result<PipelineBindingLayout, PipelineError>;
    fn create_pipeline(
        &mut self,
        vertex_shader: Shader,
        fragment_shader: Shader,
        renderpass: Renderpass,
        binding_layouts: &[PipelineBindingLayout],
    ) -> Result<Pipeline, PipelineError>;
    fn create_pipeline_binding(
        &mut self,
        pipeline_layout: PipelineBindingLayout,
    ) -> Result<PipelineBinding, PipelineError>;
    fn bind_buffer(
        &mut self,
        pipeline_binding: PipelineBinding,
        buffer: Buffer,
        index: u32,
    ) -> Result<(), BufferError>;
    fn bind_sampler(
        &mut self,
        pipeline_binding: PipelineBinding,
        sampler: Sampler,
        image_view: ImageView,
        index: u32,
    ) -> Result<(), SamplerError>;

    fn create_command_list(&mut self) -> Result<CommandList, CommandListError>;
    fn command_begin(
        &mut self,
        command_list: CommandList,
    ) -> Result<(), CommandListError>;
    fn command_end(
        &mut self,
        command_list: CommandList,
    ) -> Result<(), CommandListError>;
    fn command_reset(
        &mut self,
        command_list: CommandList,
    ) -> Result<(), CommandListError>;
    fn command_begin_renderpass(
        &mut self,
        command_list: CommandList,
        renderpass: Renderpass,
        framebuffer: Framebuffer,
    ) -> Result<(), CommandListError>;
    fn command_end_renderpass(
        &mut self,
        command_list: CommandList,
    ) -> Result<(), CommandListError>;
    fn command_bind_pipeline(
        &mut self,
        command_list: CommandList,
        pipeline: Pipeline,
    ) -> Result<(), CommandListError>;
    fn command_bind_pipeline_binding(
        &mut self,
        command_list: CommandList,
        pipeline: Pipeline,
        binding: PipelineBinding,
        binding_index: u32,
    ) -> Result<(), CommandListError>;
    fn command_draw(
        &mut self,
        command_list: CommandList,
        vertex_buffer: Buffer,
        index_buffer: Buffer,
    ) -> Result<(), CommandListError>;
    fn submit_commands(
        &mut self,
        command_list: CommandList,
        wait_semaphores: &[Semaphore],
        signal_semaphores: &[Semaphore],
        signal_fence: Fence,
    ) -> Result<(), CommandListError>;

    fn create_swapchain(
        &mut self,
        window: &Window,
        old_swapchain: Option<Swapchain>,
    ) -> Result<(Swapchain, Vec<Image>), SwapchainError>;
    fn delete_swapchain(
        &mut self,
        swapchain: Swapchain,
    ) -> Result<(), SwapchainError>;
    fn next_image_id(
        &mut self,
        swapchain: Swapchain,
        signal_semaphore: Semaphore,
    ) -> Result<u32, SwapchainError>;
    fn present(
        &mut self,
        swapchain: Swapchain,
        wait_semaphores: &[Semaphore],
        present_image_index: u32,
    ) -> Result<(), SwapchainError>;

    fn create_image(
        &mut self,
        size: (u32, u32),
        format: ImageFormat,
        image_usage: &[ImageUsage],
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
        window: &Window,
        color_view: ImageView,
        depth_view: ImageView,
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

    fn create_semaphore(&mut self) -> Result<Semaphore, SemaphoreError>;
    fn delete_semaphore(
        &mut self,
        semaphore: Semaphore,
    ) -> Result<(), SemaphoreError>;
}
