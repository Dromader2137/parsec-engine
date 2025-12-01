//! The built-in renderer.

use std::sync::Arc;

pub mod assets;
pub mod camera_data;
pub mod components;
pub mod draw_queue;
pub mod image_data;
pub mod material_data;
pub mod mesh_data;
pub mod sync;
pub mod transform_data;

use sync::{VulkanRendererFrameSync, VulkanRendererImageSync};

use crate::{
    ecs::system::system,
    graphics::{
        renderer::{
            assets::mesh::Mesh,
            camera_data::CameraData,
            draw_queue::{Draw, MeshAndMaterial},
            image_data::{init_renderer_images, recreate_renderer_images},
            material_data::MaterialData,
            mesh_data::MeshData,
            transform_data::TransformData,
        },
        vulkan::{
            VulkanError,
            command_buffer::{CommandBuffer, CommandPool},
            descriptor_set::{
                DescriptorPool, DescriptorPoolSize, DescriptorSet,
                DescriptorType,
            },
            device::Device,
            fence::Fence,
            framebuffer::Framebuffer,
            graphics_pipeline::{Vertex, VertexField, VertexFieldFormat},
            physical_device::PhysicalDevice,
            queue::Queue,
            renderpass::Renderpass,
            surface::Surface,
            swapchain::Swapchain,
        },
    },
    math::vec::{Vec2f, Vec3f},
    resources::{Resource, Resources},
    utils::id_vec::IdVec,
};

