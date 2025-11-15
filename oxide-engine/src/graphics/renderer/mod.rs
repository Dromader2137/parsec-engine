use std::sync::Arc;

pub mod draw_queue;
pub mod image_data;
pub mod sync;

use sync::{VulkanRendererFrameSync, VulkanRendererImageSync};

use crate::{
    graphics::{
        camera::CameraData,
        material::{MaterialBase, MaterialData},
        mesh::MeshData,
        renderer::{
            draw_queue::{Draw, MeshAndMaterial},
            image_data::{init_renderer_images, recreate_renderer_images},
        },
        transform::TransformData,
        vulkan::{
            VulkanError,
            buffer::{Buffer, BufferUsage},
            command_buffer::{CommandBuffer, CommandPool},
            descriptor_set::{
                DescriptorPool, DescriptorPoolSize, DescriptorSet, DescriptorSetBinding,
                DescriptorSetLayout, DescriptorType,
            },
            device::Device,
            fence::Fence,
            framebuffer::Framebuffer,
            graphics_pipeline::{Vertex, VertexField, VertexFieldFormat},
            physical_device::PhysicalDevice,
            queue::Queue,
            renderpass::Renderpass,
            shader::{ShaderModule, ShaderType},
            surface::Surface,
            swapchain::Swapchain,
        },
    },
    math::vec::Vec3f,
    resources::ResourceCollection,
    utils::id_vec::IdVec,
};

pub struct RendererResizeFlag(bool);
pub struct RendererCurrentFrame(usize);
pub struct FramesInFlight(usize);

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
    device: Arc<Device>,
    frames_in_flight: usize,
) -> Result<Vec<VulkanRendererFrameSync>, VulkanError> {
    let mut ret = Vec::new();
    for _ in 0..frames_in_flight {
        ret.push(VulkanRendererFrameSync::new(device.clone())?);
    }
    Ok(ret)
}

fn create_image_sync(
    device: Arc<Device>,
    image_count: usize,
) -> Result<Vec<VulkanRendererImageSync>, VulkanError> {
    let mut ret = Vec::new();
    for _ in 0..image_count {
        ret.push(VulkanRendererImageSync::new(device.clone())?);
    }
    Ok(ret)
}

fn create_commad_buffers(
    command_pool: Arc<CommandPool>,
    frames_in_flight: usize,
) -> Result<Vec<Arc<CommandBuffer>>, VulkanError> {
    let mut ret = Vec::new();
    for _ in 0..frames_in_flight {
        ret.push(CommandBuffer::new(command_pool.clone())?);
    }
    Ok(ret)
}

pub fn init_renderer(resources: &mut ResourceCollection) -> Result<(), VulkanError> {
    let renderpass = {
        let surface = resources.get::<Arc<Surface>>().unwrap();
        let device = resources.get::<Arc<Device>>().unwrap();
        Renderpass::new(surface.clone(), device.clone())?
    };
    resources.add(renderpass).unwrap();
    init_renderer_images(resources).unwrap();
    let frames_in_flight = {
        let swapchain = resources.get::<Arc<Swapchain>>().unwrap();
        2.min(swapchain.swapchain_images.len()).max(1)
    };
    let (frame_sync, image_sync) = {
        let device = resources.get::<Arc<Device>>().unwrap();
        let swapchain = resources.get::<Arc<Swapchain>>().unwrap();
        let frame_sync = create_frame_sync(device.clone(), frames_in_flight)?;
        let image_sync = create_image_sync(device.clone(), swapchain.swapchain_images.len())?;
        (frame_sync, image_sync)
    };
    let command_buffers = {
        let command_pool = resources.get::<Arc<CommandPool>>().unwrap();
        create_commad_buffers(command_pool.clone(), frames_in_flight)?
    };

    let descriptor_pool = {
        let device = resources.get::<Arc<Device>>().unwrap();
        DescriptorPool::new(device.clone(), 32, &[DescriptorPoolSize::new(
            16,
            DescriptorType::UNIFORM_BUFFER,
        )])?
    };

    let graphics_queue = {
        let physical_device = resources.get::<Arc<PhysicalDevice>>().unwrap();
        let device = resources.get::<Arc<Device>>().unwrap();
        Queue::present(device.clone(), physical_device.get_queue_family_index())
    };

    resources.add(frame_sync).unwrap();
    resources.add(image_sync).unwrap();
    resources.add(command_buffers).unwrap();
    resources.add(descriptor_pool).unwrap();
    resources.add(graphics_queue).unwrap();
    resources.add(FramesInFlight(frames_in_flight)).unwrap();
    resources.add(RendererCurrentFrame(0)).unwrap();
    resources.add(RendererResizeFlag(false)).unwrap();
    resources.add(Vec::<Draw>::new()).unwrap();
    resources.add(IdVec::<Arc<MaterialBase>>::new()).unwrap();
    resources.add(IdVec::<Arc<ShaderModule>>::new()).unwrap();
    resources.add(IdVec::<Arc<Buffer>>::new()).unwrap();
    resources
        .add(IdVec::<MeshData<DefaultVertex>>::new())
        .unwrap();
    resources.add(IdVec::<MaterialData>::new()).unwrap();
    resources.add(IdVec::<TransformData>::new()).unwrap();
    resources.add(IdVec::<CameraData>::new()).unwrap();
    resources.add(IdVec::<Arc<DescriptorSet>>::new()).unwrap();

    Ok(())
}

