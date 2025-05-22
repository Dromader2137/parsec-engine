use std::{f32, sync::Arc};

pub mod camera;
pub mod image_data;
pub mod material;
pub mod sync;

use image_data::VulkanRendererImageData;
use sync::{VulkanRendererFrameSync, VulkanRendererImageSync};

use crate::{
    graphics::{
        mesh::MeshData, vulkan::graphics_pipeline::VertexFieldFormat, window::WindowWrapper,
    },
    math::{mat::Matrix4f, vec::Vec3f},
};

use super::{
    VulkanError,
    buffer::{Buffer, BufferUsage},
    command_buffer::CommandBuffer,
    context::VulkanContext,
    descriptor_set::{
        DescriptorPool, DescriptorPoolSize, DescriptorSet, DescriptorSetBinding,
        DescriptorSetLayout, DescriptorStage, DescriptorType,
    },
    fence::Fence,
    graphics_pipeline::{GraphicsPipeline, Vertex, VertexField},
    renderpass::Renderpass,
    shader::{ShaderModule, read_shader_code},
};

#[allow(unused)]
pub struct VulkanRenderer {
    context: Arc<VulkanContext>,
    renderpass: Arc<Renderpass>,
    image_data: VulkanRendererImageData,
    frame_sync: Vec<VulkanRendererFrameSync>,
    image_sync: Vec<VulkanRendererImageSync>,
    command_buffers: Vec<Arc<CommandBuffer>>,
    pipeline: Arc<GraphicsPipeline>,
    descriptor_pool: Arc<DescriptorPool>,
    mvp_set: Arc<DescriptorSet>,
    mvp_buffer: Arc<Buffer<Matrix4f>>,
    mesh_data: MeshData<DefaultVertex>,
    vertex_shader: Arc<ShaderModule>,
    fragment_shader: Arc<ShaderModule>,
    resize: bool,
    current_frame: usize,
    frames_in_flight: usize,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct DefaultVertex {
    position: [f32; 3],
    normal: [f32; 3],
    tangent: [f32; 3],
    uv: [f32; 2],
}

impl Vertex for DefaultVertex {
    fn size() -> u32 {
        size_of::<f32>() as u32 * 11
    }

    fn description() -> Vec<VertexField> {
        vec![
            VertexField {
                format: VertexFieldFormat::R32G32B32_SFLOAT,
                offset: 0,
            },
            VertexField {
                format: VertexFieldFormat::R32G32B32_SFLOAT,
                offset: size_of::<f32>() as u32 * 3,
            },
            VertexField {
                format: VertexFieldFormat::R32G32B32_SFLOAT,
                offset: size_of::<f32>() as u32 * 6,
            },
            VertexField {
                format: VertexFieldFormat::R32G32_SFLOAT,
                offset: size_of::<f32>() as u32 * 9,
            },
        ]
    }
}

impl DefaultVertex {
    fn new(pos: Vec3f, nor: Vec3f) -> DefaultVertex {
        DefaultVertex {
            position: [pos.x, pos.y, pos.z],
            normal: [nor.x, nor.y, nor.z],
            tangent: [0.0, 1.0, 0.0],
            uv: [0.0, 0.0],
        }
    }
}

fn create_frame_sync(context: Arc<VulkanContext>, frames_in_flight: usize) -> Result<Vec<VulkanRendererFrameSync>, VulkanError> {
    let mut ret = Vec::new();
    for _ in 0..frames_in_flight {
        ret.push(VulkanRendererFrameSync::new(context.device.clone())?);
    }
    Ok(ret)
}

fn create_image_sync(context: Arc<VulkanContext>, image_count: usize) -> Result<Vec<VulkanRendererImageSync>, VulkanError> {
    let mut ret = Vec::new();
    for _ in 0..image_count {
        ret.push(VulkanRendererImageSync::new(context.device.clone())?);
    }
    Ok(ret)
}

fn create_commad_buffers(context: Arc<VulkanContext>, frames_in_flight: usize) -> Result<Vec<Arc<CommandBuffer>>, VulkanError> {
    let mut ret = Vec::new();
    for _ in 0..frames_in_flight {
        ret.push(CommandBuffer::new(context.command_pool.clone())?);
    }
    Ok(ret)
}

impl VulkanRenderer {
    pub fn new(
        context: Arc<VulkanContext>,
        window: Arc<WindowWrapper>,
    ) -> Result<VulkanRenderer, VulkanError> {
        let mut frames_in_flight = 2;
        let renderpass = Renderpass::new(context.surface.clone(), context.device.clone())?;
        let image_data = VulkanRendererImageData::new(context.clone(), renderpass.clone(), window)?;
        frames_in_flight = image_data.clamp_frames_in_flight(frames_in_flight);
        let frame_sync = create_frame_sync(context.clone(), frames_in_flight)?;
        let image_sync = create_image_sync(context.clone(), image_data.swapchain.swapchain_images.len())?;
        let command_buffers = create_commad_buffers(context.clone(), frames_in_flight)?;
        let vertex_shader = ShaderModule::new(context.device.clone(), &read_shader_code("shaders/simple.spv")?)?;
        let fragment_shader = ShaderModule::new(context.device.clone(), &read_shader_code("shaders/flat.spv")?)?;

        let pos = vec![
            Vec3f::new(0.0, 0.0, 0.0),
            Vec3f::new(1.0, 1.0, 1.0),
            Vec3f::new(0.0, 1.0, 1.0),
            Vec3f::new(0.0, 0.0, 0.0),
        ];

        let nor = vec![
            Vec3f::new(0.0, 0.0, 0.0),
            Vec3f::new(-0.966742, -0.255752, 0.0),
            Vec3f::new(-0.966824, 0.255443, 0.0),
            Vec3f::new(-0.092052, 0.995754, 0.0),
        ];

        let indices = vec![1, 2, 3];

        let vertices = pos
            .iter()
            .zip(nor.iter())
            .map(|x| DefaultVertex::new(*x.0, *x.1))
            .collect();

        let mesh_data = MeshData::new(
            context.device.clone(),
            vertices,
            indices,
        )?;

        let mvp_buffer = Buffer::from_vec(
            context.device.clone(),
            vec![Matrix4f::indentity()],
            BufferUsage::UNIFORM_BUFFER,
        )?;

        let descriptor_pool = DescriptorPool::new(
            context.device.clone(),
            32,
            &[DescriptorPoolSize::new(16, DescriptorType::UNIFORM_BUFFER)],
        )?;

        let bindings = vec![DescriptorSetBinding::new(
            0,
            DescriptorType::UNIFORM_BUFFER,
            DescriptorStage::VERTEX,
        )];

        let mvp_set_layout = DescriptorSetLayout::new(context.device.clone(), bindings)?;
        let mvp_set = DescriptorSet::new(mvp_set_layout, descriptor_pool.clone())?;
        mvp_set.bind_buffer(mvp_buffer.clone(), 0)?;

        let pipeline = GraphicsPipeline::new::<DefaultVertex>(
            image_data.framebuffers[0].clone(),
            vertex_shader.clone(),
            fragment_shader.clone(),
            vec![mvp_set.descriptor_layout.clone()],
        )?;

        Ok(VulkanRenderer {
            context,
            renderpass,
            image_data,
            frame_sync,
            image_sync,
            command_buffers,
            vertex_shader,
            fragment_shader,
            pipeline,
            frames_in_flight,
            descriptor_pool,
            mvp_set,
            mvp_buffer,
            mesh_data,
            resize: false,
            current_frame: 0,
        })
    }

