//! The built-in renderer.

use std::{collections::HashMap, ops::DerefMut};

pub mod camera_data;
pub mod components;
pub mod depth_image;
pub mod draw_queue;
pub mod graphics_bundle;
pub mod light_data;
pub mod material_data;
pub mod mesh_data;
pub mod present_image;
pub mod shadow;
pub mod sync;
pub mod integrated_image;
pub mod image_atlas;
pub mod transform_data;

use sync::{RendererFrameSync, RendererImageSync};

use crate::{
    ctx::Ctx,
    error::{OptionNoneErr, ParsecError},
    graphics::{
        ActiveGraphicsBackend,
        command_list::{Command, CommandList},
        framebuffer::{Framebuffer, FramebufferBuilder},
        image::{ImageFormat, ImageSize},
        pipeline::DefaultVertex,
        renderpass::{
            Renderpass, RenderpassAttachment, RenderpassAttachmentLoadOp,
            RenderpassAttachmentStoreOp, RenderpassAttachmentType,
            RenderpassBuilder, RenderpassClearValue, RenderpassHandle,
        },
        window::Window,
    },
    renderer::{
        camera_data::{CameraData, CameraDataManager},
        depth_image::DepthImage,
        draw_queue::{Draw, MeshAndMaterial},
        light_data::RendererLights,
        material_data::{MaterialBase, MaterialData},
        mesh_data::MeshData,
        present_image::PresentImage,
        shadow::RendererShadows,
        transform_data::{TransformData, TransformDataManager},
    },
    utils::identifiable::IdStore,
};

fn create_frame_sync(
    backend: &mut ActiveGraphicsBackend,
    frames_in_flight: usize,
) -> Vec<RendererFrameSync> {
    let mut ret = Vec::new();
    for _ in 0..frames_in_flight {
        ret.push(RendererFrameSync::new(backend));
    }
    ret
}

fn create_image_sync(
    backend: &mut ActiveGraphicsBackend,
    image_count: usize,
) -> Vec<RendererImageSync> {
    let mut ret = Vec::new();
    for _ in 0..image_count {
        ret.push(RendererImageSync::new(backend));
    }
    ret
}

