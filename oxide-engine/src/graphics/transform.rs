use std::sync::Arc;

use crate::{
    graphics::{
        renderer::{create_buffer, create_descriptor_set},
        vulkan::{
            VulkanError, buffer::Buffer, descriptor_set::{DescriptorSet, DescriptorSetBinding, DescriptorStage, DescriptorType}
        },
    },
    math::{mat::Matrix4f, vec::Vec3f},
    resources::ResourceCollection,
    utils::id_vec::IdVec,
};

#[derive(Debug)]
pub struct TransformData {
    pub transform_matrix: Matrix4f,
    pub model_buffer_id: u32,
    pub model_set_id: u32,
}

pub fn create_transform_data(resources: &mut ResourceCollection, position: Vec3f) -> Result<u32, VulkanError> {
    let model_buffer_id = create_buffer(resources, vec![Matrix4f::translation(position)])?;
    let model_set_id = create_descriptor_set(
        resources,
        vec![DescriptorSetBinding::new(
            0,
            DescriptorType::UNIFORM_BUFFER,
            DescriptorStage::VERTEX,
        )],
    )?;
    {
        let descriptor_sets = resources.get::<IdVec<Arc<DescriptorSet>>>().unwrap();
        let buffers = resources.get::<IdVec<Arc<Buffer>>>().unwrap();
        let model_set = descriptor_sets.get(model_set_id).unwrap();
        let model_buffer = buffers.get(model_buffer_id).unwrap();
        model_set.bind_buffer(model_buffer.clone(), 0)?;
    }
    let transform_data = TransformData {
        transform_matrix: Matrix4f::translation(position),
        model_buffer_id,
        model_set_id,
    };
    let mut transforms = resources.get_mut::<IdVec<TransformData>>().unwrap();
    Ok(transforms.push(transform_data))
}
