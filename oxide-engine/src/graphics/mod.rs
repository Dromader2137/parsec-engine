use std::sync::Arc;

use vulkan::VulkanError;
use window::{WindowError, WindowWrapper};

use crate::{
    app::{self},
    assets::library::AssetLibrary,
    ecs::{
        system::{System, SystemBundle, SystemInput, SystemTrigger},
        world::{World, query::QueryIter},
    },
    error::EngineError,
    graphics::{
        renderer::{
            assets::mesh::Mesh,
            camera_data::update_camera_data,
            components::{camera::Camera, mesh_renderer::MeshRenderer, transform::Transform},
            draw_queue::{Draw, MeshAndMaterial},
            init_renderer, queue_clear, queue_draw, render,
        },
        vulkan::{context::init_vulkan, device::Device},
    }, resources::Rsc,
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
    fn systems(self) -> Vec<System> {
        vec![
            System::new(
                SystemTrigger::LateStart,
                |SystemInput { .. }| {
                    let window = {
                        let event_loop = app::ACTIVE_EVENT_LOOP.take().unwrap();
                        let event_loop_raw = event_loop.get_event_loop();
                        WindowWrapper::new(event_loop_raw, "Oxide Engine test").unwrap()
                    };
                    Rsc::add(window).unwrap();
                    init_vulkan().unwrap();
                    init_renderer().unwrap();
                },
            ),
            System::new(
                SystemTrigger::Render,
                |SystemInput {
                     world,
                     assets,
                 }| {
                    update_camera_data().unwrap();
                    auto_enqueue(world, assets);
                    render().unwrap();
                    queue_clear();
                },
            ),
            System::new(
                SystemTrigger::Update,
                |SystemInput { .. }| {
                    let window = Rsc::<Arc<WindowWrapper>>::get().unwrap();
                    window.request_redraw(); 
                },
            ),
            System::new(SystemTrigger::End, |SystemInput { .. }| {
                let device = Rsc::<Arc<Device>>::get().unwrap();
                device.wait_idle().unwrap();
            }),
        ]
    }
}

fn auto_enqueue(world: &World, assets: &AssetLibrary) {
    let mut cameras = world.query::<(&[Transform], &[Camera])>().unwrap();
    let mut mesh_renderers = world.query::<(&[Transform], &[MeshRenderer])>().unwrap();
    while let Some((_, (camera_transform, camera))) = cameras.next() {
        while let Some((_, (transform, mesh_renderer))) = mesh_renderers.next() {
            let mesh = assets
                .get_one::<Mesh>(mesh_renderer.mesh_id as usize)
                .unwrap();
            if mesh.data_id.is_none() {
                continue;
            }
            queue_draw(
                Draw::MeshAndMaterial(MeshAndMaterial {
                    mesh_id: mesh.data_id.unwrap(),
                    material_id: mesh_renderer.material_id,
                    transform_id: transform.data_id,
                    camera_transform_id: camera_transform.data_id,
                    camera_id: camera.data_id,
                }),
            );
        }
    }
}
