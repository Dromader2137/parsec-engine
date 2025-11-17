use std::sync::Arc;

use crate::{
    graphics::{
        renderer::{create_buffer, create_descriptor_set},
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

#[derive(Debug)]
pub struct TransformData {
    pub model_matrix: Matrix4f,
    pub model_buffer_id: u32,
    pub model_set_id: u32,
    pub look_at_matrix: Matrix4f,
    pub look_at_buffer_id: u32,
    pub look_at_set_id: u32,
    pub changed: bool,
}

pub fn create_transform_data(
    resources: &ResourceCollection,
    position: Vec3f,
    scale: Vec3f,
    rotation: Vec3f,
) -> Result<u32, VulkanError> {
    let _ = rotation;
    let _ = scale;
    let model_matrix = Matrix4f::translation(position);
    let look_at_matrix = Matrix4f::look_at(position, Vec3f::FORWARD, Vec3f::UP);
    let model_buffer_id = create_buffer(resources, vec![model_matrix])?;
    let look_at_buffer_id = create_buffer(resources, vec![look_at_matrix])?;
    let model_set_id = create_descriptor_set(resources, vec![DescriptorSetBinding::new(
        0,
        DescriptorType::UNIFORM_BUFFER,
        DescriptorStage::VERTEX,
    )])?;
    let look_at_set_id = create_descriptor_set(resources, vec![DescriptorSetBinding::new(
        0,
        DescriptorType::UNIFORM_BUFFER,
        DescriptorStage::VERTEX,
    )])?;
    {
        let descriptor_sets = resources.get::<IdVec<Arc<DescriptorSet>>>().unwrap();
        let buffers = resources.get::<IdVec<Arc<Buffer>>>().unwrap();
        let model_set = descriptor_sets.get(model_set_id).unwrap();
        let model_buffer = buffers.get(model_buffer_id).unwrap();
        model_set.bind_buffer(model_buffer.clone(), 0)?;
        let look_at_set = descriptor_sets.get(look_at_set_id).unwrap();
        let look_at_buffer = buffers.get(look_at_buffer_id).unwrap();
        look_at_set.bind_buffer(look_at_buffer.clone(), 0)?;
    }
    let transform_data = TransformData {
        model_matrix,
        model_buffer_id,
        model_set_id,
        look_at_matrix,
        look_at_buffer_id,
        look_at_set_id,
        changed: false,
    };
    let mut transforms = resources.get_mut::<IdVec<TransformData>>().unwrap();
    Ok(transforms.push(transform_data))
}
