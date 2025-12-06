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
            material_data::{MaterialBase, MaterialData},
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
            instance::Instance,
            physical_device::PhysicalDevice,
            queue::Queue,
            renderpass::Renderpass,
            surface::Surface,
            swapchain::Swapchain,
        },
        window::WindowWrapper,
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
    device: &Device,
    frames_in_flight: usize,
) -> Result<Vec<VulkanRendererFrameSync>, VulkanError> {
    let mut ret = Vec::new();
    for _ in 0..frames_in_flight {
        ret.push(VulkanRendererFrameSync::new(device)?);
    }
    Ok(ret)
}

fn create_image_sync(
    device: &Device,
    image_count: usize,
) -> Result<Vec<VulkanRendererImageSync>, VulkanError> {
    let mut ret = Vec::new();
    for _ in 0..image_count {
        ret.push(VulkanRendererImageSync::new(device)?);
    }
    Ok(ret)
}

fn create_commad_buffers(
    device: &Device,
    command_pool: &CommandPool,
    frames_in_flight: usize,
) -> Result<Vec<CommandBuffer>, VulkanError> {
    let mut ret = Vec::new();
    for _ in 0..frames_in_flight {
        ret.push(CommandBuffer::new(device, command_pool)?);
    }
    Ok(ret)
}

#[system]
pub fn init_renderer(
    instance: Resource<Instance>,
    surface: Resource<Surface>,
    device: Resource<Device>,
    physical_device: Resource<PhysicalDevice>,
    command_pool: Resource<CommandPool>,
    window: Resource<WindowWrapper>,
) {
    let renderpass = Renderpass::new(&surface, &device).unwrap();
    let renderpass = Resources::add(renderpass).unwrap();

    let (swapchain, swapchain_images) = init_renderer_images(
        &instance,
        &physical_device,
        &window,
        &surface,
        &device,
        &renderpass,
    )
    .unwrap();

    Resources::add(swapchain).unwrap();
    let swapchain_images = Resources::add(swapchain_images).unwrap();

    let frames_in_flight = 2.min(swapchain_images.len()).max(1);
    let frames_in_flight =
        Resources::add(FramesInFlight(frames_in_flight)).unwrap();

    let frame_sync = create_frame_sync(&device, frames_in_flight.0).unwrap();
    Resources::add(frame_sync).unwrap();

    let image_sync =
        create_image_sync(&device, swapchain_images.len()).unwrap();
    Resources::add(image_sync).unwrap();

    let command_buffers =
        create_commad_buffers(&device, &command_pool, frames_in_flight.0)
            .unwrap();
    Resources::add(command_buffers).unwrap();

    let descriptor_pool =
        DescriptorPool::new(&device, 32, &[DescriptorPoolSize::new(
            16,
            DescriptorType::UNIFORM_BUFFER,
        )])
        .unwrap();
    Resources::add(descriptor_pool).unwrap();

    let graphics_queue =
        Queue::present(&device, physical_device.get_queue_family_index());
    Resources::add(graphics_queue).unwrap();

    Resources::add(RendererCurrentFrame(0)).unwrap();
    Resources::add(RendererResizeFlag(false)).unwrap();
    Resources::add(Vec::<Draw>::new()).unwrap();
    Resources::add(IdVec::<MeshData<DefaultVertex>>::new()).unwrap();
    Resources::add(IdVec::<Mesh>::new()).unwrap();
    Resources::add(IdVec::<MaterialData>::new()).unwrap();
    Resources::add(IdVec::<MaterialBase>::new()).unwrap();
    Resources::add(IdVec::<TransformData>::new()).unwrap();
    Resources::add(IdVec::<CameraData>::new()).unwrap();
    Resources::add(IdVec::<Arc<DescriptorSet>>::new()).unwrap();
}

