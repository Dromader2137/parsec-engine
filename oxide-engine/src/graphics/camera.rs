use std::sync::Arc;

use crate::{
  graphics::{
    renderer::{create_buffer, create_descriptor_set, get_aspect_ratio},
    vulkan::{
      VulkanError,
      buffer::Buffer,
      descriptor_set::{DescriptorSet, DescriptorSetBinding, DescriptorStage, DescriptorType},
    },
  },
  math::mat::Matrix4f,
  resources::ResourceCollection,
  utils::id_vec::IdVec,
};

pub struct CameraData {
  pub projection_matrix: Matrix4f,
  pub view_matrix: Matrix4f,
  pub projection_buffer_id: u32,
  pub view_buffer_id: u32,
  pub projection_set_id: u32,
  pub view_set_id: u32,
}

pub fn create_camera_data(
  resources: &mut ResourceCollection,
  fov: f32,
  near: f32,
  far: f32,
) -> Result<u32, VulkanError> {
  let projection = Matrix4f::perspective(fov, get_aspect_ratio(resources), near, far);
  let projection_buffer_id = create_buffer(resources, vec![projection])?;
  let view_buffer_id = create_buffer(resources, vec![Matrix4f::indentity()])?;
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
    projection_matrix: projection,
    view_matrix: Matrix4f::indentity(),
    projection_buffer_id,
    view_buffer_id,
    projection_set_id,
    view_set_id,
  };
  let mut cameras = resources.get_mut::<IdVec<CameraData>>().unwrap();
  Ok(cameras.push(camera_data))
}

pub fn update_camera_data(
  resources: &mut ResourceCollection,
  fov: f32,
  near: f32,
  far: f32,
) -> Result<(), VulkanError> {
  let projection = Matrix4f::perspective(fov, get_aspect_ratio(resources), near, far);
  let mut cameras = resources.get_mut::<IdVec<CameraData>>().unwrap();
  let buffers = resources.get::<IdVec<Arc<Buffer>>>().unwrap();
  for camera in cameras.iter_mut() {
    camera.projection_matrix = projection;
    let projection_buffer = buffers.get(camera.projection_buffer_id).unwrap();
    projection_buffer.update(vec![projection])?;
  }
  Ok(())
}
