use std::sync::Arc;

pub mod draw_queue;
pub mod image_data;
pub mod sync;

use image_data::VulkanRendererImageData;
use sync::{VulkanRendererFrameSync, VulkanRendererImageSync};

use crate::{
    graphics::{
        camera::CameraData, mesh::MeshData, renderer::draw_queue::{Draw, MeshAndMaterial}, vulkan::{
            buffer::{Buffer, BufferUsage}, command_buffer::CommandBuffer, context::VulkanContext, descriptor_set::{
                DescriptorPool, DescriptorPoolSize, DescriptorSet, DescriptorSetBinding, DescriptorSetLayout,
                DescriptorType,
            }, fence::Fence, graphics_pipeline::{GraphicsPipeline, Vertex, VertexField, VertexFieldFormat}, renderpass::Renderpass, shader::{ShaderModule, ShaderType}, VulkanError
        }
    },
    math::vec::Vec3f, utils::id_vec::IdVec,
};

struct MaterialBase {
    pipeline: Arc<GraphicsPipeline>,
    descriptor_sets: Vec<Arc<DescriptorSet>>,
}

struct MaterialData {
    base: Arc<MaterialBase>,
    uniform_buffers: Vec<Vec<Arc<Buffer>>>,
}

impl MaterialData {
    fn bind(&self, command_buffer: Arc<CommandBuffer>) {
        command_buffer.bind_graphics_pipeline(self.base.pipeline.clone());
        for (idx, set) in self.base.descriptor_sets.iter().enumerate() {
            command_buffer.bind_descriptor_set(set.clone(), self.base.pipeline.clone(), idx as u32);
        }
    }

    fn bind_buffers(&self) -> Result<(), VulkanError> {
        for (idx, set) in self.base.descriptor_sets.iter().enumerate() {
            for (buffer_idx, buffer) in self.uniform_buffers[idx].iter().enumerate() {
                set.bind_buffer(buffer.clone(), buffer_idx as u32)?;
            }
        }
        Ok(())
    }
}

pub struct VulkanRenderer {
    //Vulkan stuff
    context: Arc<VulkanContext>,
    renderpass: Arc<Renderpass>,
    image_data: VulkanRendererImageData,
    frame_sync: Vec<VulkanRendererFrameSync>,
    image_sync: Vec<VulkanRendererImageSync>,

    //Resources
    command_buffers: Vec<Arc<CommandBuffer>>,
    material_bases: IdVec<Arc<MaterialBase>>,
    meshes: IdVec<MeshData<DefaultVertex>>,
    shaders: IdVec<Arc<ShaderModule>>,
    materials: IdVec<MaterialData>,
    buffers: IdVec<Arc<Buffer>>,
    cameras: IdVec<CameraData>,
    transforms: IdVec<>
    descriptor_pool: Arc<DescriptorPool>,

    //State
    resize: bool,
    current_frame: usize,
    frames_in_flight: usize,

