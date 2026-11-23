use std::collections::HashMap;

use parsec_engine_math::mat::Matrix4f;

use crate::{
    create_counter,
    ctx::Ctx,
    ecs::world::fetch::Mut,
    error::ParsecError,
    graphics::{
        ActiveGraphicsBackend,
        buffer::{Buffer, BufferBuilder, BufferContent, BufferUsage},
        pipeline::{
            PipelineBindingType, PipelineResource, PipelineResourceBindingLayout,
            PipelineResourceLayout, PipelineResourceLayoutBuilder, PipelineShaderStage,
        },
        window::Window,
    },
    renderer::{RendererError, components::camera::Camera, get_resource, get_resource_mut},
    utils::{
        IdType,
        identifiable::{IdStore, Identifiable},
    },
};

pub struct CameraData {
    camera_data_id: IdType,
    pub projection_matrix: Matrix4f,
    pub projection_buffer: Buffer,
    projection_layout: PipelineResourceLayout,
    pub projection_resource: PipelineResource,
}

pub struct CameraDataManager {
    pub component_to_data: HashMap<u32, u32>,
}

create_counter! {ID_COUNTER}
impl CameraData {
    pub fn new(
        backend: &mut ActiveGraphicsBackend,
        window: &Window,
        vfov: f32,
        near: f32,
        far: f32,
    ) -> Result<CameraData, RendererError> {
        let projection_matrix = Matrix4f::perspective(vfov, window.aspect_ratio(), near, far);
        let projection_buffer = BufferBuilder::new()
            .usage(&[BufferUsage::Uniform])
            .data(BufferContent::from_slice(&[projection_matrix]))
            .build(backend)?;
        let projection_layout = PipelineResourceLayoutBuilder::new()
            .binding(PipelineResourceBindingLayout {
                binding_type: PipelineBindingType::UniformBuffer,
                shader_stages: vec![PipelineShaderStage::Vertex],
            })
            .build(backend)?;
        let projection_binding = projection_layout.create_resource(backend)?;
        projection_binding.bind_buffer(backend, projection_buffer.handle(), 0)?;
        Ok(CameraData {
            camera_data_id: ID_COUNTER.next(),
            projection_matrix,
            projection_buffer,
            projection_layout,
            projection_resource: projection_binding,
        })
    }

    pub fn destroy(self, backend: &mut ActiveGraphicsBackend) -> Result<(), RendererError> {
        self.projection_buffer.destroy(backend)?;
        self.projection_resource.destroy(backend)?;
        self.projection_layout.destroy(backend)?;
        Ok(())
    }
}

impl Identifiable for CameraData {
    fn id(&self) -> IdType { self.camera_data_id }
}

pub fn add_camera_data(ctx: Ctx) -> Result<(), ParsecError> {
    let window = get_resource::<Window>(ctx.resources)?;
    let mut backend = get_resource_mut::<ActiveGraphicsBackend>(ctx.resources)?;
    let mut cameras_data = get_resource_mut::<IdStore<CameraData>>(ctx.resources)?;
    let mut camera_data_manager = get_resource_mut::<CameraDataManager>(ctx.resources)?;
    let mut cameras = ctx.world.query::<Mut<Camera>>().map_err(RendererError::from)?;

    for (_, camera) in cameras.iter() {
        if let std::collections::hash_map::Entry::Vacant(e) =
            camera_data_manager.component_to_data.entry(camera.camera_id())
        {
            let camera_data = CameraData::new(
                &mut backend,
                &window,
                camera.vertical_fov,
                camera.near_clipping_plane,
                camera.far_clipping_plane,
            )?;
            let data_id = cameras_data.push(camera_data);
            e.insert(data_id);
        }
    }

    Ok(())
}

pub fn update_camera_data(ctx: Ctx) -> Result<(), ParsecError> {
    let window = get_resource::<Window>(ctx.resources)?;
    let mut backend = get_resource_mut::<ActiveGraphicsBackend>(ctx.resources)?;
    let mut cameras_data = get_resource_mut::<IdStore<CameraData>>(ctx.resources)?;
    let camera_data_manager = get_resource::<CameraDataManager>(ctx.resources)?;
    let mut cameras = ctx.world.query::<Camera>().map_err(RendererError::from)?;

    let aspect_ratio = window.aspect_ratio();
    for (_, camera) in cameras.iter() {
        if let Some(data_id) = camera_data_manager.component_to_data.get(&camera.camera_id()) {
            let camera_data = cameras_data
                .get_mut(*data_id)
                .ok_or(RendererError::CameraDataNotFound(*data_id))?;
            camera_data.projection_matrix = Matrix4f::perspective(
                camera.vertical_fov,
                aspect_ratio,
                camera.near_clipping_plane,
                camera.far_clipping_plane,
            );
            backend
                .update_buffer(
                    camera_data.projection_buffer.handle(),
                    BufferContent::from_slice(&[camera_data.projection_matrix]),
                )
                .map_err(RendererError::from)?;
        }
    }

    Ok(())
}
