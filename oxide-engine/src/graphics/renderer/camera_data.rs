use std::sync::Arc;

use crate::{
    graphics::{
        renderer::{create_buffer, create_descriptor_set, get_aspect_ratio},
        vulkan::{
            VulkanError,
            buffer::Buffer,
            descriptor_set::{
                DescriptorSet, DescriptorSetBinding, DescriptorStage, DescriptorType,
            },
        },
    },
    math::mat::Matrix4f,
    resources::ResourceCollection,
    utils::id_vec::IdVec,
};

pub struct CameraData {
    vfov: f32,
    near: f32,
    far: f32,
    pub projection_matrix: Matrix4f,
    pub projection_buffer_id: u32,
    pub projection_set_id: u32,
    pub changed: bool,
}

pub fn create_camera_data(
    resources: &ResourceCollection,
    vfov: f32,
    near: f32,
    far: f32,
) -> Result<u32, VulkanError> {
    let projection_matrix = Matrix4f::perspective(vfov, get_aspect_ratio(resources), near, far);
    let projection_buffer_id = create_buffer(resources, vec![projection_matrix])?;
    let projection_set_id = create_descriptor_set(resources, vec![DescriptorSetBinding::new(
        0,
        DescriptorType::UNIFORM_BUFFER,
        DescriptorStage::VERTEX,
    )])?;
    {
        let descriptor_sets = resources.get::<IdVec<Arc<DescriptorSet>>>().unwrap();
        let buffers = resources.get::<IdVec<Arc<Buffer>>>().unwrap();
        let projection_set = descriptor_sets.get(projection_set_id).unwrap();
        let projection_buffer = buffers.get(projection_buffer_id).unwrap();
        projection_set.bind_buffer(projection_buffer.clone(), 0)?;
    }
    let camera_data = CameraData {
        vfov,
        near,
        far,
        projection_matrix,
        projection_buffer_id,
        projection_set_id,
        changed: false,
    };
    let mut cameras = resources.get_mut::<IdVec<CameraData>>().unwrap();
    Ok(cameras.push(camera_data))
}

pub fn update_camera_data(resources: &ResourceCollection) -> Result<(), VulkanError> {
    let aspect_ratio = get_aspect_ratio(resources);
    let mut cameras = resources.get_mut::<IdVec<CameraData>>().unwrap();
    let mut buffers = resources.get_mut::<IdVec<Arc<Buffer>>>().unwrap();
    for camera in cameras.iter_mut() {
        camera.projection_matrix =
            Matrix4f::perspective(camera.vfov, aspect_ratio, camera.near, camera.far);
        let projection_buffer = buffers.get_mut(camera.projection_buffer_id).unwrap();
        projection_buffer.update(vec![camera.projection_matrix])?;
    }
    Ok(())
}