fn create_commad_lists(
    backend: &mut ActiveGraphicsBackend,
    frames_in_flight: usize,
) -> Vec<CommandList> {
    let mut ret = Vec::new();
    for _ in 0..frames_in_flight {
        ret.push(backend.create_command_list().unwrap());
    }
    ret
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

pub fn init_renderer(ctx: Ctx) -> Result<(), ParsecError> {
    let mut backend =
        ctx.resources.get_mut::<ActiveGraphicsBackend>().none_err()?;
    let window = ctx.resources.get::<Window>().none_err()?;

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
        .build(&mut backend)
        .unwrap();
    let swapchain_image_handles = backend.present_images();
    let swapchain_images = swapchain_image_handles
        .into_iter()
        .map(|img| PresentImage::new(&mut backend, img).unwrap())
        .collect::<Vec<_>>();
    let depth_image =
        DepthImage::new(&mut backend, ImageSize::new(window.size()).unwrap())
            .unwrap();
    let framebuffers = swapchain_images
        .iter()
        .map(|present_image| {
            FramebufferBuilder::new()
                .attachment(present_image.image_view_handle())
                .attachment(depth_image.image_view_handle())
                .size(window.size())
                .renderpass(renderpass.handle())
                .build(&mut backend)
                .unwrap()
        })
        .collect::<Vec<_>>();
    let frames_in_flight = 1;
    let frame_sync = create_frame_sync(&mut backend, frames_in_flight);
    let image_sync =
        create_image_sync(&mut backend, swapchain_images.len());
    let command_lists =
        create_commad_lists(&mut backend, frames_in_flight);

    let shadow_data = RendererShadows::new(&mut backend);
    let light_data = RendererLights::new(&mut backend);

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
    ctx.resources
        .add(RendererFramesInFlight(frames_in_flight as u32));
    ctx.resources.add(ResizeFlag(false));
    ctx.resources.add(Vec::<Draw>::new());
    ctx.resources.add(IdStore::<MeshData<DefaultVertex>>::new());
    ctx.resources.add(IdStore::<MaterialBase>::new());
    ctx.resources.add(IdStore::<MaterialData>::new());
    ctx.resources.add(IdStore::<TransformData>::new());
    ctx.resources.add(IdStore::<CameraData>::new());
    ctx.resources.add(TransformDataManager {
        component_to_data: HashMap::new(),
    });
    ctx.resources.add(CameraDataManager {
        component_to_data: HashMap::new(),
    });
    Ok(())
}

fn recreate_size_dependent_components(
    backend: &mut ActiveGraphicsBackend,
    window: &Window,
    present_images: &mut [PresentImage],
    depth_image: &mut DepthImage,
    framebuffers: &mut Vec<Framebuffer>,
    renderpass: RenderpassHandle,
) {
    backend.wait_idle();
    backend.handle_resize(window).unwrap();

    let new_swapchain_image_handles = backend.present_images();
    for (new_present_image_handle, present_image) in new_swapchain_image_handles
        .into_iter()
        .zip(present_images.iter_mut())
    {
        present_image
            .recreate(backend, new_present_image_handle)
            .unwrap();
    }
    depth_image
        .recreate(backend, ImageSize::new(window.size()).unwrap())
        .unwrap();
    let mut new_framebuffers = present_images
        .iter()
        .map(|present_image| {
            FramebufferBuilder::new()
                .attachment(present_image.image_view_handle())
                .attachment(depth_image.image_view_handle())
                .size(window.size())
                .renderpass(renderpass)
                .build(backend)
                .unwrap()
        })
        .collect::<Vec<_>>();

    for framebuffer in framebuffers.drain(0..framebuffers.len()) {
        framebuffer.destroy(backend).unwrap();
    }

    framebuffers.append(&mut new_framebuffers);
}

pub fn render(ctx: Ctx) -> Result<(), ParsecError> {
    let mut backend =
        ctx.resources.get_mut::<ActiveGraphicsBackend>().none_err()?;
    let mut current_frame =
        ctx.resources.get_mut::<RendererCurrentFrame>().none_err()?;
    let mut resize = ctx.resources.get_mut::<ResizeFlag>().none_err()?;
    let frames_in_flight =
        ctx.resources.get_mut::<RendererFramesInFlight>().none_err()?;
    let window = ctx.resources.get::<Window>().none_err()?;
    let frame_sync =
        ctx.resources.get::<Vec<RendererFrameSync>>().none_err()?;
    let image_sync =
        ctx.resources.get::<Vec<RendererImageSync>>().none_err()?;
    let mut present_images =
        ctx.resources.get_mut::<RendererPresentImages>().none_err()?;
    let mut depth_image =
        ctx.resources.get_mut::<RendererDepthImage>().none_err()?;
    let renderpass =
        ctx.resources.get_mut::<RendererMainRenderpass>().none_err()?;
    let mut framebuffers =
        ctx.resources.get_mut::<RendererFramebuffers>().none_err()?;
    let mut command_lists =
        ctx.resources.get_mut::<Vec<CommandList>>().none_err()?;
    let draw_queue = ctx.resources.get::<Vec<Draw>>().none_err()?;
    let meshes_data = ctx
        .resources
        .get::<IdStore<MeshData<DefaultVertex>>>()
        .none_err()?;
    let materials_data =
        ctx.resources.get::<IdStore<MaterialData>>().none_err()?;
    let material_bases =
        ctx.resources.get::<IdStore<MaterialBase>>().none_err()?;
    let transforms_data =
        ctx.resources.get::<IdStore<TransformData>>().none_err()?;
    let cameras_data = ctx.resources.get::<IdStore<CameraData>>().none_err()?;
    let shadows = ctx.resources.get::<RendererShadows>().none_err()?;
    let lights = ctx.resources.get::<RendererLights>().none_err()?;

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
        );
        resize.0 = false;
        return Ok(());
    }

    let present_index = match backend.start_frame(
        frame_sync[current_frame.0 as usize].image_available_semaphore,
    ) {
        Ok(val) => val,
        Err(err) => panic!("{:?}", err),
    };
    backend
        .reset_gpu_to_cpu_fence(
            frame_sync[current_frame.0 as usize].command_buffer_fence,
        )
        .unwrap();

    let command_list = &mut command_lists[current_frame.0 as usize];
    let framebuffer = &mut framebuffers.0[present_index as usize];

    command_list.reset();
    command_list.cmd(Command::Begin);
    command_list.cmd(Command::BeginRenderpass(
        shadows.renderpass.handle(),
        shadows.framebuffer.handle(),
    ));

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
                let mesh = meshes_data.get(*mesh).unwrap();
                let camera = cameras_data.get(*camera).unwrap();
                let camera_transform =
                    transforms_data.get(*camera_transform).unwrap();
                let transform = transforms_data.get(*transform).unwrap();
                material.bind(
                    command_list,
                    material_base,
                    camera,
                    camera_transform,
                    transform,
                    &lights,
                    &shadows,
                );
                mesh.record_commands_instanced(
                    command_list,
                    lights.data.light_count,
                );
            },
        }
    }

    command_list.cmd(Command::EndRenderpass);
    command_list.cmd(Command::BeginRenderpass(
        renderpass.0.handle(),
        framebuffer.handle(),
    ));

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
                    material_bases.get(material.material_base_id()).unwrap();
                let mesh = meshes_data.get(*mesh).unwrap();
                let camera = cameras_data.get(*camera).unwrap();
                let camera_transform =
                    transforms_data.get(*camera_transform).unwrap();
                let transform = transforms_data.get(*transform).unwrap();
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

    backend
        .submit_commands(
            command_list,
            &[frame_sync[current_frame.0 as usize].image_available_semaphore],
            &[image_sync[present_index as usize].rendering_complete_semaphore],
            frame_sync[current_frame.0 as usize].command_buffer_fence,
        )
        .unwrap();

    match backend.end_frame(
        &[image_sync[present_index as usize].rendering_complete_semaphore],
        present_index,
    ) {
        Ok(_) => (),
        _ => panic!("Shouldn't be here"),
    };

    let command_buffer_fence =
        frame_sync[current_frame.0 as usize].command_buffer_fence;
    backend.wait_gpu_to_cpu_fence(command_buffer_fence).unwrap();

    current_frame.0 = (current_frame.0 + 1) % frames_in_flight.0;
    Ok(())
}

pub fn queue_clear(ctx: Ctx) -> Result<(), ParsecError> {
    ctx.resources.get_mut::<Vec<Draw>>().none_err()?.clear();
    Ok(())
}

#[derive(Debug)]
pub enum RendererError {
    ShaderNotFound(u32),
    BufferNotFound(u32),
    MaterialBaseNotFound(u32),
}
