use std::sync::{Arc, Mutex};

use vulkan::VulkanError;
use window::{WindowError, WindowWrapper};

use crate::{
    app::{self},
    ecs::{
        system::{System, SystemBundle, SystemTrigger},
        world::{World, query::{OldQuery, OldQueryIter}},
    },
    error::EngineError,
    graphics::{
        renderer::{
            InitRenderer, QueueClear, Render,
            assets::mesh::Mesh,
            camera_data::{AddCameraData, UpdateCameraData},
            components::{camera::Camera, mesh_renderer::MeshRenderer, transform::Transform},
            draw_queue::{Draw, MeshAndMaterial},
            mesh_data::AddMeshData,
            transform_data::AddTransformData,
        },
        vulkan::{context::InitVulkan, device::Device},
    },
    resources::{Resource, Resources},
    system,
    utils::id_vec::IdVec,
};

pub mod renderer;
pub mod vulkan;
pub mod window;

#[derive(Debug)]
pub enum GraphicsError {
    WindowError(WindowError),
    VulkanError(VulkanError),
    Uninitialized,
}

impl From<GraphicsError> for EngineError {
    fn from(value: GraphicsError) -> Self { EngineError::GraphicsError(value) }
}

#[derive(Default)]
pub struct GraphicsBundle {}
impl SystemBundle for GraphicsBundle {
    fn systems(self) -> Vec<(SystemTrigger, Box<dyn System>)> {
        vec![
            (SystemTrigger::LateStart, InitWindow::new()),
            (SystemTrigger::LateStart, InitVulkan::new()),
            (SystemTrigger::LateStart, InitRenderer::new()),
            (SystemTrigger::Render, UpdateCameraData::new()),
            (SystemTrigger::Render, AutoEnqueue::new()),
            (SystemTrigger::Render, Render::new()),
            (SystemTrigger::Render, QueueClear::new()),
            (SystemTrigger::Update, RequestRedraw::new()),
            (SystemTrigger::Update, AddCameraData::new()),
            (SystemTrigger::Update, AddMeshData::new()),
            (SystemTrigger::Update, AddTransformData::new()),
            (SystemTrigger::End, EndWaitIdle::new()),
        ]
    }
}

#[system]
fn request_redraw(window: Resource<Arc<WindowWrapper>>) { window.request_redraw(); }

#[system]
fn end_wait_idle(device: Resource<Arc<Device>>) { device.wait_idle().unwrap() }

#[system]
fn init_window() {
    let window = {
        let event_loop = app::ACTIVE_EVENT_LOOP.take().unwrap();
        let event_loop_raw = event_loop.get_event_loop();
        WindowWrapper::new(event_loop_raw, "Oxide Engine test").unwrap()
    };
    Resources::add(window).unwrap();
}

// #[system]
fn auto_enqueue(
    mut draw_queue: Resource<Vec<Draw>>,
    meshes: Resource<IdVec<Mesh>>,
    mut cameras: OldQuery<(&[Transform], &[Camera])>,
    mut mesh_renderers: OldQuery<(&[Transform], &[MeshRenderer])>,
) {
    while let Some((_, (camera_transform, camera))) = cameras.next() {
        while let Some((_, (transform, mesh_renderer))) = mesh_renderers.next() {
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
pub struct AutoEnqueue;
    impl AutoEnqueue {
        pub fn new() -> Box<Self> {
            Box::new(Self)
        }
    }
    impl crate::ecs::system::System for AutoEnqueue {
        fn run(&mut self) {
            let mut draw_queue = <Resource<
                Vec<Draw>,
            > as crate::ecs::system::SystemInput>::borrow();
            let meshes = <Resource<
                IdVec<Mesh>,
            > as crate::ecs::system::SystemInput>::borrow();
            let mut cameras = <OldQuery<
                (&[Transform], &[Camera]),
            > as crate::ecs::system::SystemInput>::borrow();
            let mut mesh_renderers = <OldQuery<
                (&[Transform], &[MeshRenderer]),
            > as crate::ecs::system::SystemInput>::borrow();
            auto_enqueue(draw_queue, meshes, cameras, mesh_renderers);
        }
    }
