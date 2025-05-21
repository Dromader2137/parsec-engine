use std::f32;

pub mod camera;
pub mod images;
pub mod material;
pub mod sync;

use images::VulkanRendererFrameData;
use sync::VulkanRendererSync;

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

pub struct VulkanRenderer {
    renderpass: Renderpass,
    frame_data: VulkanRendererFrameData,
    sync: VulkanRendererSync,
    command_buffers: Vec<CommandBuffer>,
    pipeline: GraphicsPipeline,
    descriptor_pool: DescriptorPool,
    mvp_set: DescriptorSet,
    mvp_buffer: Buffer<Matrix4f>,
    mesh_data: MeshData<DefaultVertex>,
    vertex_shader: ShaderModule,
    fragment_shader: ShaderModule,
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

impl VulkanRenderer {
    pub fn new(
        context: &VulkanContext,
        window: &WindowWrapper,
    ) -> Result<VulkanRenderer, VulkanError> {
        let mut frames_in_flight = 3;
        let renderpass = Renderpass::new(&context.surface, &context.device)?;
        let frame_data = VulkanRendererFrameData::new(context, window, &renderpass)?;
        frames_in_flight = frame_data.clamp_frames_in_flight(frames_in_flight);
        let sync = VulkanRendererSync::new(context, frames_in_flight, frame_data.swapchain_images.len())?;
        let command_buffers = {
            let mut out = Vec::new();
            for _ in 0..frame_data.swapchain_images.len() {
                out.push(CommandBuffer::new(&context.device, &context.command_pool)?);
            }
            out
        };
        let vertex_shader =
            ShaderModule::new(&context.device, &read_shader_code("shaders/simple.spv")?)?;
        let fragment_shader =
            ShaderModule::new(&context.device, &read_shader_code("shaders/flat.spv")?)?;

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
            &context.instance,
            &context.physical_device,
            &context.device,
            vertices,
            indices,
        )?;

        let mvp_buffer = Buffer::from_vec(
            &context.instance,
            &context.physical_device,
            &context.device,
            vec![Matrix4f::indentity()],
            BufferUsage::UNIFORM_BUFFER,
        )?;

        let descriptor_pool = DescriptorPool::new(
            &context.device,
            32,
            &[DescriptorPoolSize::new(16, DescriptorType::UNIFORM_BUFFER)],
        )?;
        let bindings = vec![DescriptorSetBinding::new(
            0,
            DescriptorType::UNIFORM_BUFFER,
            DescriptorStage::VERTEX,
        )];
        let mvp_set_layout = DescriptorSetLayout::new(&context.device, bindings)?;
        let mvp_set = DescriptorSet::new(&context.device, mvp_set_layout, &descriptor_pool)?;
        mvp_set.bind_buffer(&context.device, &mvp_buffer, 0)?;

        let pipeline = GraphicsPipeline::new::<DefaultVertex>(
            &context.device,
            &frame_data.framebuffers[0],
            &renderpass,
            &vertex_shader,
            &fragment_shader,
            &[mvp_set.layout.clone()],
        )?;

        Ok(VulkanRenderer {
            renderpass,
            frame_data,
            sync,
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
        context: &VulkanContext,
        window: &WindowWrapper,
    ) -> Result<(), VulkanError> {
        context.device.wait_idle()?;

        self.frame_data
            .recreate(context, window, &self.renderpass)?;

        Ok(())
    }

    pub fn cleanup(&mut self, context: &VulkanContext) -> Result<(), VulkanError> {
        context.device.wait_idle()?;
        self.renderpass.cleanup(&context.device);
        self.fragment_shader.cleanup(&context.device);
        self.vertex_shader.cleanup(&context.device);
        self.pipeline.cleanup(&context.device);
        self.frame_data.cleanup(&context.device);
        self.sync.cleanup(&context.device);
        self.mesh_data.cleanup(&context.device);
        self.mvp_set
            .cleanup(&context.device, &self.descriptor_pool)?;
        self.descriptor_pool.cleanup(&context.device);
        self.mvp_buffer.cleanup(&context.device);
        Ok(())
    }

    pub fn render(
        &mut self,
        context: &VulkanContext,
        window: &WindowWrapper,
    ) -> Result<(), VulkanError> {
        let current_frame = self.current_frame;
        let sync_bundle = self.sync.get_frame_sync_bundle(current_frame);
        sync_bundle.command_buffer_fence.wait(&context.device)?;
        println!("{}", self.sync.frame_to_image_mapping[current_frame]);

        if window.minimized() {
            return Ok(());
        }

        if self.resize {
            self.recreate_size_dependent_components(context, window)?;
            self.resize = false;
        }

        let (present_index, suboptimal) = self
            .frame_data
            .swapchain
            .acquire_next_image(&sync_bundle.image_available_semaphore, &Fence::null())?;
        let present_index = present_index as usize;
        self.sync.frame_to_image_mapping[current_frame] = present_index;

        self.resize |= suboptimal;
        sync_bundle.command_buffer_fence.reset(&context.device)?;

        let command_buffer = &self.command_buffers[current_frame];
        let framebuffer = &self.frame_data.framebuffers[present_index];

        let (width, height) = (window.get_width(), window.get_height());
        let aspect = width as f32 / height as f32;
        self.mvp_buffer.update(
            &context.device,
            vec![
                Matrix4f::perspective(40.0_f32.to_radians(), aspect, 5.0, 100.0)
                    * Matrix4f::look_at(Vec3f::ZERO, Vec3f::FORWARD, Vec3f::UP)
                    * Matrix4f::translation(Vec3f::FORWARD * 30.0)
            ],
        )?;

        command_buffer.reset(&context.device)?;
        command_buffer.begin(&context.device)?;
        command_buffer.begin_renderpass(&context.device, &self.renderpass, framebuffer);
        command_buffer.set_viewports(&context.device, framebuffer);
        command_buffer.set_scissor(&context.device, framebuffer);
        command_buffer.bind_graphics_pipeline(&context.device, &self.pipeline);
        command_buffer.bind_descriptor_set(&context.device, &self.mvp_set, &self.pipeline, 0);
        self.mesh_data.record_commands(&context.device, command_buffer);
        command_buffer.end_renderpass(&context.device);
        command_buffer.end(&context.device)?;

        context.graphics_queue.submit(
            &context.device,
            &[&sync_bundle.image_available_semaphore, &sync_bundle.rendering_complete_semaphore],
            &[&sync_bundle.rendering_complete_semaphore],
            &[command_buffer],
            &sync_bundle.command_buffer_fence
        )?;

        self.frame_data.swapchain.present(
            &context.graphics_queue,
            &[&sync_bundle.rendering_complete_semaphore],
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
