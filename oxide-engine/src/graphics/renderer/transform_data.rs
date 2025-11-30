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
    math::{mat::Matrix4f, quat::Quat, vec::Vec3f},
    resources::Resource,
    utils::id_vec::IdVec,
};

pub struct TransformData {
    pub translation_matrix: Matrix4f,
    pub scale_matrix: Matrix4f,
    pub rotation_matrix: Matrix4f,
    pub translation_buffer: Arc<Buffer>,
    pub scale_buffer: Arc<Buffer>,
    pub rotation_buffer: Arc<Buffer>,
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
        rotation: Quat,
    ) -> Result<TransformData, VulkanError> {
        let translation_matrix = Matrix4f::translation(position);
        let translation_buffer = Buffer::from_vec(
            device.clone(),
            &[translation_matrix],
            BufferUsage::UNIFORM_BUFFER,
        )
        .unwrap();
        let scale_matrix = Matrix4f::scale(scale);
        let scale_buffer =
            Buffer::from_vec(device.clone(), &[scale_matrix], BufferUsage::UNIFORM_BUFFER).unwrap();
        let rotation_matrix = rotation.into_matrix();
        let rotation_buffer = Buffer::from_vec(
            device.clone(),
            &[rotation_matrix],
            BufferUsage::UNIFORM_BUFFER,
        )
        .unwrap();
        let look_at_matrix =
            Matrix4f::look_at(position, Vec3f::FORWARD * rotation, Vec3f::UP * rotation);
        let look_at_buffer = Buffer::from_vec(
            device.clone(),
            &[look_at_matrix],
            BufferUsage::UNIFORM_BUFFER,
        )
        .unwrap();
        let model_set_layout = DescriptorSetLayout::new(device.clone(), vec![
            DescriptorSetBinding::new(0, DescriptorType::UNIFORM_BUFFER, DescriptorStage::VERTEX),
            DescriptorSetBinding::new(1, DescriptorType::UNIFORM_BUFFER, DescriptorStage::VERTEX),
            DescriptorSetBinding::new(2, DescriptorType::UNIFORM_BUFFER, DescriptorStage::VERTEX),
        ])
        .unwrap();
        let look_at_set_layout = DescriptorSetLayout::new(device, vec![DescriptorSetBinding::new(
            0,
            DescriptorType::UNIFORM_BUFFER,
            DescriptorStage::VERTEX,
        )])
        .unwrap();
        let model_set = DescriptorSet::new(model_set_layout, descriptor_pool.clone()).unwrap();
        let look_at_set = DescriptorSet::new(look_at_set_layout, descriptor_pool).unwrap();
        model_set
            .bind_buffer(translation_buffer.clone(), 0)
            .unwrap();
        model_set.bind_buffer(scale_buffer.clone(), 1).unwrap();
        model_set.bind_buffer(rotation_buffer.clone(), 2).unwrap();
        look_at_set.bind_buffer(look_at_buffer.clone(), 0).unwrap();
        Ok(TransformData {
            translation_matrix,
            scale_matrix,
            rotation_matrix,
            translation_buffer,
            scale_buffer,
            rotation_buffer,
            model_set,
            look_at_matrix,
            look_at_buffer,
            look_at_set,
        })
    }

    fn update_buffers_from_data(&mut self) -> Result<(), VulkanError> {
        self.translation_buffer
            .update(vec![self.translation_matrix])?;
        self.scale_buffer.update(vec![self.scale_matrix])?;
        self.rotation_buffer.update(vec![self.rotation_matrix])?;
        self.look_at_buffer.update(vec![self.look_at_matrix])?;
        Ok(())
    }
}

#[system]
fn add_transform_data(
    device: Resource<Arc<Device>>,
    descriptor_pool: Resource<Arc<DescriptorPool>>,
    mut transforms_data: Resource<IdVec<TransformData>>,
    mut transforms: Query<Mut<Transform>>,
) {
    for (_, transform) in transforms.iter() {
        if transform.data_id.is_none() {
            let transform_data = TransformData::new(
                device.clone(),
                descriptor_pool.clone(),
                transform.position,
                transform.scale,
                transform.rotation,
            )
            .unwrap();

            let data_id = transforms_data.push(transform_data);
            transform.data_id = Some(data_id);
        }
    }
}

#[system]
fn update_transform_data(
    mut transforms_data: Resource<IdVec<TransformData>>,
    mut transforms: Query<Transform>,
) {
    for (_, transform) in transforms.iter() {
        if transform.data_id.is_none() {
            continue;
        }
        let data = transforms_data.get_mut(transform.data_id.unwrap()).unwrap();
        data.translation_matrix = Matrix4f::translation(transform.position);
        data.scale_matrix = Matrix4f::scale(transform.scale);
        data.rotation_matrix = transform.rotation.into_matrix();
        data.look_at_matrix = Matrix4f::look_at(
            transform.position,
            Vec3f::FORWARD * transform.rotation,
            Vec3f::UP * transform.rotation,
        );
        data.update_buffers_from_data().unwrap();
    }
}
