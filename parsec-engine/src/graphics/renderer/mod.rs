//! The built-in renderer.

use std::{collections::HashMap, ops::DerefMut};

pub mod assets;
pub mod camera_data;
pub mod components;
pub mod depth_image;
pub mod draw_queue;
pub mod light_data;
pub mod material_data;
pub mod mesh_data;
pub mod present_image;
pub mod sync;
pub mod texture;
pub mod texture_atlas;
pub mod transform_data;

use sync::{RendererFrameSync, RendererImageSync};

use crate::{
    ecs::system::{requests::Requests, system},
    graphics::{
        ActiveGraphicsBackend,
        buffer::{Buffer, BufferBuilder, BufferContent, BufferUsage},
        command_list::{Command, CommandList},
        framebuffer::{Framebuffer, FramebufferBuilder},
        image::{
            Image, ImageAspect, ImageBuilder, ImageFormat, ImageSize,
            ImageUsage, ImageView, ImageViewBuilder,
        },
        pipeline::{
            DefaultVertex, PipelineBindingType, PipelineOptions,
            PipelineResource, PipelineResourceBindingLayout,
            PipelineResourceHandle, PipelineResourceLayout,
            PipelineResourceLayoutBuilder, PipelineShaderStage,
        },
        renderer::{
            assets::mesh::Mesh,
            camera_data::{CameraData, CameraDataManager},
            depth_image::DepthImage,
            draw_queue::{Draw, MeshAndMaterial},
            material_data::{
                MaterialBase, MaterialData, MaterialPipelineBinding,
            },
            mesh_data::MeshData,
            present_image::PresentImage,
            transform_data::{TransformData, TransformDataManager},
        },
        renderpass::{
            Renderpass, RenderpassAttachment, RenderpassAttachmentLoadOp,
            RenderpassAttachmentStoreOp, RenderpassAttachmentType,
            RenderpassBuilder, RenderpassClearValue, RenderpassHandle,
        },
        sampler::{Sampler, SamplerBuilder},
        shader::{Shader, ShaderBuilder, ShaderType},
        vulkan::shader::read_shader_code,
        window::Window,
    },
    math::{mat::Matrix4f, uvec::Vec2u, vec::Vec3f},
    resources::Resource,
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

#[allow(unused)]
pub struct RendererShadowpassData {
    light_dir_buffer: Buffer,
    light_dir_layout: PipelineResourceLayout,
    light_dir_resource: PipelineResource,
    renderpass: Renderpass,
    material_id: u32,
    vertex_shader: Shader,
    fragment_shader: Shader,
    image: Image,
    image_view: ImageView,
    image_sampler: Sampler,
    image_resource: PipelineResource,
    framebuffer: Framebuffer,
    proj_buffer: Buffer,
    proj_resource: PipelineResource,
    look_buffer: Buffer,
    look_resource: PipelineResource,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::NoUninit)]
struct LD {
    dir: Vec3f,
    _pad: u32,
    mat: Matrix4f,
}

