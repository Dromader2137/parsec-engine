use std::sync::Arc;

use crate::{
    ecs::{
        system::system,
        world::{fetch::Mut, query::Query},
    },
    graphics::{
        renderer::components::transform::Transform,
        vulkan::{
            VulkanError,
            buffer::{Buffer, BufferUsage},
            descriptor_set::{
                DescriptorPool, DescriptorSet, DescriptorSetBinding, DescriptorSetLayout,
                DescriptorStage, DescriptorType,
            },
            device::Device,
        },
    },
    math::{mat::Matrix4f, vec::Vec3f},
    resources::Resource,
    utils::id_vec::IdVec,
};

pub struct TransformData {
    pub model_matrix: Matrix4f,
    pub model_buffer: Arc<Buffer>,
    pub model_set: Arc<DescriptorSet>,
    pub look_at_matrix: Matrix4f,
    pub look_at_buffer: Arc<Buffer>,
    pub look_at_set: Arc<DescriptorSet>,
}

impl TransformData {
    pub fn new(
        device: Arc<Device>,
        descriptor_pool: Arc<DescriptorPool>,
        position: Vec3f,
        scale: Vec3f,
        rotation: Vec3f,
    ) -> Result<TransformData, VulkanError> {
        let _ = rotation;
        let _ = scale;
        let model_matrix = Matrix4f::translation(position);
        let look_at_matrix = Matrix4f::look_at(position, Vec3f::FORWARD, Vec3f::UP);
        let model_buffer =
            Buffer::from_vec(device.clone(), &[model_matrix], BufferUsage::UNIFORM_BUFFER).unwrap();
        let look_at_buffer = Buffer::from_vec(
            device.clone(),
            &[look_at_matrix],
            BufferUsage::UNIFORM_BUFFER,
        )
        .unwrap();
        let model_set_layout =
            DescriptorSetLayout::new(device.clone(), vec![DescriptorSetBinding::new(
                0,
                DescriptorType::UNIFORM_BUFFER,
                DescriptorStage::VERTEX,
            )])
            .unwrap();
        let look_at_set_layout = DescriptorSetLayout::new(device, vec![DescriptorSetBinding::new(
            0,
            DescriptorType::UNIFORM_BUFFER,
            DescriptorStage::VERTEX,
        )])
        .unwrap();
        let model_set = DescriptorSet::new(model_set_layout, descriptor_pool.clone()).unwrap();
        let look_at_set = DescriptorSet::new(look_at_set_layout, descriptor_pool).unwrap();
        model_set.bind_buffer(model_buffer.clone(), 0).unwrap();
        look_at_set.bind_buffer(look_at_buffer.clone(), 0).unwrap();
        Ok(TransformData {
            model_matrix,
            model_buffer,
            model_set,
            look_at_matrix,
            look_at_buffer,
            look_at_set,
        })
    }
}

#[system]
fn add_transform_data(
    device: Resource<Arc<Device>>,
    descriptor_pool: Resource<Arc<DescriptorPool>>,
    mut transforms_data: Resource<IdVec<TransformData>>,
    mut transforms: Query<Mut<Transform>>,
) {
    for (_, transform) in transforms.into_iter() {
        if transform.data_id.is_none() {
            let transform_data = TransformData::new(
                device.clone(),
                descriptor_pool.clone(),
                transform.position,
                Vec3f::ZERO,
                Vec3f::ZERO,
            )
            .unwrap();

            let data_id = transforms_data.push(transform_data);
            transform.data_id = Some(data_id);
        }
    }
}