    draw_queue: Vec<Draw>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DefaultVertex {
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
    pub fn new(pos: Vec3f, nor: Vec3f) -> DefaultVertex {
        DefaultVertex {
            position: [pos.x, pos.y, pos.z],
            normal: [nor.x, nor.y, nor.z],
            tangent: [0.0, 1.0, 0.0],
            uv: [0.0, 0.0],
        }
    }
}

fn create_frame_sync(
    context: Arc<VulkanContext>,
    frames_in_flight: usize,
) -> Result<Vec<VulkanRendererFrameSync>, VulkanError> {
    let mut ret = Vec::new();
    for _ in 0..frames_in_flight {
        ret.push(VulkanRendererFrameSync::new(context.device.clone())?);
    }
    Ok(ret)
}

fn create_image_sync(
    context: Arc<VulkanContext>,
    image_count: usize,
) -> Result<Vec<VulkanRendererImageSync>, VulkanError> {
    let mut ret = Vec::new();
    for _ in 0..image_count {
        ret.push(VulkanRendererImageSync::new(context.device.clone())?);
    }
    Ok(ret)
}

fn create_commad_buffers(
    context: Arc<VulkanContext>,
    frames_in_flight: usize,
) -> Result<Vec<Arc<CommandBuffer>>, VulkanError> {
    let mut ret = Vec::new();
    for _ in 0..frames_in_flight {
        ret.push(CommandBuffer::new(context.command_pool.clone())?);
    }
    Ok(ret)
}

impl VulkanRenderer {
    pub fn new(context: Arc<VulkanContext>) -> Result<VulkanRenderer, VulkanError> {
        let mut frames_in_flight = 2;
        let renderpass = Renderpass::new(context.surface.clone(), context.device.clone())?;
        let image_data = VulkanRendererImageData::new(context.clone(), renderpass.clone())?;
        frames_in_flight = image_data.clamp_frames_in_flight(frames_in_flight);
        let frame_sync = create_frame_sync(context.clone(), frames_in_flight)?;
        let image_sync = create_image_sync(context.clone(), image_data.swapchain.swapchain_images.len())?;
        let command_buffers = create_commad_buffers(context.clone(), frames_in_flight)?;

        let descriptor_pool = DescriptorPool::new(
            context.device.clone(),
            32,
            &[DescriptorPoolSize::new(16, DescriptorType::UNIFORM_BUFFER)],
        )?;

        Ok(VulkanRenderer {
            context,
            renderpass,
            image_data,
            frame_sync,
            image_sync,
            command_buffers,
            material_bases: IdVec::new(),
            shaders: IdVec::new(),
            meshes: IdVec::new(),
            materials: IdVec::new(),
            buffers: IdVec::new(),
            descriptor_pool,
            frames_in_flight,
            resize: false,
            current_frame: 0,
            draw_queue: Vec::new(),
        })
    }

    pub fn recreate_size_dependent_components(&mut self) -> Result<(), VulkanError> {
        self.context.device.wait_idle()?;

        self.image_data
            .recreate(self.context.clone(), self.renderpass.clone())?;

        Ok(())
    }

