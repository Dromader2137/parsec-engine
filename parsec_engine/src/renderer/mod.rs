//! The built-in renderer.

use std::{any::type_name, collections::HashMap};

pub mod camera_data;
pub mod components;
pub mod depth_image;
pub mod draw_queue;
pub mod graphics_bundle;
pub mod integrated_image;
pub mod light_data;
pub mod material_data;
pub mod mesh_data;
pub mod present_image;
pub mod shadow;
pub mod sync;
pub mod transform_data;

use sync::{RendererFrameSync, RendererImageSync};

use crate::{
    ctx::Ctx,
    ecs::{
        resources::{
            ResourceError, ResourceMarker, Resources,
            resource::{Resource, ResourceMut},
        },
        world::WorldError,
    },
    error::SystemError,
    graphics::{
        ActiveGraphicsBackend, EventLoopAccessError,
        backend::BackendError,
        buffer::BufferError,
        command_list::{Command, CommandList},
        framebuffer::{Framebuffer, FramebufferBuilder, FramebufferError},
        image::{ImageError, ImageFormat, ImageSize},
        pipeline::{DefaultVertex, PipelineError},
        renderpass::{
            Renderpass, RenderpassAttachment, RenderpassAttachmentLoadOp,
            RenderpassAttachmentStoreOp, RenderpassAttachmentType, RenderpassBuilder,
            RenderpassClearValue, RenderpassError, RenderpassHandle,
        },
        shader_module::ShaderError,
        window::{Window, WindowError},
    },
    renderer::{
        camera_data::{CameraData, CameraDataManager},
        depth_image::DepthImage,
        draw_queue::{Draw, MeshAndMaterial},
        integrated_image::IntegratedImageError,
        light_data::RendererLights,
        material_data::{MaterialBase, MaterialData},
        mesh_data::MeshData,
        present_image::PresentImage,
        shadow::RendererShadows,
        transform_data::{TransformData, TransformDataManager},
    },
    utils::identifiable::IdStore,
};

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum RendererError {
    #[error("resource '{0}' not found or already borrowed")]
    ResourceNotFound(&'static str),
    #[error("mesh data {0} not found")]
    MeshDataNotFound(u32),
    #[error("material data {0} not found")]
    MaterialDataNotFound(u32),
    #[error("material base {0} not found")]
    MaterialBaseNotFound(u32),
    #[error("camera data {0} not found")]
    CameraDataNotFound(u32),
    #[error("transform data {0} not found")]
    TransformDataNotFound(u32),
    #[error("mesh asset not loaded")]
    MeshAssetNotLoaded,
    #[error(transparent)]
    Backend(#[from] BackendError),
    #[error(transparent)]
    Buffer(#[from] BufferError),
    #[error(transparent)]
    Pipeline(#[from] PipelineError),
    #[error(transparent)]
    Renderpass(#[from] RenderpassError),
    #[error(transparent)]
    Framebuffer(#[from] FramebufferError),
    #[error(transparent)]
    Image(#[from] ImageError),
    #[error(transparent)]
    IntegratedImage(#[from] IntegratedImageError),
    #[error(transparent)]
    Shader(#[from] ShaderError),
    #[error(transparent)]
    Window(#[from] WindowError),
    #[error(transparent)]
    EventLoopAccess(#[from] EventLoopAccessError),
    #[error(transparent)]
    World(#[from] WorldError),
    #[error(transparent)]
    Resource(#[from] ResourceError),
}

fn get_resource<R: ResourceMarker>(
    resources: &Resources,
) -> Result<Resource<'_, R>, RendererError> {
    resources.get::<R>().ok_or(RendererError::ResourceNotFound(type_name::<R>()))
}

fn get_resource_mut<R: ResourceMarker>(
    resources: &Resources,
) -> Result<ResourceMut<'_, R>, RendererError> {
    resources.get_mut::<R>().ok_or(RendererError::ResourceNotFound(type_name::<R>()))
}

fn create_frame_sync(
    backend: &mut ActiveGraphicsBackend,
    frames_in_flight: usize,
) -> Result<Vec<RendererFrameSync>, BackendError> {
    (0..frames_in_flight).map(|_| RendererFrameSync::new(backend)).collect()
}

fn create_image_sync(
    backend: &mut ActiveGraphicsBackend,
    image_count: usize,
) -> Result<Vec<RendererImageSync>, BackendError> {
    (0..image_count).map(|_| RendererImageSync::new(backend)).collect()
}

fn create_commad_lists(
    backend: &mut ActiveGraphicsBackend,
    frames_in_flight: usize,
) -> Result<Vec<CommandList>, BackendError> {
    (0..frames_in_flight).map(|_| backend.create_command_list()).collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResizeFlag(pub bool);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RendererCurrentFrame(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RendererFramesInFlight(pub u32);
#[derive(Debug, Clone, PartialEq)]
pub struct RendererMainRenderpass(pub Renderpass);
#[derive(Debug)]
pub struct RendererPresentImages(pub Vec<PresentImage>);
#[derive(Debug)]
pub struct RendererDepthImage(pub DepthImage);
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RendererFramebuffers(pub Vec<Framebuffer>);

pub fn init_renderer(ctx: Ctx) -> Result<(), SystemError> { Ok(try_init_renderer(ctx)?) }

fn try_init_renderer(ctx: Ctx) -> Result<(), RendererError> {
    let mut backend = get_resource_mut::<ActiveGraphicsBackend>(ctx.resources)?;
    let window = get_resource::<Window>(ctx.resources)?;

    let surface_format = backend.get_surface_format();

    let renderpass = RenderpassBuilder::new()
        .attachment(RenderpassAttachment {
            attachment_type: RenderpassAttachmentType::Color,
            image_format: surface_format,
            clear_value: RenderpassClearValue::Color(0.0, 0.0, 0.0, 0.0),
            load_op: RenderpassAttachmentLoadOp::Clear,
            store_op: RenderpassAttachmentStoreOp::Store,
        })
        .attachment(RenderpassAttachment {
            attachment_type: RenderpassAttachmentType::Depth,
            image_format: ImageFormat::D32,
            clear_value: RenderpassClearValue::Depth(1.0),
            load_op: RenderpassAttachmentLoadOp::Clear,
            store_op: RenderpassAttachmentStoreOp::DontCare,
        })
        .build(&mut backend)?;
    let swapchain_image_handles = backend.present_images();
    let swapchain_images = swapchain_image_handles
        .into_iter()
        .map(|img| PresentImage::new(&mut backend, img))
        .collect::<Result<Vec<_>, _>>()?;
    let depth_image = DepthImage::new(&mut backend, ImageSize::new(window.size())?)?;
    let framebuffers = swapchain_images
        .iter()
        .map(|present_image| {
            FramebufferBuilder::new()
                .attachment(present_image.image_view_handle())
                .attachment(depth_image.image_view_handle())
                .size(window.size())
                .renderpass(renderpass.handle())
                .build(&mut backend)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let frames_in_flight = 1;
    let frame_sync = create_frame_sync(&mut backend, frames_in_flight)?;
    let image_sync = create_image_sync(&mut backend, swapchain_images.len())?;
    let command_lists = create_commad_lists(&mut backend, frames_in_flight)?;

    let shadow_data = RendererShadows::new(&mut backend)?;
    let light_data = RendererLights::new(&mut backend)?;

    // Drop resource guards before mutating resources
    drop(backend);
    drop(window);

    ctx.resources.add(shadow_data);
    ctx.resources.add(light_data);
    ctx.resources.add(RendererMainRenderpass(renderpass));
    ctx.resources.add(RendererPresentImages(swapchain_images));
    ctx.resources.add(RendererDepthImage(depth_image));
    ctx.resources.add(RendererFramebuffers(framebuffers));
    ctx.resources.add(frame_sync);
    ctx.resources.add(image_sync);
    ctx.resources.add(command_lists);
    ctx.resources.add(RendererCurrentFrame(0));
    ctx.resources.add(RendererFramesInFlight(frames_in_flight as u32));
    ctx.resources.add(ResizeFlag(false));
    ctx.resources.add(Vec::<Draw>::new());
    ctx.resources.add(IdStore::<MeshData<DefaultVertex>>::new());
    ctx.resources.add(IdStore::<MaterialBase>::new());
    ctx.resources.add(IdStore::<MaterialData>::new());
    ctx.resources.add(IdStore::<TransformData>::new());
    ctx.resources.add(IdStore::<CameraData>::new());
    ctx.resources.add(TransformDataManager { component_to_data: HashMap::new() });
    ctx.resources.add(CameraDataManager { component_to_data: HashMap::new() });
    Ok(())
}

fn recreate_size_dependent_components(
    backend: &mut ActiveGraphicsBackend,
    window: &Window,
    present_images: &mut [PresentImage],
    depth_image: &mut DepthImage,
    framebuffers: &mut Vec<Framebuffer>,
    renderpass: RenderpassHandle,
) -> Result<(), RendererError> {
    backend.wait_idle();
    backend.handle_resize(window)?;

    let new_swapchain_image_handles = backend.present_images();
    for (new_present_image_handle, present_image) in
        new_swapchain_image_handles.into_iter().zip(present_images.iter_mut())
    {
        present_image.recreate(backend, new_present_image_handle)?;
    }
    depth_image.recreate(backend, ImageSize::new(window.size())?)?;
    let mut new_framebuffers = present_images
        .iter()
        .map(|present_image| {
            FramebufferBuilder::new()
                .attachment(present_image.image_view_handle())
                .attachment(depth_image.image_view_handle())
                .size(window.size())
                .renderpass(renderpass)
                .build(backend)
        })
        .collect::<Result<Vec<_>, _>>()?;

    for framebuffer in framebuffers.drain(0..framebuffers.len()) {
        framebuffer.destroy(backend)?;
    }

    framebuffers.append(&mut new_framebuffers);
    Ok(())
}

pub fn render(ctx: Ctx) -> Result<(), SystemError> { Ok(try_render(ctx)?) }

fn try_render(ctx: Ctx) -> Result<(), RendererError> {
    let mut backend = get_resource_mut::<ActiveGraphicsBackend>(ctx.resources)?;
    let mut current_frame = get_resource_mut::<RendererCurrentFrame>(ctx.resources)?;
    let mut resize = get_resource_mut::<ResizeFlag>(ctx.resources)?;
    let frames_in_flight = get_resource_mut::<RendererFramesInFlight>(ctx.resources)?;
    let window = get_resource::<Window>(ctx.resources)?;
    let frame_sync = get_resource::<Vec<RendererFrameSync>>(ctx.resources)?;
    let image_sync = get_resource::<Vec<RendererImageSync>>(ctx.resources)?;
    let mut present_images = get_resource_mut::<RendererPresentImages>(ctx.resources)?;
    let mut depth_image = get_resource_mut::<RendererDepthImage>(ctx.resources)?;
    let renderpass = get_resource_mut::<RendererMainRenderpass>(ctx.resources)?;
    let mut framebuffers = get_resource_mut::<RendererFramebuffers>(ctx.resources)?;
    let mut command_lists = get_resource_mut::<Vec<CommandList>>(ctx.resources)?;
    let draw_queue = get_resource::<Vec<Draw>>(ctx.resources)?;
    let meshes_data = get_resource::<IdStore<MeshData<DefaultVertex>>>(ctx.resources)?;
    let materials_data = get_resource::<IdStore<MaterialData>>(ctx.resources)?;
    let material_bases = get_resource::<IdStore<MaterialBase>>(ctx.resources)?;
    let transforms_data = get_resource::<IdStore<TransformData>>(ctx.resources)?;
    let cameras_data = get_resource::<IdStore<CameraData>>(ctx.resources)?;
    let shadows = get_resource::<RendererShadows>(ctx.resources)?;
    let lights = get_resource::<RendererLights>(ctx.resources)?;

    if window.minimized() {
        return Ok(());
    }

    if resize.0 {
        recreate_size_dependent_components(
            &mut backend,
            &window,
            &mut present_images.0,
            &mut depth_image.0,
            &mut framebuffers.0,
            renderpass.0.handle(),
        )?;
        resize.0 = false;
        return Ok(());
    }

    let present_index =
        backend.start_frame(frame_sync[current_frame.0 as usize].image_available_semaphore)?;
    backend.reset_gpu_to_cpu_fence(frame_sync[current_frame.0 as usize].command_buffer_fence)?;

    let command_list = &mut command_lists[current_frame.0 as usize];
    let framebuffer = &mut framebuffers.0[present_index as usize];

    command_list.reset();
    command_list.cmd(Command::Begin);
    command_list
        .cmd(Command::BeginRenderpass(shadows.renderpass.handle(), shadows.framebuffer.handle()));

    for draw in draw_queue.iter() {
        match draw {
            Draw::MeshAndMaterial(MeshAndMaterial {
                mesh,
                camera,
                camera_transform,
                transform,
                ..
            }) => {
                let material = &shadows.material;
                let material_base = &shadows.material_base;
                let mesh = meshes_data.get(*mesh).ok_or(RendererError::MeshDataNotFound(*mesh))?;
                let camera =
                    cameras_data.get(*camera).ok_or(RendererError::CameraDataNotFound(*camera))?;
                let camera_transform = transforms_data
                    .get(*camera_transform)
                    .ok_or(RendererError::TransformDataNotFound(*camera_transform))?;
                let transform = transforms_data
                    .get(*transform)
                    .ok_or(RendererError::TransformDataNotFound(*transform))?;
                material.bind(
                    command_list,
                    material_base,
                    camera,
                    camera_transform,
                    transform,
                    &lights,
                    &shadows,
                );
                mesh.record_commands_instanced(command_list, lights.data.light_count);
            },
        }
    }

    command_list.cmd(Command::EndRenderpass);
    command_list.cmd(Command::BeginRenderpass(renderpass.0.handle(), framebuffer.handle()));

    for draw in draw_queue.iter() {
        match draw {
            Draw::MeshAndMaterial(MeshAndMaterial {
                mesh,
                material,
                camera,
                camera_transform,
                transform,
            }) => {
                let material = materials_data
                    .get(*material)
                    .ok_or(RendererError::MaterialDataNotFound(*material))?;
                let material_base = material_bases
                    .get(material.material_base_id())
                    .ok_or(RendererError::MaterialBaseNotFound(material.material_base_id()))?;
                let mesh = meshes_data.get(*mesh).ok_or(RendererError::MeshDataNotFound(*mesh))?;
                let camera =
                    cameras_data.get(*camera).ok_or(RendererError::CameraDataNotFound(*camera))?;
                let camera_transform = transforms_data
                    .get(*camera_transform)
                    .ok_or(RendererError::TransformDataNotFound(*camera_transform))?;
                let transform = transforms_data
                    .get(*transform)
                    .ok_or(RendererError::TransformDataNotFound(*transform))?;
                material.bind(
                    command_list,
                    material_base,
                    camera,
                    camera_transform,
                    transform,
                    &lights,
                    &shadows,
                );
                mesh.record_commands(command_list);
            },
        }
    }

    command_list.cmd(Command::EndRenderpass);
    command_list.cmd(Command::End);

    backend.submit_commands(
        command_list,
        &[frame_sync[current_frame.0 as usize].image_available_semaphore],
        &[image_sync[present_index as usize].rendering_complete_semaphore],
        frame_sync[current_frame.0 as usize].command_buffer_fence,
    )?;

    backend.end_frame(
        &[image_sync[present_index as usize].rendering_complete_semaphore],
        present_index,
    )?;

    let command_buffer_fence = frame_sync[current_frame.0 as usize].command_buffer_fence;
    backend.wait_gpu_to_cpu_fence(command_buffer_fence)?;

    current_frame.0 = (current_frame.0 + 1) % frames_in_flight.0;
    Ok(())
}

pub fn queue_clear(ctx: Ctx) -> Result<(), SystemError> {
    get_resource_mut::<Vec<Draw>>(ctx.resources)?.clear();
    Ok(())
}
