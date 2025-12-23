//! The built-in renderer.

use std::ops::DerefMut;

pub mod assets;
pub mod camera_data;
pub mod components;
pub mod draw_queue;
pub mod light_data;
pub mod material_data;
pub mod mesh_data;
pub mod sync;
pub mod transform_data;

use sync::{RendererFrameSync, RendererImageSync};

use crate::{
    ecs::system::system,
    graphics::{
        backend::GraphicsBackend,
        buffer::{Buffer, BufferUsage},
        command_list::CommandList,
        framebuffer::Framebuffer,
        image::{Image, ImageFormat, ImageUsage, ImageView},
        pipeline::{
            PipelineBinding, PipelineBindingType, PipelineShaderStage,
            PipelineSubbindingLayout,
        },
        renderer::{
            assets::mesh::Mesh,
            camera_data::CameraData,
            draw_queue::{Draw, MeshAndMaterial},
            material_data::{
                MaterialBase, MaterialData, MaterialPipelineBinding,
            },
            mesh_data::{MeshData, Vertex, VertexField, VertexFieldFormat},
            transform_data::TransformData,
        },
        renderpass::{
            Renderpass, RenderpassAttachment, RenderpassAttachmentType,
            RenderpassClearValue,
        },
        sampler::Sampler,
        shader::{Shader, ShaderType},
        swapchain::{Swapchain, SwapchainError},
        vulkan::{VulkanBackend, shader::read_shader_code},
        window::Window,
    },
    math::{
        mat::Matrix4f,
        vec::{Vec2f, Vec3f},
    },
    resources::{Resource, Resources},
    utils::id_vec::IdVec,
};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DefaultVertex {
    position: [f32; 3],
    normal: [f32; 3],
    tangent: [f32; 3],
    uv: [f32; 2],
}