    pub fn render(&mut self) -> Result<(), VulkanError> {
        let current_frame = self.current_frame as usize;
        self.frame_sync[current_frame].command_buffer_fence.wait()?;

        if self.context.surface.window.minimized() {
            return Ok(());
        }

        if self.resize {
            self.recreate_size_dependent_components()?;
            self.resize = false;
        }

        let (present_index, suboptimal) = self.image_data.swapchain.acquire_next_image(
            self.frame_sync[current_frame].image_available_semaphore.clone(),
            Fence::null(self.context.device.clone()),
        )?;
        let present_index = present_index as usize;

        self.resize |= suboptimal;
        self.frame_sync[current_frame].command_buffer_fence.reset()?;

        let command_buffer = self.command_buffers[current_frame].clone();
        let framebuffer = self.image_data.framebuffers[present_index].clone();

        command_buffer.reset()?;
        command_buffer.begin()?;
        command_buffer.begin_renderpass(framebuffer.clone());
        command_buffer.set_viewports(framebuffer.clone());
        command_buffer.set_scissor(framebuffer);
        for draw in self.draw_queue.iter() {
            match draw {
                Draw::MeshAndMaterial(MeshAndMaterial {
                    mesh_id,
                    material_id,
                }) => {
                    let material = self.materials.get(*material_id).unwrap();
                    material.bind(command_buffer.clone());
                    let mesh = self.meshes.get(*mesh_id).unwrap();
                    mesh.record_commands(command_buffer.clone());
                }
            }
        }
        command_buffer.end_renderpass();
        command_buffer.end()?;

        self.context.graphics_queue.submit(
            &[self.frame_sync[current_frame].image_available_semaphore.clone()],
            &[self.image_sync[present_index].rendering_complete_semaphore.clone()],
            &[command_buffer],
            self.frame_sync[current_frame].command_buffer_fence.clone(),
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

    pub fn create_shader(&mut self, code: &[u32], shader_type: ShaderType) -> Result<u32, VulkanError> {
        Ok(self.shaders.push(
            ShaderModule::new(self.context.device.clone(), code, shader_type)?
        ))
    }

    fn get_shader(&self, shader_id: u32) -> Result<Arc<ShaderModule>, RendererError> {
        self.shaders
            .get(shader_id)
            .ok_or(RendererError::ShaderNotFound(shader_id))
            .cloned()
    }

    pub fn create_material_base(
        &mut self,
        vertex_id: u32,
        fragment_id: u32,
        layout: Vec<Vec<DescriptorSetBinding>>,
    ) -> Result<u32, VulkanError> {
        let mut descriptors = Vec::new();

        for bindings in layout {
            descriptors.push(DescriptorSetLayout::new(self.context.device.clone(), bindings)?);
        }

        let pipeline = GraphicsPipeline::new::<DefaultVertex>(
            self.image_data.framebuffers[0].clone(),
            self.get_shader(vertex_id)?,
            self.get_shader(fragment_id)?,
            descriptors,
        )?;

        let descriptor_sets = {
            let mut sets = Vec::new();
            for set_layout in pipeline.descriptor_set_layouts.iter() {
                sets.push(DescriptorSet::new(set_layout.clone(), self.descriptor_pool.clone())?);
            }
            sets
        };

        let base = Arc::new(MaterialBase {
            pipeline,
            descriptor_sets,
        });

        Ok(self.material_bases.push(base))
    }

    fn get_material_base(&self, material_base_id: u32) -> Result<Arc<MaterialBase>, RendererError> {
        self.material_bases
            .get(material_base_id)
            .ok_or(RendererError::MaterialBaseNotFound(material_base_id))
            .cloned()
    }

    pub fn create_buffer<T: Copy + Clone>(&mut self, data: Vec<T>) -> Result<u32, VulkanError> {
        Ok(self.buffers.push(
            Buffer::from_vec(self.context.device.clone(), data, BufferUsage::UNIFORM_BUFFER)?
        ))
    }

    fn get_buffer(&self, buffer_id: u32) -> Result<Arc<Buffer>, RendererError> {
        self.buffers
            .get(buffer_id)
            .ok_or(RendererError::BufferNotFound(buffer_id))
            .cloned()
    }

    pub fn create_material(
        &mut self,
        material_base_id: u32,
        buffer_ids: Vec<Vec<u32>>,
    ) -> Result<u32, VulkanError> {
        let base = self.get_material_base(material_base_id)?;
        let mut buffer_sets = Vec::new();
        for buffer_set in buffer_ids {
            buffer_sets.push(Vec::new());
            for buffer in buffer_set {
                buffer_sets.last_mut().unwrap().push(self.get_buffer(buffer)?);
            }
        }

        let material_data = MaterialData {
            base,
            uniform_buffers: buffer_sets,
        };

        material_data.bind_buffers()?;

        Ok(self.materials.push(material_data))
    }

    pub fn create_mesh(
        &mut self,
        vertices: Vec<DefaultVertex>,
        indices: Vec<u32>,
    ) -> Result<u32, VulkanError> {
        Ok(self.meshes.push(
            MeshData::new(self.context.device.clone(), vertices, indices)?,
        ))
    }

    pub fn update_buffer<T: Copy + Clone>(&self, buffer_id: u32, data: Vec<T>) -> Result<(), VulkanError> {
        let buffer = self.get_buffer(buffer_id)?;
        buffer.update(data)?;
        Ok(())
    }

    pub fn get_aspect_ratio(&self) -> f32 {
        self.image_data.framebuffers[0].renderpass.surface.aspect_ratio()
    }

    pub fn queue_draw(&mut self, draw: Draw) {
        self.draw_queue.push(draw);
    }

    pub fn queue_clear(&mut self) {
        self.draw_queue.clear();
    }
}

impl Drop for VulkanRenderer {
    fn drop(&mut self) {
        self.context.device.wait_idle().unwrap_or(());
    }
}

#[derive(Debug)]
pub enum RendererError {
    ShaderNotFound(u32),
    BufferNotFound(u32),
    MaterialBaseNotFound(u32),
}

impl From<RendererError> for VulkanError {
    fn from(value: RendererError) -> Self {
        VulkanError::RendererError(value)
    }
}
