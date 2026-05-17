use std::marker::PhantomData;

use crate::{
    assets::core::mesh::Mesh,
    ctx::Ctx,
    ecs::system::{SystemBundle, SystemTrigger, Systems},
    error::{OptionNoneErr, ParsecError},
    graphics::{
        ActiveEventLoop, ActiveGraphicsBackend, backend::GraphicsBackend,
        window::Window,
    },
    renderer::{
        ResizeFlag,
        camera_data::{CameraDataManager, add_camera_data, update_camera_data},
        components::{
            camera::Camera, mesh_renderer::MeshRenderer, transform::Transform,
        },
        draw_queue::{Draw, MeshAndMaterial},
        init_renderer,
        light_data::update_light_data,
        queue_clear, render,
        transform_data::{
            TransformDataManager, add_transform_data, update_transform_data,
        },
    },
};

pub struct GraphicsBundle<B: GraphicsBackend> {
    _marker: PhantomData<B>,
}

impl<B: GraphicsBackend> Default for GraphicsBundle<B> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<B: GraphicsBackend> SystemBundle for GraphicsBundle<B> {
    fn insert(self, systems: &mut Systems) {
        systems.add(SystemTrigger::LateStart, init_window);
        systems.add(SystemTrigger::LateStart, |ctx: Ctx| {
            let window = ctx.resources.get::<Window>().none_err()?;
            ctx.resources
                .add(ActiveGraphicsBackend::with_backend::<B>(&window)?);
            ctx.resources
                .add_dependency::<ActiveGraphicsBackend, Window>()
                .unwrap();
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

fn mark_resize(ctx: Ctx) -> Result<(), ParsecError> {
    ctx.resources.get::<ResizeFlag>().none_err()?.0 = true;
    Ok(())
}

fn request_redraw(ctx: Ctx) -> Result<(), ParsecError> {
    ctx.resources.get::<Window>().none_err()?.request_redraw();
    Ok(())
}

fn end_wait_idle(ctx: Ctx) -> Result<(), ParsecError> {
    ctx.resources
        .get::<ActiveGraphicsBackend>()
        .none_err()?
        .wait_idle();
    Ok(())
}

fn init_window(ctx: Ctx) -> Result<(), ParsecError> {
    let window = {
        let event_loop = ctx.resources.get::<ActiveEventLoop>().none_err()?;
        let event_loop = event_loop.raw_active_event_loop()?;
        Window::new(event_loop, "Oxide Engine test")?
    };
    ctx.resources.add(window);
    Ok(())
}

fn auto_enqueue(ctx: Ctx) -> Result<(), ParsecError> {
    let mut draw_queue = ctx.resources.get::<Vec<Draw>>().none_err()?;
    let camera_data_manager =
        ctx.resources.get::<CameraDataManager>().none_err()?;
    let transform_data_manager =
        ctx.resources.get::<TransformDataManager>().none_err()?;
    let mut cameras = ctx.world.query::<(Transform, Camera)>();
    let mut mesh_renderers = ctx.world.query::<(Transform, MeshRenderer)>();

    for (_, (camera_transform, camera)) in cameras.iter() {
        for (_, (transform, mesh_renderer)) in mesh_renderers.iter() {
            let mesh_asset =
                ctx.assets.get::<Mesh>(mesh_renderer.mesh).none_err()?;
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