    pub fn recreate_size_dependent_components(
        &mut self,
        window: Arc<WindowWrapper>,
    ) -> Result<(), VulkanError> {
        self.context.device.wait_idle()?;

        self.image_data.recreate(self.context.clone(), self.renderpass.clone(), window)?;

        Ok(())
    }

    pub fn render(
        &mut self,
        window: Arc<WindowWrapper>,
    ) -> Result<(), VulkanError> {
        let current_frame = self.current_frame as usize;
        self.frame_sync[current_frame].command_buffer_fence.wait()?;

        if window.minimized() {
            return Ok(());
        }

        if self.resize {
            self.recreate_size_dependent_components(window.clone())?;
            self.resize = false;
        }

        let (present_index, suboptimal) = self
            .image_data
            .swapchain
            .acquire_next_image(self.frame_sync[current_frame].image_available_semaphore.clone(), Fence::null(self.context.device.clone()))?;
        let present_index = present_index as usize;

        self.resize |= suboptimal;
        self.frame_sync[current_frame].command_buffer_fence.reset()?;

        let command_buffer = self.command_buffers[current_frame].clone();
        let framebuffer = self.image_data.framebuffers[present_index].clone();

        let (width, height) = (window.get_width(), window.get_height());
        let aspect = width as f32 / height as f32;
        self.mvp_buffer.update(
            vec![
                Matrix4f::perspective(40.0_f32.to_radians(), aspect, 5.0, 100.0)
                    * Matrix4f::look_at(Vec3f::ZERO, Vec3f::FORWARD, Vec3f::UP)
                    * Matrix4f::translation(Vec3f::FORWARD * 30.0)
            ],
        )?;

        command_buffer.reset()?;
        command_buffer.begin()?;
        command_buffer.begin_renderpass(framebuffer.clone());
        command_buffer.set_viewports(framebuffer.clone());
        command_buffer.set_scissor(framebuffer);
        command_buffer.bind_graphics_pipeline(self.pipeline.clone());
        command_buffer.bind_descriptor_set(self.mvp_set.clone(), self.pipeline.clone(), 0);
        self.mesh_data.record_commands(command_buffer.clone());
        command_buffer.end_renderpass();
        command_buffer.end()?;

        self.context.graphics_queue.submit(
            &[self.frame_sync[current_frame].image_available_semaphore.clone()],
            &[self.image_sync[present_index].rendering_complete_semaphore.clone()],
            &[command_buffer],
            self.frame_sync[current_frame].command_buffer_fence.clone()
        )?;

        self.image_data.swapchain.present(
            self.context.graphics_queue.clone(),
            &[self.image_sync[present_index].rendering_complete_semaphore.clone()],
            present_index as u32,
        )?;

        self.current_frame = (self.current_frame + 1) % self.frames_in_flight;
        Ok(())
    }

    pub fn handle_resize(&mut self) -> Result<(), VulkanError> {
        self.resize = true;
        Ok(())
    }
}

impl Drop for VulkanRenderer {
    fn drop(&mut self) {
        self.context.device.wait_idle().unwrap_or(());  
    }
}