pub fn create_descriptor_set(
    resources: &mut ResourceCollection,
    descriptor_set_bindings: Vec<DescriptorSetBinding>,
) -> Result<u32, VulkanError> {
    let descriptor_pool = resources.get::<Arc<DescriptorPool>>().unwrap();
    let device = resources.get::<Arc<Device>>().unwrap();
    let descriptor_layout = DescriptorSetLayout::new(device.clone(), descriptor_set_bindings)?;
    let descriptor_set = DescriptorSet::new(descriptor_layout, descriptor_pool.clone())?;
    let mut descriptor_sets = resources.get_mut::<IdVec<Arc<DescriptorSet>>>().unwrap();
    Ok(descriptor_sets.push(descriptor_set))
}

fn recreate_size_dependent_components(
    resources: &mut ResourceCollection,
) -> Result<(), VulkanError> {
    {
        let renderpass = resources.get::<Arc<Renderpass>>().unwrap();
        renderpass.device.wait_idle()?;
    }

    recreate_renderer_images(resources)?;

    Ok(())
}

pub fn render(resources: &mut ResourceCollection) -> Result<(), VulkanError> {
    let current_frame = {
        let current_frame = resources.get::<RendererCurrentFrame>().unwrap();
        let frame_sync = resources.get::<Vec<VulkanRendererFrameSync>>().unwrap();
        frame_sync[current_frame.0].command_buffer_fence.wait()?;
        current_frame.0
    };

    {
        let surface = resources.get::<Arc<Surface>>().unwrap();
        if surface.window.minimized() {
            return Ok(());
        }
    }

    {
        let resize = { resources.get::<RendererResizeFlag>().unwrap().0 };
        if resize {
            recreate_size_dependent_components(resources)?;
            resources.get_mut::<RendererResizeFlag>().unwrap().0 = false;
        }
    }

    let present_index = {
        let swapchain = resources.get::<Arc<Swapchain>>().unwrap();
        let frame_sync = resources.get::<Vec<VulkanRendererFrameSync>>().unwrap();
        let mut resize = resources.get_mut::<RendererResizeFlag>().unwrap();
        let (present_index, suboptimal) = swapchain.acquire_next_image(
            frame_sync[current_frame].image_available_semaphore.clone(),
            Fence::null(swapchain.device.clone()),
        )?;
        resize.0 |= suboptimal;
        frame_sync[current_frame].command_buffer_fence.reset()?;
        present_index as usize
    };

    let command_buffer = {
        let command_buffers = resources.get::<Vec<Arc<CommandBuffer>>>().unwrap();
        command_buffers[current_frame].clone()
    };
    let framebuffer = {
        let framebuffers = resources.get::<Vec<Arc<Framebuffer>>>().unwrap();
        framebuffers[present_index].clone()
    };

    {
        command_buffer.reset()?;
        command_buffer.begin()?;
        command_buffer.begin_renderpass(framebuffer.clone());
        command_buffer.set_viewports(framebuffer.clone());
        command_buffer.set_scissor(framebuffer);

        let draw_queue = resources.get::<Vec<Draw>>().unwrap();
        let materials = resources.get::<IdVec<MaterialData>>().unwrap();
        let meshes = resources.get::<IdVec<MeshData<DefaultVertex>>>().unwrap();

        for draw in draw_queue.iter() {
            match draw {
                Draw::MeshAndMaterial(MeshAndMaterial {
                    mesh_id,
                    material_id,
                    camera_id,
                    transform_id,
                }) => {
                    let material = materials.get(*material_id).unwrap();
                    let mesh = meshes.get(*mesh_id).unwrap();
                    material.bind(resources, command_buffer.clone(), *camera_id, *transform_id);
                    mesh.record_commands(command_buffer.clone());
                }
            }
        }
        command_buffer.end_renderpass();
        command_buffer.end()?;
    }

    {
        let swapchain = resources.get::<Arc<Swapchain>>().unwrap();
        let graphics_queue = resources.get::<Arc<Queue>>().unwrap();
        let frame_sync = resources.get::<Vec<VulkanRendererFrameSync>>().unwrap();
        let image_sync = resources.get::<Vec<VulkanRendererImageSync>>().unwrap();
        graphics_queue.submit(
            &[frame_sync[current_frame].image_available_semaphore.clone()],
            &[image_sync[present_index]
                .rendering_complete_semaphore
                .clone()],
            &[command_buffer],
            frame_sync[current_frame].command_buffer_fence.clone(),
        )?;

        swapchain.present(
            graphics_queue.clone(),
            &[image_sync[present_index]
                .rendering_complete_semaphore
                .clone()],
            present_index as u32,
        )?;
    }

    {
        let mut current_frame = resources.get_mut::<RendererCurrentFrame>().unwrap();
        let frames_in_flight = resources.get::<FramesInFlight>().unwrap();
        current_frame.0 = (current_frame.0 + 1) % frames_in_flight.0;
    }
    Ok(())
}

