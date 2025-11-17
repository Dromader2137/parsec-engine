use std::sync::Arc;

use crate::{
    components::transform::{Transform, TransformController}, ecs::world::{query::QueryIter, World}, graphics::{
        renderer::{create_buffer, create_descriptor_set},
        vulkan::{
            buffer::Buffer, descriptor_set::{
                DescriptorSet, DescriptorSetBinding, DescriptorStage, DescriptorType,
            }, VulkanError
        },
    }, math::mat::Matrix4f, resources::ResourceCollection, utils::id_vec::IdVec
};

#[derive(Debug)]
pub struct TransformData {
    pub transform_id: u32,
    pub transform_matrix: Matrix4f,
    pub model_buffer_id: u32,
    pub model_set_id: u32,
}

pub fn create_transform_data(
    resources: &mut ResourceCollection,
    world: &mut World,
    transform_id: u32,
) -> Result<u32, VulkanError> {
    let mut transform_components = world.query::<&[Transform]>().unwrap();
    let model = {
        let mut entity = None;
        while let Some((_, tra)) = transform_components.next() {
            if tra.id == transform_id {
                entity = Some(tra.clone());
                break;
            }
        }
        match entity {
            Some(transform) => Matrix4f::translation(transform.position),
            None => Matrix4f::indentity(),
        }
    };


    let model_buffer_id = create_buffer(resources, vec![model])?;
    let model_set_id = create_descriptor_set(resources, vec![DescriptorSetBinding::new(
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
    }
    let transform_data = TransformData {
        transform_id,
        transform_matrix: model,
        model_buffer_id,
        model_set_id,
    };
    let mut transforms = resources.get_mut::<IdVec<TransformData>>().unwrap();
    Ok(transforms.push(transform_data))
}

pub fn autoadd_transforms(
    resources: &mut ResourceCollection,
    world: &mut World,
) -> Result<(), VulkanError> {
    let transforms_to_add = {
        let mut transform_controller = resources.get_mut::<TransformController>().unwrap();
        let ret = transform_controller.just_added.clone();
        transform_controller.just_added.clear();
        ret
    };
    for id in transforms_to_add {
        create_transform_data(resources, world, id)?;
    }
    Ok(())
}
