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
                DescriptorPool, DescriptorSet, DescriptorSetBinding,
                DescriptorSetLayout, DescriptorStage, DescriptorType,
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
    pub translation_buffer: Buffer,
    pub scale_buffer: Buffer,
    pub rotation_buffer: Buffer,
    pub model_set: DescriptorSet,
    pub look_at_matrix: Matrix4f,
    pub look_at_buffer: Buffer,
    pub look_at_set: DescriptorSet,
}

impl TransformData {
    pub fn new(
        device: &Device,
        descriptor_pool: &DescriptorPool,
        position: Vec3f,
        scale: Vec3f,
        rotation: Quat,
    ) -> Result<TransformData, VulkanError> {
        let translation_matrix = Matrix4f::translation(position);
        let translation_buffer = Buffer::from_vec(
            device,
            &[translation_matrix],
            BufferUsage::UNIFORM_BUFFER,
        )
        .unwrap();
        let scale_matrix = Matrix4f::scale(scale);
        let scale_buffer = Buffer::from_vec(
            device,
            &[scale_matrix],
            BufferUsage::UNIFORM_BUFFER,
        )
        .unwrap();
        let rotation_matrix = rotation.into_matrix();
        let rotation_buffer = Buffer::from_vec(
            device,
            &[rotation_matrix],
            BufferUsage::UNIFORM_BUFFER,
        )
        .unwrap();
        let look_at_matrix = Matrix4f::look_at(
            position,
            Vec3f::FORWARD * rotation,
            Vec3f::UP * rotation,
        );
        let look_at_buffer = Buffer::from_vec(
            device,
            &[look_at_matrix],
            BufferUsage::UNIFORM_BUFFER,
        )
        .unwrap();
        let model_set_layout = DescriptorSetLayout::new(device, vec![
            DescriptorSetBinding::new(
                0,
                DescriptorType::UNIFORM_BUFFER,
                DescriptorStage::VERTEX,
            ),
            DescriptorSetBinding::new(
                1,
                DescriptorType::UNIFORM_BUFFER,
                DescriptorStage::VERTEX,
            ),
            DescriptorSetBinding::new(
                2,
                DescriptorType::UNIFORM_BUFFER,
                DescriptorStage::VERTEX,
            ),
        ])
        .unwrap();
        let look_at_set_layout =
            DescriptorSetLayout::new(device, vec![DescriptorSetBinding::new(
                0,
                DescriptorType::UNIFORM_BUFFER,
                DescriptorStage::VERTEX,
            )])
            .unwrap();
        let model_set =
            DescriptorSet::new(device, &model_set_layout, descriptor_pool)
                .unwrap();
        let look_at_set =
            DescriptorSet::new(device, &look_at_set_layout, descriptor_pool)
                .unwrap();
        model_set
            .bind_buffer(&translation_buffer, device, &model_set_layout, 0)
            .unwrap();
        model_set
            .bind_buffer(&scale_buffer, device, &model_set_layout, 1)
            .unwrap();
        model_set
            .bind_buffer(&rotation_buffer, device, &model_set_layout, 2)
            .unwrap();
        look_at_set
            .bind_buffer(&look_at_buffer, device, &model_set_layout, 0)
            .unwrap();
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

    fn update_buffers_from_data(
        &mut self,
        device: &Device,
    ) -> Result<(), VulkanError> {
        self.translation_buffer
            .update(device, &[self.translation_matrix])?;
        self.scale_buffer.update(device, &[self.scale_matrix])?;
        self.rotation_buffer
            .update(device, &[self.rotation_matrix])?;
        self.look_at_buffer.update(device, &[self.look_at_matrix])?;
        Ok(())
    }
}

#[system]
fn add_transform_data(
    device: Resource<Device>,
    descriptor_pool: Resource<DescriptorPool>,
    mut transforms_data: Resource<IdVec<TransformData>>,
    mut transforms: Query<Mut<Transform>>,
) {
    for (_, transform) in transforms.iter() {
        if transform.data_id.is_none() {
            let transform_data = TransformData::new(
                &device,
                &descriptor_pool,
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
    device: Resource<Device>,
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
        data.update_buffers_from_data(&device).unwrap();
    }
}
