use parsec_engine_ecs::{
    system::{SystemBundle, SystemTrigger},
    world::World,
};
use parsec_engine_error::{OptionNoneErr, ParsecError};
use parsec_engine_graphics::{
    ActiveEventLoop, ActiveGraphicsBackend, window::Window,
};
use parsec_engine_utils::identifiable::IdStore;
use parsec_engine_vulkan::VulkanBackend;

use parsec_engine_assets::assets::mesh::Mesh;

use crate::{
    ResizeFlag, camera_data::{CameraDataManager, add_camera_data, update_camera_data}, components::{
        camera::Camera, mesh_renderer::MeshRenderer, transform::Transform,
    }, draw_queue::{Draw, MeshAndMaterial}, light_data::update_light_data, mesh_data::add_mesh_data, queue_clear, render, transform_data::{TransformDataManager, add_transform_data, update_transform_data}
};

pub struct GraphicsBundle;
impl SystemBundle for GraphicsBundle {
    fn insert(self, systems: &mut parsec_engine_ecs::system::Systems) {
        systems.add(SystemTrigger::LateStart, init_window);
        systems.add(SystemTrigger::LateStart, init_vulkan);
        systems.add(SystemTrigger::LateStart, crate::init_renderer);
        systems.add(SystemTrigger::Render, update_camera_data);
        systems.add(SystemTrigger::Render, update_transform_data);
        systems.add(SystemTrigger::Render, update_light_data);
        systems.add(SystemTrigger::Render, auto_enqueue);
        systems.add(SystemTrigger::Render, render);
        systems.add(SystemTrigger::Render, queue_clear);
        systems.add(SystemTrigger::Update, request_redraw);
        systems.add(SystemTrigger::Update, add_camera_data);
        systems.add(SystemTrigger::Update, add_mesh_data);
        systems.add(SystemTrigger::Update, add_transform_data);
        systems.add(SystemTrigger::End, end_wait_idle);
        systems.add(SystemTrigger::WindowResized, mark_resize);
    }
}

pub fn init_vulkan(world: &mut World) -> Result<(), ParsecError> {
    let window = world.resource::<Window>();
    world.resources.add(
        ActiveGraphicsBackend::with_backend::<VulkanBackend>(&window)?,
    );
    world
        .resources
        .add_dependency::<ActiveGraphicsBackend, Window>()
        .unwrap();
    Ok(())
}

fn mark_resize(world: &World) {
    world.resource::<ResizeFlag>().0 = true;
}

fn request_redraw(world: &World) {
    world.resource::<Window>().request_redraw();
}

fn end_wait_idle(world: &World) {
    world.resource::<ActiveGraphicsBackend>().wait_idle();
}

fn init_window(world: &mut World) -> Result<(), ParsecError> {
    let window = {
        let event_loop = world.resource::<ActiveEventLoop>();
        let event_loop = event_loop.raw_active_event_loop()?;
        Window::new(event_loop, "Oxide Engine test")?
    };
    world.resources.add(window);
    Ok(())
}

fn auto_enqueue(world: &World) -> Result<(), ParsecError> {
    let mut draw_queue = world.resource::<Vec<Draw>>();
    let meshes = world.resource::<IdStore<Mesh>>();
    let camera_data_manager = world.resource::<CameraDataManager>();
    let transform_data_manager = world.resource::<TransformDataManager>();
    let mut cameras = world.query::<(Transform, Camera)>();
    let mut mesh_renderers = world.query::<(Transform, MeshRenderer)>();

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
