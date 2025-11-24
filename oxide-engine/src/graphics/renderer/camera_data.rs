use std::sync::Arc;

use crate::{
    ecs::world::{fetch::Mut, query::Query},
    graphics::{
        renderer::components::camera::Camera,
        vulkan::{
            VulkanError,
            buffer::{Buffer, BufferUsage},
            descriptor_set::{
                DescriptorPool, DescriptorSet, DescriptorSetBinding, DescriptorSetLayout,
                DescriptorStage, DescriptorType,
            },
            device::Device,
            surface::Surface,
        },
    },
    math::mat::Matrix4f,
    resources::Resource,
    system,
    utils::id_vec::IdVec,
};

pub struct CameraData {
    vfov: f32,
    near: f32,
    far: f32,
    pub projection_matrix: Matrix4f,
    pub projection_buffer: Arc<Buffer>,
    pub projection_set: Arc<DescriptorSet>,
}

impl CameraData {
    pub fn new(
        device: Arc<Device>,
        surface: Arc<Surface>,
        descriptor_pool: Arc<DescriptorPool>,
        vfov: f32,
        near: f32,
        far: f32,
    ) -> Result<CameraData, VulkanError> {
        let projection_matrix = Matrix4f::perspective(vfov, surface.aspect_ratio(), near, far);
        let projection_buffer = Buffer::from_vec(
            device.clone(),
            &[projection_matrix],
            BufferUsage::UNIFORM_BUFFER,
        )
        .unwrap();
        let projection_set_layout =
            DescriptorSetLayout::new(device, vec![DescriptorSetBinding::new(
                0,
                DescriptorType::UNIFORM_BUFFER,
                DescriptorStage::VERTEX,
            )])
            .unwrap();
        let projection_set = DescriptorSet::new(projection_set_layout, descriptor_pool).unwrap();
        projection_set
            .bind_buffer(projection_buffer.clone(), 0)
            .unwrap();
        Ok(CameraData {
            vfov,
            near,
            far,
            projection_matrix,
            projection_buffer,
            projection_set,
        })
    }
}

#[system]
fn add_camera_data(
    device: Resource<Arc<Device>>,
    surface: Resource<Arc<Surface>>,
    descriptor_pool: Resource<Arc<DescriptorPool>>,
    mut cameras_data: Resource<IdVec<CameraData>>,
    cameras: Query<Mut<Camera>>,
) {
    for camera in cameras.into_iter() {
        if camera.data_id.is_none() {
            let camera_data = CameraData::new(
                device.clone(),
                surface.clone(),
                descriptor_pool.clone(),
                camera.vertical_fov,
                camera.near_clipping_plane,
                camera.far_clipping_plane,
            )
            .unwrap();

            let data_id = cameras_data.push(camera_data);
            camera.data_id = Some(data_id);
        }
    }
}

#[system]
fn update_camera_data(surface: Resource<Arc<Surface>>, mut cameras: Resource<IdVec<CameraData>>) {
    let aspect_ratio = surface.aspect_ratio();
    for camera in cameras.iter_mut() {
        camera.projection_matrix =
            Matrix4f::perspective(camera.vfov, aspect_ratio, camera.near, camera.far);
        camera
            .projection_buffer
            .update(vec![camera.projection_matrix])
            .unwrap();
    }
}
