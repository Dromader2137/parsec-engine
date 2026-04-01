//! The built-in renderer.

use std::{collections::HashMap, ops::DerefMut};

pub mod assets;
pub mod camera_data;
pub mod components;
pub mod draw_queue;
pub mod light_data;
pub mod material_data;
pub mod mesh_data;
pub mod sync;
pub mod texture;
pub mod texture_atlas;
pub mod transform_data;

use sync::{RendererFrameSync, RendererImageSync};

use crate::{
    ecs::system::{requests::Requests, system},
    graphics::{
        ActiveGraphicsBackend,
        buffer::{Buffer, BufferContent, BufferUsage},
        command_list::{Command, CommandList},
        framebuffer::Framebuffer,
        image::{Image, ImageAspect, ImageFormat, ImageUsage, ImageView},
        pipeline::{
            DefaultVertex, PipelineBindingType, PipelineOptions,
            PipelineResource, PipelineResourceBindingLayout,
            PipelineShaderStage,
        },
        renderer::{
            assets::mesh::Mesh,
            camera_data::{CameraData, CameraDataManager},
            draw_queue::{Draw, MeshAndMaterial},
            material_data::{
                MaterialBase, MaterialData, MaterialPipelineBinding,
            },
            mesh_data::MeshData,
            transform_data::{TransformData, TransformDataManager},
        },
        renderpass::{
            Renderpass, RenderpassAttachment, RenderpassAttachmentLoadOp,
            RenderpassAttachmentStoreOp, RenderpassAttachmentType,
            RenderpassClearValue,
        },
        sampler::Sampler,
        shader::{Shader, ShaderType},
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RendererMainRenderpass(pub Renderpass);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RendererSwapchainImages(pub Vec<Image>);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RendererSwapchainImageViews(pub Vec<ImageView>);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RendererDepthImage(pub Image);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RendererDepthImageView(pub ImageView);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RendererFramebuffers(pub Vec<Framebuffer>);

#[allow(unused)]
pub struct RendererShadowpassData {
    light_dir_buffer: Buffer,
    light_dir_binding: PipelineResource,
    renderpass: Renderpass,
    material_id: u32,
    vertex_shader: Shader,
    fragment_shader: Shader,
    image: Image,
    image_view: ImageView,
    image_sampler: Sampler,
    image_binding: PipelineResource,
    framebuffer: Framebuffer,
    proj_buffer: Buffer,
    look_buffer: Buffer,
    proj_binding: PipelineResource,
    look_binding: PipelineResource,
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

    let renderpass = backend
        .create_renderpass(&[
            RenderpassAttachment {
                attachment_type: RenderpassAttachmentType::Color,
                image_format: surface_format,
                clear_value: RenderpassClearValue::Color(0.0, 0.0, 0.0, 0.0),
                load_op: RenderpassAttachmentLoadOp::Clear,
                store_op: RenderpassAttachmentStoreOp::Store,
            },
            RenderpassAttachment {
                attachment_type: RenderpassAttachmentType::Depth,
                image_format: ImageFormat::D32,
                clear_value: RenderpassClearValue::Depth(1.0),
                load_op: RenderpassAttachmentLoadOp::Clear,
                store_op: RenderpassAttachmentStoreOp::DontCare,
            },
        ])
        .unwrap();
    let swapchain_images = backend.present_images();
    let swapchain_image_views = swapchain_images
        .iter()
        .map(|img| backend.create_image_view(*img).unwrap())
        .collect::<Vec<_>>();
    let depth_image = backend
        .create_image(window.size(), ImageFormat::D32, ImageAspect::Depth, &[
            ImageUsage::DepthAttachment,
        ])
        .unwrap();
    let depth_image_view = backend.create_image_view(depth_image).unwrap();
    let framebuffers = swapchain_image_views
        .iter()
        .map(|view| {
            backend
                .create_framebuffer(
                    window.size(),
                    &[*view, depth_image_view],
                    renderpass,
                )
                .unwrap()
        })
        .collect::<Vec<_>>();
    let frames_in_flight = 1;
    let frame_sync = create_frame_sync(backend.deref_mut(), frames_in_flight);
    let image_sync =
        create_image_sync(backend.deref_mut(), swapchain_images.len());
    let command_lists =
        create_commad_lists(backend.deref_mut(), frames_in_flight);

    let shadow_renderpass = backend
        .create_renderpass(&[RenderpassAttachment {
            attachment_type: RenderpassAttachmentType::Depth,
            image_format: ImageFormat::D32,
            clear_value: RenderpassClearValue::Depth(1.0),
            load_op: RenderpassAttachmentLoadOp::Clear,
            store_op: RenderpassAttachmentStoreOp::Store,
        }])
        .unwrap();
    let shadow_vertex_shader = backend
        .create_shader(
            &read_shader_code("shaders/shadow_vert.spv").unwrap(),
            ShaderType::Vertex,
        )
        .unwrap();
    let shadow_fragment_shader = backend
        .create_shader(
            &read_shader_code("shaders/shadow_frag.spv").unwrap(),
            ShaderType::Fragment,
        )
        .unwrap();
    let shadow_material_base = MaterialBase::new(
        &mut *backend,
        shadow_vertex_shader,
        shadow_fragment_shader,
        shadow_renderpass,
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
    let shadow_depth_image = backend
        .create_image(
            Vec2u::new(shadow_size, shadow_size),
            ImageFormat::D32,
            ImageAspect::Depth,
            &[ImageUsage::DepthAttachment, ImageUsage::Sampled],
        )
        .unwrap();
    let shadow_depth_view =
        backend.create_image_view(shadow_depth_image).unwrap();
    let shadow_framebuffer = backend
        .create_framebuffer(
            Vec2u::new(shadow_size, shadow_size),
            &[shadow_depth_view],
            shadow_renderpass,
        )
        .unwrap();
    let shadow_proj_buffer = backend
        .create_buffer(
            BufferContent::from_slice(&[Matrix4f::orthographic(
                0.0, 100.0, 25.0, 25.0,
            )]),
            &[BufferUsage::Uniform],
        )
        .unwrap();
    let shadow_look_buffer = backend
        .create_buffer(
            BufferContent::from_slice(&[Matrix4f::look_at(
                Vec3f::new(-40.0, 40.0, -40.0),
                Vec3f::new(1.0, -1.0, 1.0),
                Vec3f::new(1.0, 1.0, 1.0),
            )]),
            &[BufferUsage::Uniform],
        )
        .unwrap();
    let shadow_proj_layout = backend
        .create_pipeline_resource_layout(&[PipelineResourceBindingLayout {
            binding_type: PipelineBindingType::UniformBuffer,
            shader_stages: vec![PipelineShaderStage::Vertex],
        }])
        .unwrap();
    let shadow_look_layout = backend
        .create_pipeline_resource_layout(&[PipelineResourceBindingLayout {
            binding_type: PipelineBindingType::UniformBuffer,
            shader_stages: vec![PipelineShaderStage::Vertex],
        }])
        .unwrap();
    let shadow_proj_binding = backend
        .create_pipeline_resource(shadow_proj_layout)
        .unwrap();
    let shadow_look_binding = backend
        .create_pipeline_resource(shadow_look_layout)
        .unwrap();
    backend
        .bind_buffer(shadow_proj_binding, shadow_proj_buffer, 0)
        .unwrap();
    backend
        .bind_buffer(shadow_look_binding, shadow_look_buffer, 0)
        .unwrap();
    let shadow_material = MaterialData::new(&shadow_material_base, vec![
        MaterialPipelineBinding::Model,
        MaterialPipelineBinding::Generic(shadow_look_binding),
        MaterialPipelineBinding::Generic(shadow_proj_binding),
    ]);
    let shadow_sampler = backend.create_image_sampler().unwrap();
    let shadow_tex_layout = backend
        .create_pipeline_resource_layout(&[PipelineResourceBindingLayout {
            binding_type: PipelineBindingType::TextureSampler,
            shader_stages: vec![PipelineShaderStage::Fragment],
        }])
        .unwrap();
    let shadow_tex_binding =
        backend.create_pipeline_resource(shadow_tex_layout).unwrap();
    backend
        .bind_sampler(shadow_tex_binding, shadow_sampler, shadow_depth_view, 0)
        .unwrap();

    let light_buffer = backend
        .create_buffer(
            BufferContent::from_slice(&[LD {
                dir: Vec3f::new(1.0, -1.0, 1.0),
                mat: Matrix4f::orthographic(0.0, 100.0, 25.0, 25.0)
                    * Matrix4f::look_at(
                        Vec3f::new(-40.0, 40.0, -40.0),
                        Vec3f::new(1.0, -1.0, 1.0),
                        Vec3f::new(1.0, 1.0, 1.0),
                    ),
                _pad: 0,
            }]),
            &[BufferUsage::Uniform],
        )
        .unwrap();
    let light_binding_layout = backend
        .create_pipeline_resource_layout(&[PipelineResourceBindingLayout::new(
            PipelineBindingType::UniformBuffer,
            &[PipelineShaderStage::Fragment, PipelineShaderStage::Vertex],
        )])
        .unwrap();
    let light_binding = backend
        .create_pipeline_resource(light_binding_layout)
        .unwrap();
    backend.bind_buffer(light_binding, light_buffer, 0).unwrap();

    let mut material_bases = IdStore::new();
    let mut materials_data = IdStore::new();

    material_bases.push(shadow_material_base);
    let shadow_material_id = materials_data.push(shadow_material);

    let shadow_data = RendererShadowpassData {
        light_dir_buffer: light_buffer,
        light_dir_binding: light_binding,
        renderpass: shadow_renderpass,
        material_id: shadow_material_id,
        vertex_shader: shadow_vertex_shader,
        fragment_shader: shadow_fragment_shader,
        image: shadow_depth_image,
        image_view: shadow_depth_view,
        image_sampler: shadow_sampler,
        image_binding: shadow_tex_binding,
        framebuffer: shadow_framebuffer,
        proj_buffer: shadow_proj_buffer,
        proj_binding: shadow_proj_binding,
        look_buffer: shadow_look_buffer,
        look_binding: shadow_look_binding,
    };

    requests.create_resource(shadow_data);
    requests.create_resource(RendererMainRenderpass(renderpass));
    requests.create_resource(RendererSwapchainImages(swapchain_images));
    requests.create_resource(RendererSwapchainImageViews(swapchain_image_views));
    requests.create_resource(RendererDepthImage(depth_image));
    requests.create_resource(RendererDepthImageView(depth_image_view));
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
    requests: &mut Requests,
    backend: &mut ActiveGraphicsBackend,
    window: &Window,
    swapchain_views: &[ImageView],
    depth_image: Image,
    depth_view: ImageView,
    framebuffers: &[Framebuffer],
    renderpass: Renderpass,
) {
    backend.wait_idle();
    backend.handle_resize(window).unwrap();

    let new_swapchain_images = backend.present_images();
    let new_swapchain_image_views = new_swapchain_images
        .iter()
        .map(|img| backend.create_image_view(*img).unwrap())
        .collect::<Vec<_>>();
    let new_depth_image = backend
        .create_image(window.size(), ImageFormat::D32, ImageAspect::Depth, &[
            ImageUsage::DepthAttachment,
        ])
        .unwrap();
    let new_depth_image_view =
        backend.create_image_view(new_depth_image).unwrap();
    let new_framebuffers = new_swapchain_image_views
        .iter()
        .map(|view| {
            backend
                .create_framebuffer(
                    window.size(),
                    &[*view, new_depth_image_view],
                    renderpass,
                )
                .unwrap()
        })
        .collect::<Vec<_>>();

    swapchain_views
        .iter()
        .for_each(|view| backend.delete_image_view(*view).unwrap());
    backend.delete_image_view(depth_view).unwrap();
    backend.delete_image(depth_image).unwrap();
    framebuffers.iter().for_each(|framebuffer| {
        backend.delete_framebuffer(*framebuffer).unwrap()
    });

    requests.create_resource(RendererSwapchainImages(new_swapchain_images));
    requests.create_resource(RendererSwapchainImageViews(
        new_swapchain_image_views,
    ));
    requests.create_resource(RendererDepthImage(new_depth_image));
    requests.create_resource(RendererDepthImageView(new_depth_image_view));
    requests.create_resource(RendererFramebuffers(new_framebuffers));
}

#[system]
pub fn render(
    mut requests: Resource<Requests>,
    mut backend: Resource<ActiveGraphicsBackend>,
    mut current_frame: Resource<RendererCurrentFrame>,
    mut resize: Resource<ResizeFlag>,
    frames_in_flight: Resource<RendererFramesInFlight>,
    window: Resource<Window>,
    frame_sync: Resource<Vec<RendererFrameSync>>,
    image_sync: Resource<Vec<RendererImageSync>>,
    swapchain_views: Resource<RendererSwapchainImageViews>,
    depth_image: Resource<RendererDepthImage>,
    depth_view: Resource<RendererDepthImageView>,
    renderpass: Resource<RendererMainRenderpass>,
    framebuffers: Resource<RendererFramebuffers>,
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
            &mut *requests,
            &mut *backend,
            &window,
            &swapchain_views.0,
            depth_image.0,
            depth_view.0,
            &framebuffers.0,
            renderpass.0,
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
    let framebuffer = framebuffers.0[present_index as usize];

    command_list.reset();
    command_list.cmd(Command::Begin);
    command_list.cmd(Command::BeginRenderpass(
        shadowpass_data.renderpass,
        shadowpass_data.framebuffer,
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
                    shadowpass_data.light_dir_binding,
                    shadowpass_data.image_binding,
                );
                mesh.record_commands(command_list);
            },
        }
    }

    command_list.cmd(Command::EndRenderpass);
    command_list.cmd(Command::BeginRenderpass(renderpass.0, framebuffer));

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
                    shadowpass_data.light_dir_binding,
                    shadowpass_data.image_binding,
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
