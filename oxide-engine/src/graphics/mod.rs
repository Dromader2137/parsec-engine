//! Module responsible for graphics.

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
            camera_data::{AddCameraData, UpdateCameraData},
            components::{
                camera::Camera, mesh_renderer::MeshRenderer,
                transform::Transform,
            },
            draw_queue::{Draw, MeshAndMaterial},
            mesh_data::AddMeshData,
            transform_data::{AddTransformData, UpdateTransformData},
        },
        vulkan::VulkanBackend,
    },
    resources::{Resource, Resources},
    utils::id_vec::IdVec,
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
            (SystemTrigger::LateStart, InitWindow::new()),
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
            (SystemTrigger::WindowResized, MarkResize::new()),
        ]
    }
}

#[system]
pub fn init_vulkan(window: Resource<Window>) {
    let context = VulkanBackend::init(&window);
    Resources::add(context).unwrap();
}

#[system]
fn mark_resize(mut resize: Resource<ResizeFlag>) { resize.0 = true }

#[system]
fn request_redraw(window: Resource<Window>) { window.request_redraw(); }

#[system]
fn end_wait_idle(backend: Resource<VulkanBackend>) { backend.wait_idle() }

#[system]
fn init_window() {
    let window = {
        let event_loop = app::ACTIVE_EVENT_LOOP.take().unwrap();
        let event_loop_raw = event_loop.get_event_loop();
        Window::new(event_loop_raw, "Oxide Engine test").unwrap()
    };
    Resources::add(window).unwrap();
}

#[system]
fn auto_enqueue(
    mut draw_queue: Resource<Vec<Draw>>,
    meshes: Resource<IdVec<Mesh>>,
    mut cameras: Query<(Transform, Camera)>,
    mut mesh_renderers: Query<(Transform, MeshRenderer)>,
) {
    for (_, (camera_transform, camera)) in cameras.iter() {
        for (_, (transform, mesh_renderer)) in mesh_renderers.iter() {
            let mesh_asset = meshes.get(mesh_renderer.mesh_id).unwrap();
            if mesh_asset.data_id.is_none()
                || camera.data_id.is_none()
                || camera_transform.data_id.is_none()
                || transform.data_id.is_none()
            {
                continue;
            }

            draw_queue.push(Draw::MeshAndMaterial(MeshAndMaterial {
                mesh: mesh_asset.data_id.unwrap(),
                material: mesh_renderer.material_id,
                camera: camera.data_id.unwrap(),
                camera_transform: camera_transform.data_id.unwrap(),
                transform: transform.data_id.unwrap(),
            }));
        }
    }
}
