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
    graphics::{
        renderer::{
            camera_data::CameraData,
            draw_queue::{Draw, MeshAndMaterial},
            image_data::{init_renderer_images, recreate_renderer_images},
            material_data::{MaterialBase, MaterialData},
            mesh_data::MeshData,
            transform_data::TransformData,
        },
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
    resources::{Rsc, RscMut},
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

pub fn init_renderer() -> Result<(), VulkanError> {
    let renderpass = {
        let surface = Rsc::<Arc<Surface>>::get().unwrap();
        let device = Rsc::<Arc<Device>>::get().unwrap();
        Renderpass::new(surface.clone(), device.clone())?
    };
    Rsc::add(renderpass).unwrap();
    init_renderer_images().unwrap();
    let frames_in_flight = {
        let swapchain = Rsc::<Arc<Swapchain>>::get().unwrap();
        2.min(swapchain.swapchain_images.len()).max(1)
    };
    let (frame_sync, image_sync) = {
        let device = Rsc::<Arc<Device>>::get().unwrap();
        let swapchain = Rsc::<Arc<Swapchain>>::get().unwrap();
        let frame_sync = create_frame_sync(device.clone(), frames_in_flight)?;
        let image_sync = create_image_sync(device.clone(), swapchain.swapchain_images.len())?;
        (frame_sync, image_sync)
    };
    let command_buffers = {
        let command_pool = Rsc::<Arc<CommandPool>>::get().unwrap();
        create_commad_buffers(command_pool.clone(), frames_in_flight)?
    };

    let descriptor_pool = {
        let device = Rsc::<Arc<Device>>::get().unwrap();
        DescriptorPool::new(device.clone(), 32, &[DescriptorPoolSize::new(
            16,
            DescriptorType::UNIFORM_BUFFER,
        )])?
    };

    let graphics_queue = {
        let physical_device = Rsc::<Arc<PhysicalDevice>>::get().unwrap();
        let device = Rsc::<Arc<Device>>::get().unwrap();
        Queue::present(device.clone(), physical_device.get_queue_family_index())
    };

    Rsc::add(frame_sync).unwrap();
    Rsc::add(image_sync).unwrap();
    Rsc::add(command_buffers).unwrap();
    Rsc::add(descriptor_pool).unwrap();
    Rsc::add(graphics_queue).unwrap();
    Rsc::add(FramesInFlight(frames_in_flight)).unwrap();
    Rsc::add(RendererCurrentFrame(0)).unwrap();
    Rsc::add(RendererResizeFlag(false)).unwrap();
    Rsc::add(Vec::<Draw>::new()).unwrap();
    Rsc::add(IdVec::<Arc<MaterialBase>>::new()).unwrap();
    Rsc::add(IdVec::<Arc<ShaderModule>>::new()).unwrap();
    Rsc::add(IdVec::<Arc<Buffer>>::new()).unwrap();
    Rsc::add(IdVec::<MeshData<DefaultVertex>>::new())
        .unwrap();
    Rsc::add(IdVec::<MaterialData>::new()).unwrap();
    Rsc::add(IdVec::<TransformData>::new()).unwrap();
    Rsc::add(IdVec::<CameraData>::new()).unwrap();
    Rsc::add(IdVec::<Arc<DescriptorSet>>::new()).unwrap();

    Ok(())
}

pub fn create_descriptor_set(
    descriptor_set_bindings: Vec<DescriptorSetBinding>,
) -> Result<u32, VulkanError> {
    let descriptor_pool = Rsc::<Arc<DescriptorPool>>::get().unwrap();
    let device = Rsc::<Arc<Device>>::get().unwrap();
    let descriptor_layout = DescriptorSetLayout::new(device.clone(), descriptor_set_bindings)?;
    let descriptor_set = DescriptorSet::new(descriptor_layout, descriptor_pool.clone())?;
    let mut descriptor_sets = RscMut::<IdVec<Arc<DescriptorSet>>>::get().unwrap();
    Ok(descriptor_sets.push(descriptor_set))
}

fn recreate_size_dependent_components(
) -> Result<(), VulkanError> {
    {
        let device = Rsc::<Arc<Device>>::get().unwrap();
        device.wait_idle()?;
    }

    recreate_renderer_images()?;

    Ok(())
}

pub fn render() -> Result<(), VulkanError> {
    let current_frame = {
        let current_frame = Rsc::<RendererCurrentFrame>::get().unwrap();
        let frame_sync = Rsc::<Vec<VulkanRendererFrameSync>>::get().unwrap();
        frame_sync[current_frame.0].command_buffer_fence.wait()?;
        current_frame.0
    };

    {
        let surface = Rsc::<Arc<Surface>>::get().unwrap();
        if surface.window.minimized() {
            return Ok(());
        }
    }

    {
        let resize = { Rsc::<RendererResizeFlag>::get().unwrap().0 };
        if resize {
            recreate_size_dependent_components()?;
            RscMut::<RendererResizeFlag>::get().unwrap().0 = false;
        }
    }

    let present_index = {
        let swapchain = Rsc::<Arc<Swapchain>>::get().unwrap();
        let frame_sync = Rsc::<Vec<VulkanRendererFrameSync>>::get().unwrap();
        let mut resize = RscMut::<RendererResizeFlag>::get().unwrap();
        let (present_index, suboptimal) = swapchain.acquire_next_image(
            frame_sync[current_frame].image_available_semaphore.clone(),
            Fence::null(swapchain.device.clone()),
        )?;
        resize.0 |= suboptimal;
        frame_sync[current_frame].command_buffer_fence.reset()?;
        present_index as usize
    };

    let command_buffer = {
        let command_buffers = Rsc::<Vec<Arc<CommandBuffer>>>::get().unwrap();
        command_buffers[current_frame].clone()
    };
    let framebuffer = {
        let framebuffers = Rsc::<Vec<Arc<Framebuffer>>>::get().unwrap();
        framebuffers[present_index].clone()
    };

    {
        command_buffer.reset()?;
        command_buffer.begin()?;
        command_buffer.begin_renderpass(framebuffer.clone());
        command_buffer.set_viewports(framebuffer.clone());
        command_buffer.set_scissor(framebuffer);

        let draw_queue = Rsc::<Vec<Draw>>::get().unwrap();
        let materials = Rsc::<IdVec<MaterialData>>::get().unwrap();
        let meshes = Rsc::<IdVec<MeshData<DefaultVertex>>>::get().unwrap();

        for draw in draw_queue.iter() {
            match draw {
                Draw::MeshAndMaterial(MeshAndMaterial {
                    mesh_id,
                    material_id,
                    camera_id,
                    camera_transform_id,
                    transform_id,
                }) => {
                    let material = materials.get(*material_id).unwrap();
                    let mesh = meshes.get(*mesh_id).unwrap();
                    material.bind(
                        command_buffer.clone(),
                        *camera_id,
                        *camera_transform_id,
                        *transform_id,
                    );
                    mesh.record_commands(command_buffer.clone());
                },
            }
        }
        command_buffer.end_renderpass();
        command_buffer.end()?;
    }

    {
        let swapchain = Rsc::<Arc<Swapchain>>::get().unwrap();
        let graphics_queue = Rsc::<Arc<Queue>>::get().unwrap();
        let frame_sync = Rsc::<Vec<VulkanRendererFrameSync>>::get().unwrap();
        let image_sync = Rsc::<Vec<VulkanRendererImageSync>>::get().unwrap();
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
        let mut current_frame = RscMut::<RendererCurrentFrame>::get().unwrap();
        let frames_in_flight = Rsc::<FramesInFlight>::get().unwrap();
        current_frame.0 = (current_frame.0 + 1) % frames_in_flight.0;
    }
    Ok(())
}

pub fn create_shader(
    code: &[u32],
    shader_type: ShaderType,
) -> Result<u32, VulkanError> {
    let shader = {
        let device = Rsc::<Arc<Device>>::get().unwrap();
        ShaderModule::new(device.clone(), code, shader_type)?
    };

    let mut shader_modules = RscMut::<IdVec<Arc<ShaderModule>>>::get().unwrap();
    Ok(shader_modules.push(shader))
}

pub fn create_buffer<T: Copy + Clone>(
    data: Vec<T>,
) -> Result<u32, VulkanError> {
    let device = Rsc::<Arc<Device>>::get().unwrap();
    let mut buffers = RscMut::<IdVec<Arc<Buffer>>>::get().unwrap();
    Ok(buffers.push(Buffer::from_vec(
        device.clone(),
        &data,
        BufferUsage::UNIFORM_BUFFER,
    )?))
}

pub fn update_buffer<T: Copy + Clone>(
    buffer_id: u32,
    data: Vec<T>,
) -> Result<(), VulkanError> {
    let mut buffers = RscMut::<IdVec<Arc<Buffer>>>::get().unwrap();
    let buffer = buffers.get_mut(buffer_id).unwrap();
    buffer.update(data)?;
    Ok(())
}

pub fn get_aspect_ratio() -> f32 {
    let surface = Rsc::<Arc<Surface>>::get().unwrap();
    surface.aspect_ratio()
}

pub fn queue_draw(draw: Draw) {
    let mut draw_queue = RscMut::<Vec<Draw>>::get().unwrap();
    draw_queue.push(draw);
}

pub fn queue_clear() {
    let mut draw_queue = RscMut::<Vec<Draw>>::get().unwrap();
    draw_queue.clear();
}

#[derive(Debug)]
pub enum RendererError {
    ShaderNotFound(u32),
    BufferNotFound(u32),
    MaterialBaseNotFound(u32),
}

impl From<RendererError> for VulkanError {
    fn from(value: RendererError) -> Self { VulkanError::RendererError(value) }
}