pub fn create_shader(
    resources: &mut ResourceCollection,
    code: &[u32],
    shader_type: ShaderType,
) -> Result<u32, VulkanError> {
    let shader = {
        let device = resources.get::<Arc<Device>>().unwrap();
        ShaderModule::new(device.clone(), code, shader_type)?
    };

    let mut shader_modules = resources.get_mut::<IdVec<Arc<ShaderModule>>>().unwrap();
    Ok(shader_modules.push(shader))
}

pub fn create_buffer<T: Copy + Clone>(
    resources: &mut ResourceCollection,
    data: Vec<T>,
) -> Result<u32, VulkanError> {
    let device = resources.get::<Arc<Device>>().unwrap();
    let mut buffers = resources.get_mut::<IdVec<Arc<Buffer>>>().unwrap();
    Ok(buffers.push(Buffer::from_vec(
        device.clone(),
        data,
        BufferUsage::UNIFORM_BUFFER,
    )?))
}

pub fn create_mesh(
    resources: &mut ResourceCollection,
    vertices: Vec<DefaultVertex>,
    indices: Vec<u32>,
) -> Result<u32, VulkanError> {
    let mut materials = resources
        .get_mut::<IdVec<MeshData<DefaultVertex>>>()
        .unwrap();
    let device = resources.get::<Arc<Device>>().unwrap();
    Ok(materials.push(MeshData::new(device.clone(), vertices, indices)?))
}

pub fn update_buffer<T: Copy + Clone>(
    resources: &mut ResourceCollection,
    buffer_id: u32,
    data: Vec<T>,
) -> Result<(), VulkanError> {
    let mut buffers = resources.get_mut::<IdVec<Arc<Buffer>>>().unwrap();
    let buffer = buffers.get_mut(buffer_id).unwrap();
    buffer.update(data)?;
    Ok(())
}

pub fn get_aspect_ratio(resources: &mut ResourceCollection) -> f32 {
    let surface = resources.get::<Arc<Surface>>().unwrap();
    surface.aspect_ratio()
}

pub fn queue_draw(resources: &mut ResourceCollection, draw: Draw) {
    let mut draw_queue = resources.get_mut::<Vec<Draw>>().unwrap();
    draw_queue.push(draw);
}

pub fn queue_clear(resources: &mut ResourceCollection) {
    let mut draw_queue = resources.get_mut::<Vec<Draw>>().unwrap();
    draw_queue.clear();
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
