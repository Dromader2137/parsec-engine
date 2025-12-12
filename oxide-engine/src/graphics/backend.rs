use crate::graphics::{
    buffer::{Buffer, BufferError, BufferUsage},
    command_list::{CommandList, CommandListError},
    fence::Fence,
    framebuffer::Framebuffer,
    image::{Image, ImageFormat, ImageUsage, ImageView},
    pipeline::{
        Pipeline, PipelineBinding, PipelineError, PipelineLayout,
        PipelineLayoutBinding,
    },
    renderpass::{Renderpass, RenderpassError},
    semaphore::Semaphore,
    shader::{Shader, ShaderError, ShaderType},
    swapchain::{Swapchain, SwapchainError},
    window::Window,
};

pub trait GraphicsBackend {
    fn init(window: &Window) -> Self;
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
    fn create_shader(
        &mut self,
        code: &[u32],
        shader_type: ShaderType,
    ) -> Result<Shader, ShaderError>;
    fn create_renderpass(&mut self) -> Result<Renderpass, RenderpassError>;
    fn create_pipeline_layout(
        &mut self,
        bindings: &[&[PipelineLayoutBinding]],
    ) -> Result<PipelineLayout, PipelineError>;
    fn create_pipeline(
        &mut self,
        vertex_shader: Shader,
        fragment_shader: Shader,
        renderpass: Renderpass,
        layout: PipelineLayout,
    ) -> Result<Pipeline, PipelineError>;
    fn create_pipeline_binding(
        &mut self,
        pipeline_layout: PipelineLayout,
        binding: u32,
    ) -> Result<PipelineBinding, PipelineError>;
    fn bind_buffer(
        &mut self,
        pipeline_binding: PipelineBinding,
        buffer: Buffer,
        index: u32,
    ) -> Result<(), BufferError>;
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
        present_image_index: u32,
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
    );
    fn create_swapchain(
        &mut self,
        window: &Window,
        old_swapchain: Option<Swapchain>,
    ) -> Result<(Swapchain, Vec<Image>), SwapchainError>;
    fn delete_swapchain(&mut self, swapchain: Swapchain);
    fn create_image(
        &mut self,
        size: (u32, u32),
        format: ImageFormat,
        usage: ImageUsage,
    ) -> Image;
    fn delete_image(&mut self, image: Image);
    fn create_image_view(&mut self, image: Image) -> ImageView;
    fn delete_image_view(&mut self, image_view: ImageView);
    fn create_framebuffer(
        &mut self,
        window: &Window,
        color_view: ImageView,
        depth_view: ImageView,
        renderpass: Renderpass,
    ) -> Framebuffer;
    fn delete_framebuffer(&mut self, framebuffer: Framebuffer);
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
    fn create_fence(&mut self, signaled: bool) -> Fence;
    fn wait_fence(&mut self, fence: Fence);
    fn reset_fence(&mut self, fence: Fence);
    fn create_semaphore(&mut self) -> Semaphore;
}
