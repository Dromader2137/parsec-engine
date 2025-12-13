use std::ops::DerefMut;

use crate::{
    ecs::{
        system::system,
        world::{fetch::Mut, query::Query},
    },
    graphics::{
        backend::GraphicsBackend,
        buffer::{Buffer, BufferUsage},
        pipeline::{
            PipelineBinding, PipelineBindingType, PipelineShaderStage,
            PipelineSubbindingLayout,
        },
        renderer::components::camera::Camera,
        vulkan::VulkanBackend,
        window::Window,
    },
    math::mat::Matrix4f,
    resources::Resource,
    utils::id_vec::IdVec,
};

pub struct CameraData {
    pub projection_matrix: Matrix4f,
    pub projection_buffer: Buffer,
    pub projection_binding: PipelineBinding,
}

impl CameraData {
    pub fn new(
        backend: &mut impl GraphicsBackend,
        window: &Window,
        vfov: f32,
        near: f32,
        far: f32,
    ) -> CameraData {
        let projection_matrix =
            Matrix4f::perspective(vfov, window.aspect_ratio(), near, far);
        let projection_buffer = backend
            .create_buffer(&[projection_matrix], &[BufferUsage::Uniform])
            .unwrap();
        let projection_binding_layout = backend
            .create_pipeline_binding_layout(&[PipelineSubbindingLayout {
                binding_type: PipelineBindingType::UniformBuffer,
                shader_stage: PipelineShaderStage::Vertex,
            }])
            .unwrap();
        let projection_binding = backend
            .create_pipeline_binding(projection_binding_layout)
            .unwrap();
        backend
            .bind_buffer(projection_binding, projection_buffer, 0)
            .unwrap();
        CameraData {
            projection_matrix,
            projection_buffer,
            projection_binding,
        }
    }
}

#[system]
fn add_camera_data(
    window: Resource<Window>,
    mut backend: Resource<VulkanBackend>,
    mut cameras_data: Resource<IdVec<CameraData>>,
    mut cameras: Query<Mut<Camera>>,
) {
    for (_, camera) in cameras.iter() {
        if camera.data_id.is_none() {
            let camera_data = CameraData::new(
                backend.deref_mut(),
                &window,
                camera.vertical_fov,
                camera.near_clipping_plane,
                camera.far_clipping_plane,
            );
            let data_id = cameras_data.push(camera_data);
            camera.data_id = Some(data_id);
        }
    }
}

#[system]
fn update_camera_data(
    window: Resource<Window>,
    mut backend: Resource<VulkanBackend>,
    mut cameras_data: Resource<IdVec<CameraData>>,
    mut cameras: Query<Camera>,
) {
    let aspect_ratio = window.aspect_ratio();
    for (_, camera) in cameras.iter() {
        if camera.data_id.is_none() {
            continue;
        }
        let camera_data =
            cameras_data.get_mut(camera.data_id.unwrap()).unwrap();
        camera_data.projection_matrix = Matrix4f::perspective(
            camera.vertical_fov,
            aspect_ratio,
            camera.near_clipping_plane,
            camera.far_clipping_plane,
        );
        backend
            .update_buffer(camera_data.projection_buffer, &[
                camera_data.projection_matrix
            ])
            .unwrap();
    }
}
