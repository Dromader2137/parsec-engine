use std::collections::HashMap;

use parsec_engine_math::mat::Matrix4f;

use crate::{
    create_counter,
    ctx::Ctx,
    ecs::world::fetch::Mut,
    error::{OptionNoneErr, ParsecError},
    graphics::{
        ActiveGraphicsBackend,
        buffer::{Buffer, BufferBuilder, BufferContent, BufferUsage},
        pipeline::{
            PipelineBindingType, PipelineResource,
            PipelineResourceBindingLayout, PipelineResourceLayout,
            PipelineResourceLayoutBuilder, PipelineShaderStage,
        },
        window::Window,
    },
    renderer::components::camera::Camera,
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
    ) -> CameraData {
        let projection_matrix =
            Matrix4f::perspective(vfov, window.aspect_ratio(), near, far);
        let projection_buffer = BufferBuilder::new()
            .usage(&[BufferUsage::Uniform])
            .data(BufferContent::from_slice(&[projection_matrix]))
            .build(backend)
            .unwrap();
        let projection_layout = PipelineResourceLayoutBuilder::new()
            .binding(PipelineResourceBindingLayout {
                binding_type: PipelineBindingType::UniformBuffer,
                shader_stages: vec![PipelineShaderStage::Vertex],
            })
            .build(backend)
            .unwrap();
        let projection_binding =
            projection_layout.create_resource(backend).unwrap();
        projection_binding
            .bind_buffer(backend, projection_buffer.handle(), 0)
            .unwrap();
        CameraData {
            camera_data_id: ID_COUNTER.next(),
            projection_matrix,
            projection_buffer,
            projection_layout,
            projection_resource: projection_binding,
        }
    }

    pub fn destroy(self, backend: &mut ActiveGraphicsBackend) {
        self.projection_buffer.destroy(backend).unwrap();
        self.projection_resource.destroy(backend).unwrap();
        self.projection_layout.destroy(backend).unwrap();
    }
}

impl Identifiable for CameraData {
    fn id(&self) -> IdType { self.camera_data_id }
}

pub fn add_camera_data(ctx: Ctx) -> Result<(), ParsecError> {
    let window = ctx.resources.get::<Window>().none_err()?;
    let mut backend =
        ctx.resources.get_mut::<ActiveGraphicsBackend>().none_err()?;
    let mut cameras_data =
        ctx.resources.get_mut::<IdStore<CameraData>>().none_err()?;
    let mut camera_data_manager =
        ctx.resources.get_mut::<CameraDataManager>().none_err()?;
    let mut cameras = ctx.world.query::<Mut<Camera>>();

    for (_, camera) in cameras.iter() {
        if let std::collections::hash_map::Entry::Vacant(e) =
            camera_data_manager
                .component_to_data
                .entry(camera.camera_id())
        {
            let camera_data = CameraData::new(
                &mut backend,
                &window,
                camera.vertical_fov,
                camera.near_clipping_plane,
                camera.far_clipping_plane,
            );
            let data_id = cameras_data.push(camera_data);
            e.insert(data_id);
        }
    }

    Ok(())
}

pub fn update_camera_data(ctx: Ctx) -> Result<(), ParsecError> {
    let window = ctx.resources.get::<Window>().none_err()?;
    let mut backend =
        ctx.resources.get_mut::<ActiveGraphicsBackend>().none_err()?;
    let mut cameras_data =
        ctx.resources.get_mut::<IdStore<CameraData>>().none_err()?;
    let camera_data_manager =
        ctx.resources.get::<CameraDataManager>().none_err()?;
    let mut cameras = ctx.world.query::<Camera>();

    let aspect_ratio = window.aspect_ratio();
    for (_, camera) in cameras.iter() {
        if let Some(data_id) = camera_data_manager
            .component_to_data
            .get(&camera.camera_id())
        {
            let camera_data = cameras_data.get_mut(*data_id).unwrap();
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
                .unwrap();
        }
    }

    Ok(())
}
