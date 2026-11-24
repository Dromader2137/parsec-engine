use std::marker::PhantomData;

use crate::{
    ctx::Ctx,
    ecs::system::{SystemBundle, SystemTrigger, Systems},
    error::SystemError,
    graphics::{ActiveEventLoop, ActiveGraphicsBackend, backend::GraphicsBackend, window::Window},
    renderer::{
        RendererError, ResizeFlag,
        camera_data::{CameraDataManager, add_camera_data, update_camera_data},
        components::{camera::Camera, mesh_renderer::MeshRenderer, transform::Transform},
        draw_queue::{Draw, MeshAndMaterial},
        get_resource, get_resource_mut, init_renderer,
        light_data::update_light_data,
        queue_clear, render,
        transform_data::{TransformDataManager, add_transform_data, update_transform_data},
    },
};

pub struct GraphicsBundle<B: GraphicsBackend> {
    _marker: PhantomData<B>,
}

impl<B: GraphicsBackend> Default for GraphicsBundle<B> {
    fn default() -> Self { Self { _marker: PhantomData } }
}

impl<B: GraphicsBackend> SystemBundle for GraphicsBundle<B> {
    fn insert(self, systems: &mut Systems) {
        systems.add(SystemTrigger::LateStart, init_window);
        systems.add(SystemTrigger::LateStart, |ctx: Ctx| -> Result<(), SystemError> {
            let backend = {
                let window = get_resource::<Window>(ctx.resources)?;
                ActiveGraphicsBackend::with_backend::<B>(&window).map_err(RendererError::from)?
            };
            ctx.resources.add(backend);
            ctx.resources
                .add_dependency::<ActiveGraphicsBackend, Window>()
                .map_err(RendererError::from)?;
            Ok(())
        });
        systems.add(SystemTrigger::LateStart, init_renderer);
        systems.add(SystemTrigger::Render, update_camera_data);
        systems.add(SystemTrigger::Render, update_transform_data);
        systems.add(SystemTrigger::Render, update_light_data);
        systems.add(SystemTrigger::Render, auto_enqueue);
        systems.add(SystemTrigger::Render, render);
        systems.add(SystemTrigger::Render, queue_clear);
        systems.add(SystemTrigger::Update, request_redraw);
        systems.add(SystemTrigger::Update, add_camera_data);
        systems.add(SystemTrigger::Update, add_transform_data);
        systems.add(SystemTrigger::End, end_wait_idle);
        systems.add(SystemTrigger::WindowResized, mark_resize);
    }
}

fn mark_resize(ctx: Ctx) -> Result<(), SystemError> {
    get_resource_mut::<ResizeFlag>(ctx.resources)?.0 = true;
    Ok(())
}

fn request_redraw(ctx: Ctx) -> Result<(), SystemError> {
    get_resource::<Window>(ctx.resources)?.request_redraw();
    Ok(())
}

fn end_wait_idle(ctx: Ctx) -> Result<(), SystemError> {
    get_resource::<ActiveGraphicsBackend>(ctx.resources)?.wait_idle();
    Ok(())
}

fn init_window(ctx: Ctx) -> Result<(), SystemError> {
    let window = {
        let event_loop = get_resource::<ActiveEventLoop>(ctx.resources)?;
        let event_loop = event_loop.raw_active_event_loop().map_err(RendererError::from)?;
        Window::new(event_loop, "Oxide Engine test").map_err(RendererError::from)?
    };
    ctx.resources.add(window);
    Ok(())
}

fn auto_enqueue(ctx: Ctx) -> Result<(), SystemError> {
    let mut draw_queue = get_resource_mut::<Vec<Draw>>(ctx.resources)?;
    let camera_data_manager = get_resource::<CameraDataManager>(ctx.resources)?;
    let transform_data_manager = get_resource::<TransformDataManager>(ctx.resources)?;
    let mut cameras = ctx.world.query::<(Transform, Camera)>().map_err(RendererError::from)?;
    let mut mesh_renderers =
        ctx.world.query::<(Transform, MeshRenderer)>().map_err(RendererError::from)?;

    for (_, (camera_transform, camera)) in cameras.iter() {
        for (_, (transform, mesh_renderer)) in mesh_renderers.iter() {
            let mesh_asset =
                mesh_renderer.mesh.resolve(ctx.assets).ok_or(RendererError::MeshAssetNotLoaded)?;
            let (Some(mesh_id), Some(camera_id), Some(camera_transform_id), Some(transform_id)) = (
                mesh_asset.data_id,
                camera_data_manager.component_to_data.get(&camera.camera_id()),
                transform_data_manager.component_to_data.get(&camera_transform.transform_id()),
                transform_data_manager.component_to_data.get(&transform.transform_id()),
            ) else {
                continue;
            };

            draw_queue.push(Draw::MeshAndMaterial(MeshAndMaterial {
                mesh: mesh_id,
                material: mesh_renderer.material_id,
                camera: *camera_id,
                camera_transform: *camera_transform_id,
                transform: *transform_id,
            }));
        }
    }
    Ok(())
}