#[derive(Debug, Clone, Copy)]
pub struct RendererResizeFlag(bool);
#[derive(Debug, Clone, Copy)]
pub struct RendererCurrentFrame(usize);
#[derive(Debug, Clone, Copy)]
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
    fn size() -> u32 { size_of::<f32>() as u32 * 11 }

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
    pub fn new(pos: Vec3f, nor: Vec3f, uv: Vec2f) -> DefaultVertex {
        DefaultVertex {
            position: [pos.x, pos.y, pos.z],
            normal: [nor.x, nor.y, nor.z],
            tangent: [0.0, 1.0, 0.0],
            uv: [uv.x, uv.y],
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

#[system]
pub fn init_renderer(
    surface: Resource<Arc<Surface>>,
    device: Resource<Arc<Device>>,
    physical_device: Resource<Arc<PhysicalDevice>>,
    command_pool: Resource<Arc<CommandPool>>,
) {
    let renderpass = Renderpass::new(surface.clone(), device.clone()).unwrap();
    Resources::add(renderpass.clone()).unwrap();
    let swapchain = init_renderer_images(renderpass.clone()).unwrap();
    let frames_in_flight = 2.min(swapchain.swapchain_images.len()).max(1);

    let frame_sync =
        create_frame_sync(device.clone(), frames_in_flight).unwrap();
    let image_sync =
        create_image_sync(device.clone(), swapchain.swapchain_images.len())
            .unwrap();
    let command_buffers =
        create_commad_buffers(command_pool.clone(), frames_in_flight).unwrap();

    let descriptor_pool =
        DescriptorPool::new(device.clone(), 32, &[DescriptorPoolSize::new(
            16,
            DescriptorType::UNIFORM_BUFFER,
        )])
        .unwrap();

    let graphics_queue = Queue::present(
        device.clone(),
        physical_device.get_queue_family_index(),
    );

    Resources::add(frame_sync).unwrap();
    Resources::add(image_sync).unwrap();
    Resources::add(command_buffers).unwrap();
    Resources::add(descriptor_pool).unwrap();
    Resources::add(graphics_queue).unwrap();
    Resources::add(FramesInFlight(frames_in_flight)).unwrap();
    Resources::add(RendererCurrentFrame(0)).unwrap();
    Resources::add(RendererResizeFlag(false)).unwrap();
    Resources::add(Vec::<Draw>::new()).unwrap();
    Resources::add(IdVec::<MeshData<DefaultVertex>>::new()).unwrap();
    Resources::add(IdVec::<Mesh>::new()).unwrap();
    Resources::add(IdVec::<MaterialData>::new()).unwrap();
    Resources::add(IdVec::<TransformData>::new()).unwrap();
    Resources::add(IdVec::<CameraData>::new()).unwrap();
    Resources::add(IdVec::<Arc<DescriptorSet>>::new()).unwrap();
}

fn recreate_size_dependent_components(
    device: Arc<Device>,
    renderpass: Arc<Renderpass>,
    old_swapchain: Arc<Swapchain>,
) -> Result<(), VulkanError> {
    device.wait_idle()?;
    recreate_renderer_images(renderpass, old_swapchain)
}

#[system]
pub fn prepare_render(
    mut current_frame: Resource<RendererCurrentFrame>,
    frame_sync: Resource<Vec<VulkanRendererFrameSync>>,
    mut resize: Resource<RendererResizeFlag>,
    surface: Resource<Arc<Surface>>,
    device: Resource<Arc<Device>>,
    swapchain: Resource<Arc<Swapchain>>,
    renderpass: Resource<Arc<Renderpass>>,
) {
    *current_frame = {
        frame_sync[current_frame.0]
            .command_buffer_fence
            .wait()
            .unwrap();
        *current_frame
    };

    if surface.window.minimized() {
        return;
    }

    if resize.0 {
        recreate_size_dependent_components(
            device.clone(),
            renderpass.clone(),
            swapchain.clone(),
        )
        .unwrap();
        *resize = RendererResizeFlag(false)
    }
}

#[system]
pub fn render(
    mut current_frame: Resource<RendererCurrentFrame>,
    frame_sync: Resource<Vec<VulkanRendererFrameSync>>,
    image_sync: Resource<Vec<VulkanRendererImageSync>>,
    mut resize: Resource<RendererResizeFlag>,
    swapchain: Resource<Arc<Swapchain>>,
    command_buffers: Resource<Vec<Arc<CommandBuffer>>>,
    framebuffers: Resource<Vec<Arc<Framebuffer>>>,
    draw_queue: Resource<Vec<Draw>>,
    graphics_queue: Resource<Arc<Queue>>,
    frames_in_flight: Resource<FramesInFlight>,
    meshes_data: Resource<IdVec<MeshData<DefaultVertex>>>,
    materials_data: Resource<IdVec<MaterialData>>,
    transforms_data: Resource<IdVec<TransformData>>,
    cameras_data: Resource<IdVec<CameraData>>,
) {
    let present_index = {
        let (present_index, suboptimal) = swapchain
            .acquire_next_image(
                frame_sync[current_frame.0]
                    .image_available_semaphore
                    .clone(),
                Fence::null(swapchain.device.clone()),
            )
            .unwrap();
        resize.0 |= suboptimal;
        frame_sync[current_frame.0]
            .command_buffer_fence
            .reset()
            .unwrap();
        present_index as usize
    };

    let command_buffer = command_buffers[current_frame.0].clone();
    let framebuffer = framebuffers[present_index].clone();

    command_buffer.reset().unwrap();
    command_buffer.begin().unwrap();
    command_buffer.begin_renderpass(framebuffer.clone());
    command_buffer.set_viewports(framebuffer.clone());
    command_buffer.set_scissor(framebuffer);

    for draw in draw_queue.iter() {
        match draw {
            Draw::MeshAndMaterial(MeshAndMaterial {
                mesh,
                material,
                camera,
                camera_transform,
                transform,
            }) => {
                let material = materials_data.get(*material).unwrap();
                let mesh = meshes_data.get(*mesh).unwrap();
                let camera = cameras_data.get(*camera).unwrap();
                let camera_transform =
                    transforms_data.get(*camera_transform).unwrap();
                let transform = transforms_data.get(*transform).unwrap();
                material.bind(
                    command_buffer.clone(),
                    camera,
                    camera_transform,
                    transform,
                );
                mesh.record_commands(command_buffer.clone());
            },
        }
    }
    command_buffer.end_renderpass();
    command_buffer.end().unwrap();

    graphics_queue
        .submit(
            &[frame_sync[current_frame.0]
                .image_available_semaphore
                .clone()],
            &[image_sync[present_index]
                .rendering_complete_semaphore
                .clone()],
            &[command_buffer],
            frame_sync[current_frame.0].command_buffer_fence.clone(),
        )
        .unwrap();

    swapchain
        .present(
            graphics_queue.clone(),
            &[image_sync[present_index]
                .rendering_complete_semaphore
                .clone()],
            present_index as u32,
        )
        .unwrap();

    current_frame.0 = (current_frame.0 + 1) % frames_in_flight.0;
}

#[system]
pub fn queue_clear(mut draw_queue: Resource<Vec<Draw>>) { draw_queue.clear(); }

#[derive(Debug)]
pub enum RendererError {
    ShaderNotFound(u32),
    BufferNotFound(u32),
    MaterialBaseNotFound(u32),
}

impl From<RendererError> for VulkanError {
    fn from(value: RendererError) -> Self { VulkanError::RendererError(value) }
}