fn recreate_size_dependent_components(
    instance: &Instance,
    surface: &Surface,
    device: &Device,
    physical_device: &PhysicalDevice,
    window: &WindowWrapper,
    renderpass: &Renderpass,
    swapchain: &Swapchain,
) -> Result<(), VulkanError> {
    device.wait_idle()?;
    let (swapchain, swapchain_images) = recreate_renderer_images(
        instance,
        physical_device,
        window,
        surface,
        device,
        renderpass,
        swapchain,
    )?;

    Resources::add_or_change(swapchain);
    Resources::add_or_change(swapchain_images);

    Ok(())
}

#[system]
pub fn prepare_render(
    instance: Resource<Instance>,
    surface: Resource<Surface>,
    device: Resource<Device>,
    physical_device: Resource<PhysicalDevice>,
    window: Resource<WindowWrapper>,
    renderpass: Resource<Renderpass>,
    swapchain: Resource<Swapchain>,
    mut current_frame: Resource<RendererCurrentFrame>,
    frame_sync: Resource<Vec<VulkanRendererFrameSync>>,
    mut resize: Resource<RendererResizeFlag>,
) {
    *current_frame = {
        frame_sync[current_frame.0]
            .command_buffer_fence
            .wait(&device)
            .unwrap();
        *current_frame
    };

    if window.minimized() {
        return;
    }

    if resize.0 {
        recreate_size_dependent_components(
            &instance,
            &surface,
            &device,
            &physical_device,
            &window,
            &renderpass,
            &swapchain,
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
    swapchain: Resource<Swapchain>,
    command_buffers: Resource<Vec<CommandBuffer>>,
    framebuffers: Resource<Vec<Framebuffer>>,
    draw_queue: Resource<Vec<Draw>>,
    graphics_queue: Resource<Queue>,
    frames_in_flight: Resource<FramesInFlight>,
    meshes_data: Resource<IdVec<MeshData<DefaultVertex>>>,
    materials_data: Resource<IdVec<MaterialData>>,
    materials_base: Resource<IdVec<MaterialBase>>,
    transforms_data: Resource<IdVec<TransformData>>,
    cameras_data: Resource<IdVec<CameraData>>,
    device: Resource<Device>,
    renderpass: Resource<Renderpass>,
) {
    let present_index = {
        let (present_index, suboptimal) = swapchain
            .acquire_next_image(
                &frame_sync[current_frame.0]
                    .image_available_semaphore
                    .clone(),
                &Fence::null(&device),
            )
            .unwrap();
        resize.0 |= suboptimal;
        frame_sync[current_frame.0]
            .command_buffer_fence
            .reset(&device)
            .unwrap();
        present_index as usize
    };

    let command_buffer = &command_buffers[current_frame.0];
    let framebuffer = &framebuffers[present_index];

    command_buffer.reset(&device).unwrap();
    command_buffer.begin(&device).unwrap();
    command_buffer.begin_renderpass(&device, framebuffer, &renderpass);
    command_buffer.set_viewports(&device, framebuffer, &renderpass);
    command_buffer.set_scissor(&device, framebuffer, &renderpass);

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
                let material_base =
                    materials_base.get(material.material_base_id()).unwrap();
                let mesh = meshes_data.get(*mesh).unwrap();
                let camera = cameras_data.get(*camera).unwrap();
                let camera_transform =
                    transforms_data.get(*camera_transform).unwrap();
                let transform = transforms_data.get(*transform).unwrap();
                material.bind(
                    &device,
                    material_base,
                    command_buffer,
                    camera,
                    camera_transform,
                    transform,
                );
                mesh.record_commands(&device, command_buffer);
            },
        }
    }
    command_buffer.end_renderpass(&device);
    command_buffer.end(&device).unwrap();

    graphics_queue
        .submit(
            &device,
            &[&frame_sync[current_frame.0].image_available_semaphore],
            &[&image_sync[present_index].rendering_complete_semaphore],
            &[command_buffer],
            &frame_sync[current_frame.0].command_buffer_fence,
        )
        .unwrap();

    swapchain
        .present(
            &graphics_queue,
            &[&image_sync[present_index].rendering_complete_semaphore],
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
