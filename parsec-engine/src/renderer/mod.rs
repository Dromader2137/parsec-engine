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
pub mod texture;
pub mod texture_atlas;
pub mod transform_data;

use sync::{RendererFrameSync, RendererImageSync};

use crate::{
    ecs::world::World,
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

pub fn init_renderer(world: &mut World) -> Result<(), ParsecError> {
    let mut backend = world.resources.get::<ActiveGraphicsBackend>().none_err()?;
    let window = world.resources.get::<Window>().none_err()?;

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
    let frame_sync = create_frame_sync(backend.deref_mut(), frames_in_flight);
    let image_sync =
        create_image_sync(backend.deref_mut(), swapchain_images.len());
    let command_lists =
        create_commad_lists(backend.deref_mut(), frames_in_flight);

    let shadow_data = RendererShadows::new(&mut backend);
    let light_data = RendererLights::new(&mut backend);

    // Drop resource guards before mutating world.resources
    drop(backend);
    drop(window);

    world.resources.add(shadow_data);
    world.resources.add(light_data);
    world.resources.add(RendererMainRenderpass(renderpass));
    world.resources.add(RendererPresentImages(swapchain_images));
    world.resources.add(RendererDepthImage(depth_image));
    world.resources.add(RendererFramebuffers(framebuffers));
    world.resources.add(frame_sync);
    world.resources.add(image_sync);
    world.resources.add(command_lists);
    world.resources.add(RendererCurrentFrame(0));
    world
        .resources
        .add(RendererFramesInFlight(frames_in_flight as u32));
    world.resources.add(ResizeFlag(false));
    world.resources.add(Vec::<Draw>::new());
    world
        .resources
        .add(IdStore::<MeshData<DefaultVertex>>::new());
    world.resources.add(IdStore::<MaterialBase>::new());
    world.resources.add(IdStore::<MaterialData>::new());
    world.resources.add(IdStore::<TransformData>::new());
    world.resources.add(IdStore::<CameraData>::new());
    world.resources.add(TransformDataManager {
        component_to_data: HashMap::new(),
    });
    world.resources.add(CameraDataManager {
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

pub fn render(world: &World) -> Result<(), ParsecError> {
    let mut backend = world.resources.get::<ActiveGraphicsBackend>().none_err()?;
    let mut current_frame = world.resources.get::<RendererCurrentFrame>().none_err()?;
    let mut resize = world.resources.get::<ResizeFlag>().none_err()?;
    let frames_in_flight = world.resources.get::<RendererFramesInFlight>().none_err()?;
    let window = world.resources.get::<Window>().none_err()?;
    let frame_sync = world.resources.get::<Vec<RendererFrameSync>>().none_err()?;
    let image_sync = world.resources.get::<Vec<RendererImageSync>>().none_err()?;
    let mut present_images = world.resources.get::<RendererPresentImages>().none_err()?;
    let mut depth_image = world.resources.get::<RendererDepthImage>().none_err()?;
    let renderpass = world.resources.get::<RendererMainRenderpass>().none_err()?;
    let mut framebuffers = world.resources.get::<RendererFramebuffers>().none_err()?;
    let mut command_lists = world.resources.get::<Vec<CommandList>>().none_err()?;
    let draw_queue = world.resources.get::<Vec<Draw>>().none_err()?;
    let meshes_data = world.resources.get::<IdStore<MeshData<DefaultVertex>>>().none_err()?;
    let materials_data = world.resources.get::<IdStore<MaterialData>>().none_err()?;
    let material_bases = world.resources.get::<IdStore<MaterialBase>>().none_err()?;
    let transforms_data = world.resources.get::<IdStore<TransformData>>().none_err()?;
    let cameras_data = world.resources.get::<IdStore<CameraData>>().none_err()?;
    let shadows = world.resources.get::<RendererShadows>().none_err()?;
    let lights = world.resources.get::<RendererLights>().none_err()?;

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

pub fn queue_clear(world: &World) -> Result<(), ParsecError> {
    world.resources.get::<Vec<Draw>>().none_err()?.clear();
    Ok(())
}

#[derive(Debug)]
pub enum RendererError {
    ShaderNotFound(u32),
    BufferNotFound(u32),
    MaterialBaseNotFound(u32),
}