impl Vertex for DefaultVertex {
    fn fields() -> Vec<VertexField> {
        vec![
            VertexField {
                format: VertexFieldFormat::Vec3,
            },
            VertexField {
                format: VertexFieldFormat::Vec3,
            },
            VertexField {
                format: VertexFieldFormat::Vec3,
            },
            VertexField {
                format: VertexFieldFormat::Vec2,
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
    backend: &mut impl GraphicsBackend,
    frames_in_flight: usize,
) -> Vec<RendererFrameSync> {
    let mut ret = Vec::new();
    for _ in 0..frames_in_flight {
        ret.push(RendererFrameSync::new(backend));
    }
    ret
}

fn create_image_sync(
    backend: &mut impl GraphicsBackend,
    image_count: usize,
) -> Vec<RendererImageSync> {
    let mut ret = Vec::new();
    for _ in 0..image_count {
        ret.push(RendererImageSync::new(backend));
    }
    ret
}

fn create_commad_lists(
    backend: &mut impl GraphicsBackend,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RendererSwapchain(pub Swapchain);
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
    light_dir_binding: PipelineBinding,
    renderpass: Renderpass,
    material_id: u32,
    vertex_shader: Shader,
    fragment_shader: Shader,
    image: Image,
    image_view: ImageView,
    image_sampler: Sampler,
    image_binding: PipelineBinding,
    framebuffer: Framebuffer,
    proj_buffer: Buffer,
    look_buffer: Buffer,
    proj_binding: PipelineBinding,
    look_binding: PipelineBinding,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct LD {
    dir: Vec3f,
    _pad: u8,
    mat: Matrix4f,
}

#[system]
pub fn init_renderer(
    mut backend: Resource<VulkanBackend>,
    window: Resource<Window>,
) {
    let renderpass = backend
        .create_renderpass(&[
            RenderpassAttachment {
                attachment_type: RenderpassAttachmentType::PresentColor,
                image_format: ImageFormat::RGBA8SRGB,
                clear_value: RenderpassClearValue::Color(0.0, 0.0, 0.0, 0.0),
            },
            RenderpassAttachment {
                attachment_type: RenderpassAttachmentType::PresentDepth,
                image_format: ImageFormat::D32,
                clear_value: RenderpassClearValue::Depth(1.0),
            },
        ])
        .unwrap();
    let (swapchain, swapchain_images) =
        backend.create_swapchain(&window, None).unwrap();
    let swapchain_image_views = swapchain_images
        .iter()
        .map(|img| backend.create_image_view(*img).unwrap())
        .collect::<Vec<_>>();
    let depth_image = backend
        .create_image(window.size(), ImageFormat::D32, &[
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
            attachment_type: RenderpassAttachmentType::StoreDepth,
            image_format: ImageFormat::D32,
            clear_value: RenderpassClearValue::Depth(1.0),
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
                PipelineSubbindingLayout::new(
                    PipelineBindingType::UniformBuffer,
                    PipelineShaderStage::Vertex,
                ),
                PipelineSubbindingLayout::new(
                    PipelineBindingType::UniformBuffer,
                    PipelineShaderStage::Vertex,
                ),
                PipelineSubbindingLayout::new(
                    PipelineBindingType::UniformBuffer,
                    PipelineShaderStage::Vertex,
                ),
            ],
            vec![PipelineSubbindingLayout::new(
                PipelineBindingType::UniformBuffer,
                PipelineShaderStage::Vertex,
            )],
            vec![PipelineSubbindingLayout::new(
                PipelineBindingType::UniformBuffer,
                PipelineShaderStage::Vertex,
            )],
        ],
    );
    let shadow_size = 4096;
    let shadow_depth_image = backend
        .create_image((shadow_size, shadow_size), ImageFormat::D32, &[
            ImageUsage::DepthAttachment,
            ImageUsage::Sampled,
        ])
        .unwrap();
    let shadow_depth_view =
        backend.create_image_view(shadow_depth_image).unwrap();
    let shadow_framebuffer = backend
        .create_framebuffer(
            (shadow_size, shadow_size),
            &[shadow_depth_view],
            shadow_renderpass,
        )
        .unwrap();
    let shadow_proj_buffer = backend
        .create_buffer(&[Matrix4f::orthographic(0.0, 100.0, 25.0, 25.0)], &[
            BufferUsage::Uniform,
        ])
        .unwrap();
    let shadow_look_buffer = backend
        .create_buffer(
            &[Matrix4f::look_at(
                Vec3f::new(-40.0, 40.0, -40.0),
                Vec3f::new(1.0, -1.0, 1.0),
                Vec3f::new(1.0, 1.0, 1.0),
            )],
            &[BufferUsage::Uniform],
        )
        .unwrap();
    let shadow_proj_layout = backend
        .create_pipeline_binding_layout(&[PipelineSubbindingLayout {
            binding_type: PipelineBindingType::UniformBuffer,
            shader_stage: PipelineShaderStage::Vertex,
        }])
        .unwrap();
    let shadow_look_layout = backend
        .create_pipeline_binding_layout(&[PipelineSubbindingLayout {
            binding_type: PipelineBindingType::UniformBuffer,
            shader_stage: PipelineShaderStage::Vertex,
        }])
        .unwrap();
    let shadow_proj_binding =
        backend.create_pipeline_binding(shadow_proj_layout).unwrap();
    let shadow_look_binding =
        backend.create_pipeline_binding(shadow_look_layout).unwrap();
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
        .create_pipeline_binding_layout(&[PipelineSubbindingLayout {
            binding_type: PipelineBindingType::TextureSampler,
            shader_stage: PipelineShaderStage::Fragment,
        }])
        .unwrap();
    let shadow_tex_binding =
        backend.create_pipeline_binding(shadow_tex_layout).unwrap();
    backend
        .bind_sampler(shadow_tex_binding, shadow_sampler, shadow_depth_view, 0)
        .unwrap();

    let light_buffer = backend
        .create_buffer(
            &[LD {
                dir: Vec3f::new(1.0, -1.0, 1.0),
                mat: Matrix4f::orthographic(0.0, 100.0, 25.0, 25.0)
                    * Matrix4f::look_at(
                        Vec3f::new(-40.0, 40.0, -40.0),
                        Vec3f::new(1.0, -1.0, 1.0),
                        Vec3f::new(1.0, 1.0, 1.0),
                    ),
                _pad: 0,
            }],
            &[BufferUsage::Uniform],
        )
        .unwrap();
    let light_binding_layout = backend
        .create_pipeline_binding_layout(&[PipelineSubbindingLayout::new(
            PipelineBindingType::UniformBuffer,
            PipelineShaderStage::Fragment,
        )])
        .unwrap();
    let light_binding = backend
        .create_pipeline_binding(light_binding_layout)
        .unwrap();
    backend.bind_buffer(light_binding, light_buffer, 0).unwrap();

    let mut material_bases = IdVec::new();
    let mut materials_data = IdVec::new();

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

    Resources::add(shadow_data).unwrap();
    Resources::add(RendererMainRenderpass(renderpass)).unwrap();
    Resources::add(RendererSwapchain(swapchain)).unwrap();
    Resources::add(RendererSwapchainImages(swapchain_images)).unwrap();
    Resources::add(RendererSwapchainImageViews(swapchain_image_views)).unwrap();
    Resources::add(RendererDepthImage(depth_image)).unwrap();
    Resources::add(RendererDepthImageView(depth_image_view)).unwrap();
    Resources::add(RendererFramebuffers(framebuffers)).unwrap();
    Resources::add(frame_sync).unwrap();
    Resources::add(image_sync).unwrap();
    Resources::add(command_lists).unwrap();
    Resources::add(RendererCurrentFrame(0)).unwrap();
    Resources::add(RendererFramesInFlight(frames_in_flight as u32)).unwrap();
    Resources::add(ResizeFlag(false)).unwrap();
    Resources::add(Vec::<Draw>::new()).unwrap();
    Resources::add(IdVec::<MeshData<DefaultVertex>>::new()).unwrap();
    Resources::add(IdVec::<Mesh>::new()).unwrap();
    Resources::add(material_bases).unwrap();
    Resources::add(materials_data).unwrap();
    Resources::add(IdVec::<TransformData>::new()).unwrap();
    Resources::add(IdVec::<CameraData>::new()).unwrap();
}

fn recreate_size_dependent_components(
    backend: &mut impl GraphicsBackend,
    window: &Window,
    swapchain: Swapchain,
    swapchain_images: &[Image],
    swapchain_views: &[ImageView],
    depth_image: Image,
    depth_view: ImageView,
    framebuffers: &[Framebuffer],
    renderpass: Renderpass,
) {
    backend.wait_idle();

    let (new_swapchain, new_swapchain_images) =
        backend.create_swapchain(window, Some(swapchain)).unwrap();
    let new_swapchain_image_views = new_swapchain_images
        .iter()
        .map(|img| backend.create_image_view(*img).unwrap())
        .collect::<Vec<_>>();
    let new_depth_image = backend
        .create_image(window.size(), ImageFormat::D32, &[
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

    backend.delete_swapchain(swapchain).unwrap();
    swapchain_views
        .iter()
        .for_each(|view| backend.delete_image_view(*view).unwrap());
    swapchain_images
        .iter()
        .for_each(|img| backend.delete_image(*img).unwrap());
    backend.delete_image_view(depth_view).unwrap();
    backend.delete_image(depth_image).unwrap();
    framebuffers.iter().for_each(|framebuffer| {
        backend.delete_framebuffer(*framebuffer).unwrap()
    });

    Resources::add_or_change(RendererSwapchain(new_swapchain));
    Resources::add_or_change(RendererSwapchainImages(new_swapchain_images));
    Resources::add_or_change(RendererSwapchainImageViews(
        new_swapchain_image_views,
    ));
    Resources::add_or_change(RendererDepthImage(new_depth_image));
    Resources::add_or_change(RendererDepthImageView(new_depth_image_view));
    Resources::add_or_change(RendererFramebuffers(new_framebuffers));
}

#[system]
pub fn render(
    mut backend: Resource<VulkanBackend>,
    mut current_frame: Resource<RendererCurrentFrame>,
    mut resize: Resource<ResizeFlag>,
    frames_in_flight: Resource<RendererFramesInFlight>,
    window: Resource<Window>,
    frame_sync: Resource<Vec<RendererFrameSync>>,
    image_sync: Resource<Vec<RendererImageSync>>,
    swapchain: Resource<RendererSwapchain>,
    swapchain_images: Resource<RendererSwapchainImages>,
    swapchain_views: Resource<RendererSwapchainImageViews>,
    depth_image: Resource<RendererDepthImage>,
    depth_view: Resource<RendererDepthImageView>,
    renderpass: Resource<RendererMainRenderpass>,
    framebuffers: Resource<RendererFramebuffers>,
    command_lists: Resource<Vec<CommandList>>,
    draw_queue: Resource<Vec<Draw>>,
    meshes_data: Resource<IdVec<MeshData<DefaultVertex>>>,
    materials_data: Resource<IdVec<MaterialData>>,
    material_bases: Resource<IdVec<MaterialBase>>,
    transforms_data: Resource<IdVec<TransformData>>,
    cameras_data: Resource<IdVec<CameraData>>,
    shadowpass_data: Resource<RendererShadowpassData>,
) {
    let command_buffer_fence =
        frame_sync[current_frame.0 as usize].command_buffer_fence;
    backend.wait_fence(command_buffer_fence).unwrap();

    if window.minimized() {
        return;
    }

    if resize.0 {
        recreate_size_dependent_components(
            backend.deref_mut(),
            &window,
            swapchain.0,
            &swapchain_images.0,
            &swapchain_views.0,
            depth_image.0,
            depth_view.0,
            &framebuffers.0,
            renderpass.0,
        );
        resize.0 = false;
        return;
    }

    let present_index = match backend.next_image_id(
        swapchain.0,
        frame_sync[current_frame.0 as usize].image_available_semaphore,
    ) {
        Ok(val) => val,
        Err(SwapchainError::SwapchainOutOfDate) => {
            resize.0 = true;
            return;
        },
        _ => panic!("Shouldn't be here"),
    };
    backend
        .reset_fence(frame_sync[current_frame.0 as usize].command_buffer_fence)
        .unwrap();

    let command_list = command_lists[current_frame.0 as usize];
    let framebuffer = framebuffers.0[present_index as usize];

    backend.command_reset(command_list).unwrap();
    backend.command_begin(command_list).unwrap();
    backend
        .command_begin_renderpass(
            command_list,
            shadowpass_data.renderpass,
            shadowpass_data.framebuffer,
        )
        .unwrap();

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
                    backend.deref_mut(),
                    command_list,
                    material_base,
                    camera,
                    camera_transform,
                    transform,
                    shadowpass_data.light_dir_binding,
                    shadowpass_data.image_binding,
                );
                mesh.record_commands(backend.deref_mut(), command_list);
            },
        }
    }

    backend.command_end_renderpass(command_list).unwrap();
    backend
        .command_begin_renderpass(command_list, renderpass.0, framebuffer)
        .unwrap();

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
                    backend.deref_mut(),
                    command_list,
                    material_base,
                    camera,
                    camera_transform,
                    transform,
                    shadowpass_data.light_dir_binding,
                    shadowpass_data.image_binding,
                );
                mesh.record_commands(backend.deref_mut(), command_list);
            },
        }
    }

    backend.command_end_renderpass(command_list).unwrap();
    backend.command_end(command_list).unwrap();

    backend
        .submit_commands(
            command_list,
            &[frame_sync[current_frame.0 as usize].image_available_semaphore],
            &[image_sync[present_index as usize].rendering_complete_semaphore],
            frame_sync[current_frame.0 as usize].command_buffer_fence,
        )
        .unwrap();

    match backend.present(
        swapchain.0,
        &[image_sync[present_index as usize].rendering_complete_semaphore],
        present_index,
    ) {
        Ok(_) => (),
        Err(SwapchainError::SwapchainOutOfDate) => {
            resize.0 = true;
        },
        _ => panic!("Shouldn't be here"),
    };

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
