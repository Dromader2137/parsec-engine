use parsec_engine_ecs::{
    resources::Resource,
    system::{System, SystemBundle, SystemTrigger, requests::Requests, system},
    world::query::Query,
};
use parsec_engine_error::{OptionNoneErr, ParsecError};
use parsec_engine_graphics::{
    ActiveEventLoop, ActiveGraphicsBackend, window::Window,
};
use parsec_engine_utils::identifiable::IdStore;
use parsec_engine_vulkan::VulkanBackend;

use crate::{
    InitRenderer, QueueClear, Render, ResizeFlag,
    assets::mesh::Mesh,
    camera_data::{AddCameraData, CameraDataManager, UpdateCameraData},
    components::{
        camera::Camera, mesh_renderer::MeshRenderer, transform::Transform,
    },
    draw_queue::{Draw, MeshAndMaterial},
    light_data::UpdateLightData,
    mesh_data::AddMeshData,
    transform_data::{
        AddTransformData, TransformDataManager, UpdateTransformData,
    },
};

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
            (SystemTrigger::Render, UpdateLightData::new()),
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
pub fn init_vulkan(
    mut requests: Resource<Requests>,
    window: Resource<Window>,
) -> Result<(), ParsecError> {
    requests.create_resource(ActiveGraphicsBackend::with_backend::<
        VulkanBackend,
    >(&window)?);
    requests.create_resource_dependency::<ActiveGraphicsBackend, Window>();
    Ok(())
}

#[system]
fn mark_resize(mut resize: Resource<ResizeFlag>) { resize.0 = true; }

#[system]
fn request_redraw(window: Resource<Window>) { window.request_redraw(); }

#[system]
fn end_wait_idle(backend: Resource<ActiveGraphicsBackend>) {
    backend.wait_idle();
}

#[system]
fn init_window(
    mut requests: Resource<Requests>,
    event_loop: Resource<ActiveEventLoop>,
) -> Result<(), ParsecError> {
    let window = {
        let event_loop = event_loop.raw_active_event_loop()?;
        Window::new(event_loop, "Oxide Engine test")?
    };
    requests.create_resource(window);
    Ok(())
}

#[system]
fn auto_enqueue(
    mut draw_queue: Resource<Vec<Draw>>,
    meshes: Resource<IdStore<Mesh>>,
    mut cameras: Query<(Transform, Camera)>,
    camera_data_manager: Resource<CameraDataManager>,
    mut mesh_renderers: Query<(Transform, MeshRenderer)>,
    transform_data_manager: Resource<TransformDataManager>,
) -> Result<(), ParsecError> {
    for (_, (camera_transform, camera)) in cameras.iter() {
        for (_, (transform, mesh_renderer)) in mesh_renderers.iter() {
            let mesh_asset = meshes.get(mesh_renderer.mesh_id).none_err()?;
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
                mesh: mesh_asset.data_id.none_err()?,
                material: mesh_renderer.material_id,
                camera: *camera_data_manager
                    .component_to_data
                    .get(&camera.camera_id())
                    .none_err()?,
                camera_transform: *transform_data_manager
                    .component_to_data
                    .get(&camera_transform.transform_id())
                    .none_err()?,
                transform: *transform_data_manager
                    .component_to_data
                    .get(&transform.transform_id())
                    .none_err()?,
            }));
        }
    }
    Ok(())
}
