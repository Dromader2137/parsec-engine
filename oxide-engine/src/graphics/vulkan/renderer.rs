use crate::graphics::{vulkan::graphics_pipeline::VertexFieldFormat, window::WindowWrapper};

use super::{VulkanError, buffer::{Buffer, BufferUsage}, command_buffer::CommandBuffer, context::VulkanContext, fence::Fence, framebuffer::Framebuffer, graphics_pipeline::{GraphicsPipeline, Vertex, VertexField}, image::{Image, ImageView}, renderpass::Renderpass, semaphore::Semaphore, shader::{read_shader_code, ShaderModule}, swapchain::Swapchain};

#[allow(unused)]
pub struct VulkanRenderer {
    swapchain: Swapchain,
    swapchain_images: Vec<Image>,
    swapchain_image_views: Vec<ImageView>,
    renderpass: Renderpass,
    framebuffers: Vec<Framebuffer>,
    command_buffers: Vec<CommandBuffer>,
    command_buffer_fences: Vec<Fence>,
    rendering_semaphores: Vec<Semaphore>,
    present_semaphores: Vec<Semaphore>,
    vertex_shader: ShaderModule,
    fragment_shader: ShaderModule,
    pipeline: GraphicsPipeline,
    resize: bool,
    current_frame: u32,
    frames_in_flight: u32,
    vertex_buffer: Buffer<V>
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct V {
    pos: [f32; 2],
    col: [f32; 3],
}

impl Vertex for V {
    fn size() -> u32 {
        size_of::<f32>() as u32 * 5
    }

    fn description() -> Vec<VertexField> {
        vec![
            VertexField { format: VertexFieldFormat::R32G32_SFLOAT, offset: 0 },
            VertexField { format: VertexFieldFormat::R32G32B32_SFLOAT, offset: size_of::<f32>() as u32 * 2 },
        ]
    }
}

impl V {
    fn new(x: f32, y: f32, r: f32, g: f32, b: f32) -> V {
        V {
            pos: [x, -y],
            col: [r, g, b]
        }
    }
}

impl VulkanRenderer {
    pub fn new(context: &VulkanContext, window: &WindowWrapper) -> Result<VulkanRenderer, VulkanError> {
        let mut frames_in_flight = 3;
        let swapchain = Swapchain::new(&context.instance, &context.surface, &context.physical_device, &context.device, window)?;
        let swapchain_images = swapchain.get_images()?; 
        let swapchain_format = context.surface.format().into();
        let swapchain_image_views = {
            let mut out = Vec::new();
            for img in swapchain_images.iter() {
                let view = ImageView::from_image(&context.device, img, swapchain_format)?;
                out.push(view);
            }
            out
        };
        let renderpass = Renderpass::new(&context.surface, &context.device)?;
        frames_in_flight = frames_in_flight.min(swapchain_images.len() as u32 - 1).max(1);
        let framebuffers = {
            let mut out = Vec::new();
            for image_view in swapchain_image_views.iter() {
                out.push(Framebuffer::new(&context.surface, &context.device, image_view, &renderpass, window)?);
            }
            out
        };
        let command_buffers = {
            let mut out = Vec::new();
            for _ in 0..frames_in_flight {
                out.push(CommandBuffer::new(&context.device, &context.command_pool)?);
            }
            out
        };
        let command_buffer_fences = {
            let mut out = Vec::new();
            for _ in 0..frames_in_flight {
                out.push(Fence::new(&context.device, true)?);
            }
            out
        };
        let rendering_semaphores = {
            let mut out = Vec::new();
            for _ in 0..frames_in_flight {
                out.push(Semaphore::new(&context.device)?);
            }
            out
        };
        let present_semaphores = {
            let mut out = Vec::new();
            for _ in 0..frames_in_flight {
                out.push(Semaphore::new(&context.device)?);
            }
            out
        };
        let vertex_shader = ShaderModule::new(&context.device, &read_shader_code("shaders/simple.spv")?)?;
        let fragment_shader = ShaderModule::new(&context.device, &read_shader_code("shaders/flat.spv")?)?;
        let pipeline = GraphicsPipeline::new::<V>(&context.device, &framebuffers[0], &renderpass, &vertex_shader, &fragment_shader)?;

        let vertices = vec![
            V::new(0.0, 0.8, 1.0, 0.0, 0.0),
            V::new(-0.8, -0.8, 0.0, 1.0, 0.0),
            V::new(0.8, -0.8, 0.0, 0.0, 1.0),
        ];

        let vertex_buffer = Buffer::from_vec(&context.instance, &context.physical_device, &context.device, vertices, BufferUsage::VERTEX_BUFFER)?;

        Ok(VulkanRenderer { swapchain, swapchain_images, swapchain_image_views, renderpass, framebuffers, command_buffers, command_buffer_fences, rendering_semaphores, present_semaphores, vertex_shader, fragment_shader, pipeline, resize: false , current_frame: 0, frames_in_flight, vertex_buffer})
    }

