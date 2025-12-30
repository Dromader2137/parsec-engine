use std::{collections::HashMap, ops::DerefMut};

use crate::{
    ecs::{
        system::system,
        world::{fetch::Mut, query::Query},
    },
    graphics::{
        backend::GraphicsBackend,
        buffer::{Buffer, BufferUsage},
        pipeline::{
            PipelineBinding, PipelineBindingType, PipelineShaderStage,
            PipelineSubbindingLayout,
        },
        renderer::components::transform::Transform,
        vulkan::VulkanBackend,
    },
    math::{mat::Matrix4f, quat::Quat, vec::Vec3f},
    resources::Resource,
    utils::{
        IdType,
        identifiable::{IdStore, Identifiable},
    },
};

pub struct TransformData {
    transform_data_id: IdType,
    pub translation_matrix: Matrix4f,
    pub scale_matrix: Matrix4f,
    pub rotation_matrix: Matrix4f,
    pub translation_buffer: Buffer,
    pub scale_buffer: Buffer,
    pub rotation_buffer: Buffer,
    pub model_binding: PipelineBinding,
    pub look_at_matrix: Matrix4f,
    pub look_at_buffer: Buffer,
    pub look_at_binding: PipelineBinding,
}

pub struct TransformDataManager {
    pub component_to_data: HashMap<u32, u32>,
}

crate::create_counter! {ID_COUNTER}
impl TransformData {
    pub fn new(
        backend: &mut impl GraphicsBackend,
        position: Vec3f,
        scale: Vec3f,
        rotation: Quat,
    ) -> TransformData {
        let translation_matrix = Matrix4f::translation(position);
        let translation_buffer = backend
            .create_buffer(&[translation_matrix], &[BufferUsage::Uniform])
            .unwrap();
        let scale_matrix = Matrix4f::scale(scale);
        let scale_buffer = backend
            .create_buffer(&[scale_matrix], &[BufferUsage::Uniform])
            .unwrap();
        let rotation_matrix = rotation.into_matrix();
        let rotation_buffer = backend
            .create_buffer(&[scale_matrix], &[BufferUsage::Uniform])
            .unwrap();
        let look_at_matrix = Matrix4f::look_at(
            position,
            Vec3f::FORWARD * rotation,
            Vec3f::UP * rotation,
        );
        let look_at_buffer = backend
            .create_buffer(&[scale_matrix], &[BufferUsage::Uniform])
            .unwrap();
        let model_pipeline_layout = backend
            .create_pipeline_binding_layout(&[
                PipelineSubbindingLayout {
                    binding_type: PipelineBindingType::UniformBuffer,
                    shader_stage: PipelineShaderStage::Vertex,
                },
                PipelineSubbindingLayout {
                    binding_type: PipelineBindingType::UniformBuffer,
                    shader_stage: PipelineShaderStage::Vertex,
                },
                PipelineSubbindingLayout {
                    binding_type: PipelineBindingType::UniformBuffer,
                    shader_stage: PipelineShaderStage::Vertex,
                },
            ])
            .unwrap();
        let look_at_pipeline_layout = backend
            .create_pipeline_binding_layout(&[PipelineSubbindingLayout {
                binding_type: PipelineBindingType::UniformBuffer,
                shader_stage: PipelineShaderStage::Vertex,
            }])
            .unwrap();
        let model_binding = backend
            .create_pipeline_binding(model_pipeline_layout)
            .unwrap();
        let look_at_binding = backend
            .create_pipeline_binding(look_at_pipeline_layout)
            .unwrap();
        backend
            .bind_buffer(model_binding, translation_buffer, 0)
            .unwrap();
        backend.bind_buffer(model_binding, scale_buffer, 1).unwrap();
        backend
            .bind_buffer(model_binding, rotation_buffer, 2)
            .unwrap();
        backend
            .bind_buffer(look_at_binding, look_at_buffer, 0)
            .unwrap();
        TransformData {
            transform_data_id: ID_COUNTER.next(),
            translation_matrix,
            scale_matrix,
            rotation_matrix,
            translation_buffer,
            scale_buffer,
            rotation_buffer,
            model_binding,
            look_at_matrix,
            look_at_buffer,
            look_at_binding,
        }
    }

    fn update_buffers_from_data(&mut self, backend: &mut impl GraphicsBackend) {
        backend
            .update_buffer(self.translation_buffer, &[self.translation_matrix])
            .unwrap();
        backend
            .update_buffer(self.scale_buffer, &[self.scale_matrix])
            .unwrap();
        backend
            .update_buffer(self.rotation_buffer, &[self.rotation_matrix])
            .unwrap();
        backend
            .update_buffer(self.look_at_buffer, &[self.look_at_matrix])
            .unwrap();
    }
}

impl Identifiable for TransformData {
    fn id(&self) -> IdType { self.transform_data_id }
}

#[system]
fn add_transform_data(
    mut backend: Resource<VulkanBackend>,
    mut transforms_data: Resource<IdStore<TransformData>>,
    mut transforms_data_manager: Resource<TransformDataManager>,
    mut transforms: Query<Mut<Transform>>,
) {
    for (_, transform) in transforms.iter() {
        if !transforms_data_manager
            .component_to_data
            .contains_key(&transform.transform_id())
        {
            let transform_data = TransformData::new(
                backend.deref_mut(),
                transform.position,
                transform.scale,
                transform.rotation,
            );
            let data_id = transforms_data.push(transform_data);
            transforms_data_manager
                .component_to_data
                .insert(transform.transform_id(), data_id);
        }
    }
}

#[system]
fn update_transform_data(
    mut backend: Resource<VulkanBackend>,
    mut transforms_data: Resource<IdStore<TransformData>>,
    transforms_data_manager: Resource<TransformDataManager>,
    mut transforms: Query<Transform>,
) {
    for (_, transform) in transforms.iter() {
        if let Some(data_id) = transforms_data_manager
            .component_to_data
            .get(&transform.transform_id())
        {
            let data = transforms_data.get_mut(*data_id).unwrap();
            data.translation_matrix = Matrix4f::translation(transform.position);
            data.scale_matrix = Matrix4f::scale(transform.scale);
            data.rotation_matrix = transform.rotation.into_matrix();
            data.look_at_matrix = Matrix4f::look_at(
                transform.position,
                Vec3f::FORWARD * transform.rotation,
                Vec3f::UP * transform.rotation,
            );
            data.update_buffers_from_data(backend.deref_mut());
        }
    }
}
