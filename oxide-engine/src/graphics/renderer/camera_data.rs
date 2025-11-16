use std::sync::Arc;

use crate::{
    components::{camers::Camera, transform::Transform},
    ecs::world::{World, query::QueryIter},
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
    math::{mat::Matrix4f, vec::Vec3f},
    resources::ResourceCollection,
    utils::id_vec::IdVec,
};

pub struct CameraData {
    pub camera_id: u32,
    pub projection_matrix: Matrix4f,
    pub view_matrix: Matrix4f,
    pub projection_buffer_id: u32,
    pub view_buffer_id: u32,
    pub projection_set_id: u32,
    pub view_set_id: u32,
}

pub fn create_camera_data(
    resources: &mut ResourceCollection,
    world: &mut World,
    camera_id: u32,
) -> Result<u32, VulkanError> {
    let mut camera_components = world.query::<(&[Camera], &[Transform])>().unwrap();
    let (projection, view) = {
        let mut entity = None;
        while let Some((_, (cam, tra))) = camera_components.next() {
            if cam.id == camera_id {
                entity = Some((cam.clone(), tra.clone()));
                break;
            }
        }
        match entity {
            Some((camera, transform)) => (
                Matrix4f::perspective(
                    camera.vfov,
                    get_aspect_ratio(resources),
                    camera.near,
                    camera.far,
                ),
                Matrix4f::look_at(transform.position, Vec3f::FORWARD, Vec3f::UP),
            ),
            None => (Matrix4f::indentity(), Matrix4f::indentity()),
        }
    };
    let projection_buffer_id = create_buffer(resources, vec![projection])?;
    let view_buffer_id = create_buffer(resources, vec![view])?;
    let projection_set_id = create_descriptor_set(resources, vec![DescriptorSetBinding::new(
        0,
        DescriptorType::UNIFORM_BUFFER,
        DescriptorStage::VERTEX,
    )])?;
    let view_set_id = create_descriptor_set(resources, vec![DescriptorSetBinding::new(
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
        let view_set = descriptor_sets.get(view_set_id).unwrap();
        let view_buffer = buffers.get(view_buffer_id).unwrap();
        view_set.bind_buffer(view_buffer.clone(), 0)?;
    }
    let camera_data = CameraData {
        camera_id,
        projection_matrix: projection,
        view_matrix: view,
        projection_buffer_id,
        view_buffer_id,
        projection_set_id,
        view_set_id,
    };
    let mut cameras = resources.get_mut::<IdVec<CameraData>>().unwrap();
    Ok(cameras.push(camera_data))
}

pub fn autoadd_cameras(
    resources: &mut ResourceCollection,
    world: &mut World,
) -> Result<(), VulkanError> {
    let mut ids_to_add = Vec::new();
    {
        let mut camera_components = world.query::<&[Camera]>().unwrap();
        let cameras_data = resources.get_mut::<IdVec<CameraData>>().unwrap();
        while let Some((_, camera)) = camera_components.next() {
            if cameras_data
                .iter()
                .find(|x| x.camera_id == camera.id)
                .is_none()
            {
                ids_to_add.push(camera.id);
            }
        }
    }
    for id in ids_to_add {
        create_camera_data(resources, world, id)?;
    }
    Ok(())
}

pub fn update_camera_data(
    resources: &mut ResourceCollection,
    world: &mut World,
) -> Result<(), VulkanError> {
    let aspect_ratio = get_aspect_ratio(resources);
    let mut cameras = resources.get_mut::<IdVec<CameraData>>().unwrap();
    for camera in cameras.iter_mut() {
        let mut camera_components = world.query::<(&[Camera], &[Transform])>().unwrap();
        let (projection, view) = {
            let mut entity = None;
            while let Some((_, (cam, tra))) = camera_components.next() {
                if cam.id == camera.camera_id {
                    entity = Some((cam.clone(), tra.clone()));
                    break;
                }
            }
            match entity {
                Some((camera, transform)) => (
                    Matrix4f::perspective(camera.vfov, aspect_ratio, camera.near, camera.far),
                    Matrix4f::look_at(transform.position, Vec3f::FORWARD, Vec3f::UP),
                ),
                None => (Matrix4f::indentity(), Matrix4f::indentity()),
            }
        };
        let buffers = resources.get::<IdVec<Arc<Buffer>>>().unwrap();
        camera.projection_matrix = projection;
        camera.view_matrix = view;
        let projection_buffer = buffers.get(camera.projection_buffer_id).unwrap();
        let view_buffer = buffers.get(camera.view_buffer_id).unwrap();
        projection_buffer.update(vec![projection])?;
        view_buffer.update(vec![view])?;
    }
    Ok(())
}