#[system]
pub fn init_renderer(
    mut requests: Resource<Requests>,
    mut backend: Resource<ActiveGraphicsBackend>,
    window: Resource<Window>,
) {
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

    let shadow_renderpass = RenderpassBuilder::new()
        .attachment(RenderpassAttachment {
            attachment_type: RenderpassAttachmentType::Depth,
            image_format: ImageFormat::D32,
            clear_value: RenderpassClearValue::Depth(1.0),
            load_op: RenderpassAttachmentLoadOp::Clear,
            store_op: RenderpassAttachmentStoreOp::Store,
        })
        .build(&mut backend)
        .unwrap();
    let shadow_vertex_shader = ShaderBuilder::new()
        .code(&read_shader_code("shaders/shadow_vert.spv").unwrap())
        .shader_type(ShaderType::Vertex)
        .build(&mut backend)
        .unwrap();
    let shadow_fragment_shader = ShaderBuilder::new()
        .code(&read_shader_code("shaders/shadow_frag.spv").unwrap())
        .shader_type(ShaderType::Fragment)
        .build(&mut backend)
        .unwrap();
    let shadow_material_base = MaterialBase::new(
        &mut *backend,
        shadow_vertex_shader.handle(),
        shadow_fragment_shader.handle(),
        shadow_renderpass.handle(),
        vec![
            vec![
                PipelineResourceBindingLayout::new(
                    PipelineBindingType::UniformBuffer,
                    &[PipelineShaderStage::Vertex],
                ),
                PipelineResourceBindingLayout::new(
                    PipelineBindingType::UniformBuffer,
                    &[PipelineShaderStage::Vertex],
                ),
                PipelineResourceBindingLayout::new(
                    PipelineBindingType::UniformBuffer,
                    &[PipelineShaderStage::Vertex],
                ),
            ],
            vec![PipelineResourceBindingLayout::new(
                PipelineBindingType::UniformBuffer,
                &[PipelineShaderStage::Vertex],
            )],
            vec![PipelineResourceBindingLayout::new(
                PipelineBindingType::UniformBuffer,
                &[PipelineShaderStage::Vertex],
            )],
        ],
        PipelineOptions::default(),
    );
    let shadow_size = 1 << 10;
    let shadow_depth_image = ImageBuilder::new()
        .size(ImageSize::new(Vec2u::new(shadow_size, shadow_size)).unwrap())
        .format(ImageFormat::D32)
        .aspect(ImageAspect::Depth)
        .usage(&[ImageUsage::DepthAttachment, ImageUsage::Sampled])
        .build(&mut backend)
        .unwrap();
    let shadow_depth_view = ImageViewBuilder::new()
        .image(shadow_depth_image.handle())
        .build(&mut backend)
        .unwrap();
    let shadow_framebuffer = FramebufferBuilder::new()
        .attachment(shadow_depth_view.handle())
        .size(Vec2u::new(shadow_size, shadow_size))
        .renderpass(shadow_renderpass.handle())
        .build(&mut backend)
        .unwrap();
    let shadow_proj_buffer = BufferBuilder::new()
        .usage(&[BufferUsage::Uniform])
        .data(BufferContent::from_slice(&[Matrix4f::orthographic(
            0.0, 100.0, 25.0, 25.0,
        )]))
        .build(&mut backend)
        .unwrap();
    let shadow_look_buffer = BufferBuilder::new()
        .usage(&[BufferUsage::Uniform])
        .data(BufferContent::from_slice(&[Matrix4f::look_at(
            Vec3f::new(-40.0, 40.0, -40.0),
            Vec3f::new(1.0, -1.0, 1.0),
            Vec3f::new(1.0, 1.0, 1.0),
        )]))
        .build(&mut backend)
        .unwrap();
    let shadow_proj_resource = shadow_material_base.resource_layouts()[2]
        .create_resource(&mut backend)
        .unwrap();
    let shadow_look_resource = shadow_material_base.resource_layouts()[1]
        .create_resource(&mut backend)
        .unwrap();
    shadow_proj_resource
        .bind_buffer(&mut backend, shadow_proj_buffer.handle(), 0)
        .unwrap();
    shadow_look_resource
        .bind_buffer(&mut backend, shadow_look_buffer.handle(), 0)
        .unwrap();
    let shadow_material = MaterialData::new(&shadow_material_base, vec![
        MaterialPipelineBinding::Model,
        MaterialPipelineBinding::Generic(shadow_look_resource.handle()),
        MaterialPipelineBinding::Generic(shadow_proj_resource.handle()),
    ]);
    let shadow_sampler = SamplerBuilder::new().build(&mut backend).unwrap();
    let shadow_tex_resource = PipelineResourceLayoutBuilder::new()
        .bindings(&[PipelineResourceBindingLayout {
            binding_type: PipelineBindingType::TextureSampler,
            shader_stages: vec![PipelineShaderStage::Fragment],
        }])
        .build(&mut backend)
        .unwrap()
        .create_resource(&mut backend)
        .unwrap();
    shadow_tex_resource
        .bind_sampler(
            &mut backend,
            shadow_sampler.handle(),
            shadow_depth_view.handle(),
            0,
        )
        .unwrap();

    let light_buffer = BufferBuilder::new()
        .usage(&[BufferUsage::Uniform])
        .data(BufferContent::from_slice(&[LD {
            dir: Vec3f::new(1.0, -1.0, 1.0),
            mat: Matrix4f::orthographic(0.0, 100.0, 25.0, 25.0)
                * Matrix4f::look_at(
                    Vec3f::new(-40.0, 40.0, -40.0),
                    Vec3f::new(1.0, -1.0, 1.0),
                    Vec3f::new(1.0, 1.0, 1.0),
                ),
            _pad: 0,
        }]))
        .build(&mut backend)
        .unwrap();
    let light_layout = PipelineResourceLayoutBuilder::new()
        .bindings(&[PipelineResourceBindingLayout::new(
            PipelineBindingType::UniformBuffer,
            &[PipelineShaderStage::Fragment, PipelineShaderStage::Vertex],
        )])
        .build(&mut backend)
        .unwrap();
    let light_resource = light_layout.create_resource(&mut backend).unwrap();
    light_resource
        .bind_buffer(&mut backend, light_buffer.handle(), 0)
        .unwrap();

    let mut material_bases = IdStore::new();
    let mut materials_data = IdStore::new();

    material_bases.push(shadow_material_base);
    let shadow_material_id = materials_data.push(shadow_material);

    let shadow_data = RendererShadowpassData {
        light_dir_buffer: light_buffer,
        light_dir_layout: light_layout,
        light_dir_resource: light_resource,
        renderpass: shadow_renderpass,
        material_id: shadow_material_id,
        vertex_shader: shadow_vertex_shader,
        fragment_shader: shadow_fragment_shader,
        image: shadow_depth_image,
        image_view: shadow_depth_view,
        image_sampler: shadow_sampler,
        image_resource: shadow_tex_resource,
        framebuffer: shadow_framebuffer,
        proj_buffer: shadow_proj_buffer,
        proj_resource: shadow_proj_resource,
        look_buffer: shadow_look_buffer,
        look_resource: shadow_look_resource,
    };

    requests.create_resource(shadow_data);
    requests.create_resource(RendererMainRenderpass(renderpass));
    requests.create_resource(RendererPresentImages(swapchain_images));
    requests.create_resource(RendererDepthImage(depth_image));
    requests.create_resource(RendererFramebuffers(framebuffers));
    requests.create_resource(frame_sync);
    requests.create_resource(image_sync);
    requests.create_resource(command_lists);
    requests.create_resource(RendererCurrentFrame(0));
    requests.create_resource(RendererFramesInFlight(frames_in_flight as u32));
    requests.create_resource(ResizeFlag(false));
    requests.create_resource(Vec::<Draw>::new());
    requests.create_resource(IdStore::<MeshData<DefaultVertex>>::new());
    requests.create_resource(IdStore::<Mesh>::new());
    requests.create_resource(material_bases);
    requests.create_resource(materials_data);
    requests.create_resource(IdStore::<TransformData>::new());
    requests.create_resource(IdStore::<CameraData>::new());
    requests.create_resource(TransformDataManager {
        component_to_data: HashMap::new(),
    });
    requests.create_resource(CameraDataManager {
        component_to_data: HashMap::new(),
    });
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

#[system]
pub fn render(
    mut backend: Resource<ActiveGraphicsBackend>,
    mut current_frame: Resource<RendererCurrentFrame>,
    mut resize: Resource<ResizeFlag>,
    frames_in_flight: Resource<RendererFramesInFlight>,
    window: Resource<Window>,
    frame_sync: Resource<Vec<RendererFrameSync>>,
    image_sync: Resource<Vec<RendererImageSync>>,
    mut present_images: Resource<RendererPresentImages>,
    mut depth_image: Resource<RendererDepthImage>,
    renderpass: Resource<RendererMainRenderpass>,
    mut framebuffers: Resource<RendererFramebuffers>,
    mut command_lists: Resource<Vec<CommandList>>,
    draw_queue: Resource<Vec<Draw>>,
    meshes_data: Resource<IdStore<MeshData<DefaultVertex>>>,
    materials_data: Resource<IdStore<MaterialData>>,
    material_bases: Resource<IdStore<MaterialBase>>,
    transforms_data: Resource<IdStore<TransformData>>,
    cameras_data: Resource<IdStore<CameraData>>,
    shadowpass_data: Resource<RendererShadowpassData>,
) {
    if window.minimized() {
        return;
    }

    if resize.0 {
        recreate_size_dependent_components(
            &mut *backend,
            &window,
            &mut present_images.0,
            &mut depth_image.0,
            &mut framebuffers.0,
            renderpass.0.handle(),
        );
        resize.0 = false;
        return;
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
        shadowpass_data.renderpass.handle(),
        shadowpass_data.framebuffer.handle(),
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
                let material =
                    materials_data.get(shadowpass_data.material_id).unwrap();
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
                    shadowpass_data.light_dir_resource.handle(),
                    shadowpass_data.image_resource.handle(),
                );
                mesh.record_commands(command_list);
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
                    shadowpass_data.light_dir_resource.handle(),
                    shadowpass_data.image_resource.handle(),
                );
                mesh.record_commands(command_list);
            },
        }
    }

    command_list.cmd(Command::EndRenderpass);
    command_list.cmd(Command::End);

    backend
        .submit_commands(
            &command_list,
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
}

#[system]
pub fn queue_clear(mut draw_queue: Resource<Vec<Draw>>) { draw_queue.clear(); }

#[derive(Debug)]
pub enum RendererError {
    ShaderNotFound(u32),
    BufferNotFound(u32),
    MaterialBaseNotFound(u32),
}