    pub fn recreate_size_dependent_components(&mut self, context: &VulkanContext, window: &WindowWrapper) -> Result<(), VulkanError> {
        context.device.wait_idle()?;
        self.framebuffers.iter().for_each(|x| x.cleanup(&context.device));
        self.swapchain_image_views.iter().for_each(|x| x.cleanup(&context.device));
        self.swapchain.cleanup();

        let swapchain = Swapchain::new(&context.instance, &context.surface, &context.physical_device, &context.device, window)?;
        let swapchain_images = swapchain.get_images()?; 
        let swapchain_format = context.surface.format().into();
        let swapchain_image_views = {
            let mut out = Vec::new();
            for img in swapchain_images.iter() {
                let view = ImageView::from_image(&context.device, img, swapchain_format)?;
                out.push(view);
            }
            out
        };
        let framebuffers = {
            let mut out = Vec::new();
            for image_view in swapchain_image_views.iter() {
                out.push(Framebuffer::new(&context.surface, &context.device, image_view, &self.renderpass, window)?);
            }
            out
        };

        self.swapchain = swapchain;
        self.swapchain_images = swapchain_images;
        self.swapchain_image_views = swapchain_image_views;
        self.framebuffers = framebuffers;

        Ok(())
    }

    pub fn cleanup(&mut self, context: &VulkanContext) -> Result<(), VulkanError> {
        context.device.wait_idle()?;
        self.renderpass.cleanup(&context.device);
        self.present_semaphores.iter().for_each(|x| x.cleanup(&context.device));
        self.rendering_semaphores.iter().for_each(|x| x.cleanup(&context.device));
        self.command_buffer_fences.iter().for_each(|x| x.cleanup(&context.device));
        self.fragment_shader.cleanup(&context.device);
        self.vertex_shader.cleanup(&context.device);
        self.pipeline.cleanup(&context.device);
        self.framebuffers.iter().for_each(|x| x.cleanup(&context.device));
        self.swapchain_image_views.iter().for_each(|x| x.cleanup(&context.device));
        self.swapchain_images.iter().for_each(|x| x.cleanup(&context.device));
        self.swapchain.cleanup();
        Ok(())
    }

    pub fn render(&mut self, context: &VulkanContext, window: &WindowWrapper) -> Result<(), VulkanError> {
        let current_frame = self.current_frame as usize;
        self.command_buffer_fences[current_frame].wait(&context.device)?;
        if window.minimized() {
            return Ok(());
        }
        if self.resize {
            self.recreate_size_dependent_components(context, window)?;
            self.resize = false;
            self.current_frame = 0;
        }
        let (present_index, suboptimal) = self.swapchain.acquire_next_image(&self.present_semaphores[current_frame], &Fence::null())?;
        self.resize |= suboptimal;
        self.command_buffer_fences[current_frame].reset(&context.device)?;
        let present_index = present_index as usize;
        let command_buffer = &self.command_buffers[current_frame];
        let framebuffer = &self.framebuffers[present_index];
        command_buffer.reset(&context.device)?;
        command_buffer.begin(&context.device)?;
        command_buffer.begin_renderpass(&context.device, &self.renderpass, framebuffer);
        command_buffer.set_viewports(&context.device, framebuffer);
        command_buffer.set_scissor(&context.device, framebuffer);
        command_buffer.bind_graphics_pipeline(&context.device, &self.pipeline);
        command_buffer.bind_vertex_buffer(&context.device, &self.vertex_buffer);
        command_buffer.draw(&context.device, 6, 1, 0, 0);
        command_buffer.end_renderpass(&context.device);
        command_buffer.end(&context.device)?;
        context.graphics_queue.submit(&context.device, &[&self.present_semaphores[current_frame]], &[&self.rendering_semaphores[current_frame]], &[command_buffer], &self.command_buffer_fences[current_frame])?;
        self.swapchain.present(&context.graphics_queue, &[&self.rendering_semaphores[current_frame]], present_index as u32)?;
        self.current_frame = (self.current_frame + 1) % self.frames_in_flight;
        Ok(())
    }

    pub fn handle_resize(&mut self) -> Result<(), VulkanError> {
        self.resize = true;
        Ok(())
    }
}
