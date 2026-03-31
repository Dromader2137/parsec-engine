//! Module responsible for graphics.
//!
//! # Vulkan API abstraction layers:
//!
//! ## Bindings layer
//!
//! This layer is entirely provided by the [`ash`] crate.
//!
//! ## Wrapper layer
//!
//! Wrapper around the raw vulkan bindings provided by [`ash`] that provides error handling and
//! custom enums.
//!
//! ## Backend layer
//!
//! Graphics back-end trait that lets higher level abstractions use different graphics APIs.
//!
//! ## Renderer layer
//!
//! Renderer that automatically manages images, buffers, etc.
//!
//! Layers can only use types and functions provided by the layer directly below it

use std::ops::{Deref, DerefMut};

use window::Window;

use crate::{
    app::{self},
    ecs::{
        system::{System, SystemBundle, SystemTrigger, system},
        world::query::Query,
    },
    graphics::{
        backend::GraphicsBackend,
        renderer::{
            InitRenderer, QueueClear, Render, ResizeFlag,
            assets::mesh::Mesh,
            camera_data::{AddCameraData, CameraDataManager, UpdateCameraData},
            components::{
                camera::Camera, mesh_renderer::MeshRenderer,
                transform::Transform,
            },
            draw_queue::{Draw, MeshAndMaterial},
            mesh_data::AddMeshData,
            transform_data::{
                AddTransformData, TransformDataManager, UpdateTransformData,
            },
        },
        vulkan::VulkanBackend,
    },
    resources::{Resource, Resources},
    utils::identifiable::IdStore,
};

pub mod backend;
pub mod buffer;
pub mod command_list;
pub mod fence;
pub mod framebuffer;
pub mod image;
pub mod pipeline;
pub mod renderer;
pub mod renderpass;
pub mod sampler;
pub mod semaphore;
pub mod shader;
pub mod swapchain;
pub mod vulkan;
pub mod window;

#[derive(Default)]
pub struct GraphicsBundle {}
impl SystemBundle for GraphicsBundle {
    fn systems(self) -> Vec<(SystemTrigger, Box<dyn System>)> {
        vec![
            (SystemTrigger::LateStart, InitVulkan::new()),
            (SystemTrigger::LateStart, InitRenderer::new()),
            (SystemTrigger::Render, UpdateCameraData::new()),
            (SystemTrigger::Render, UpdateTransformData::new()),
            (SystemTrigger::Render, AutoEnqueue::new()),
            (SystemTrigger::Render, Render::new()),
            (SystemTrigger::Render, QueueClear::new()),
            (SystemTrigger::Update, RequestRedraw::new()),
            (SystemTrigger::Update, AddCameraData::new()),
            (SystemTrigger::Update, AddMeshData::new()),
            (SystemTrigger::Update, AddTransformData::new()),
            (SystemTrigger::End, EndWaitIdle::new()),
            (SystemTrigger::WindowResized, HandleResize::new()),
        ]
    }
}

pub struct CurrentGraphicsBackend(Box<dyn GraphicsBackend>);

impl Deref for CurrentGraphicsBackend {
    type Target = Box<dyn GraphicsBackend>;
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for CurrentGraphicsBackend {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

#[system]
pub fn init_vulkan() {
    let context = VulkanBackend::init().unwrap();
    Resources::add(CurrentGraphicsBackend(Box::new(context))).unwrap();
}

#[system]
fn handle_resize(mut backend: Resource<CurrentGraphicsBackend>) {
    backend.handle_resize();
}

#[system]
fn request_redraw(backend: Resource<CurrentGraphicsBackend>) {
    backend.request_redraw();
}

#[system]
fn end_wait_idle(backend: Resource<CurrentGraphicsBackend>) {
    backend.wait_idle()
}

#[system]
fn auto_enqueue(
    mut draw_queue: Resource<Vec<Draw>>,
    meshes: Resource<IdStore<Mesh>>,
    mut cameras: Query<(Transform, Camera)>,
    camera_data_manager: Resource<CameraDataManager>,
    mut mesh_renderers: Query<(Transform, MeshRenderer)>,
    transform_data_manager: Resource<TransformDataManager>,
) {
    for (_, (camera_transform, camera)) in cameras.iter() {
        for (_, (transform, mesh_renderer)) in mesh_renderers.iter() {
            let mesh_asset = meshes.get(mesh_renderer.mesh_id).unwrap();
            if mesh_asset.data_id.is_none()
                || !camera_data_manager
                    .component_to_data
                    .contains_key(&camera.camera_id())
                || !transform_data_manager
                    .component_to_data
                    .contains_key(&camera_transform.transform_id())
                || !transform_data_manager
                    .component_to_data
                    .contains_key(&transform.transform_id())
            {
                continue;
            }

            draw_queue.push(Draw::MeshAndMaterial(MeshAndMaterial {
                mesh: mesh_asset.data_id.unwrap(),
                material: mesh_renderer.material_id,
                camera: *camera_data_manager
                    .component_to_data
                    .get(&camera.camera_id())
                    .unwrap(),
                camera_transform: *transform_data_manager
                    .component_to_data
                    .get(&camera_transform.transform_id())
                    .unwrap(),
                transform: *transform_data_manager
                    .component_to_data
                    .get(&transform.transform_id())
                    .unwrap(),
            }));
        }
    }
}
