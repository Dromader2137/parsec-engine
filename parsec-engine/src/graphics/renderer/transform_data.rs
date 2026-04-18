use std::collections::HashMap;

use crate::{
    ecs::{
        system::system,
        world::{fetch::Mut, query::Query},
    },
    graphics::{
        ActiveGraphicsBackend,
        buffer::{Buffer, BufferBuilder, BufferContent, BufferUsage},
        pipeline::{
            PipelineBindingType, PipelineResource,
            PipelineResourceBindingLayout, PipelineResourceHandle,
            PipelineResourceLayout, PipelineResourceLayoutBuilder,
            PipelineShaderStage,
        },
        renderer::components::transform::Transform,
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
    model_layout: PipelineResourceLayout,
    pub model_resource: PipelineResource,
    pub look_at_matrix: Matrix4f,
    pub look_at_buffer: Buffer,
    look_at_layout: PipelineResourceLayout,
    pub look_at_resource: PipelineResource,
}

pub struct TransformDataManager {
    pub component_to_data: HashMap<u32, u32>,
}

crate::create_counter! {ID_COUNTER}
impl TransformData {
    pub fn new(
        backend: &mut ActiveGraphicsBackend,
        position: Vec3f,
        scale: Vec3f,
        rotation: Quat,
    ) -> TransformData {
        let translation_matrix = Matrix4f::translation(position);
        let translation_buffer = BufferBuilder::new()
            .usage(&[BufferUsage::Uniform])
            .data(BufferContent::from_slice(&[translation_matrix]))
            .build(backend)
            .unwrap();
        let scale_matrix = Matrix4f::scale(scale);
        let scale_buffer = BufferBuilder::new()
            .usage(&[BufferUsage::Uniform])
            .data(BufferContent::from_slice(&[scale_matrix]))
            .build(backend)
            .unwrap();
        let rotation_matrix = rotation.into_matrix();
        let rotation_buffer = BufferBuilder::new()
            .usage(&[BufferUsage::Uniform])
            .data(BufferContent::from_slice(&[scale_matrix]))
            .build(backend)
            .unwrap();
        let look_at_matrix = Matrix4f::look_at(
            position,
            Vec3f::FORWARD * rotation,
            Vec3f::UP * rotation,
        );
        let look_at_buffer = BufferBuilder::new()
            .usage(&[BufferUsage::Uniform])
            .data(BufferContent::from_slice(&[scale_matrix]))
            .build(backend)
            .unwrap();
        let mut model_layout = PipelineResourceLayoutBuilder::new()
            .bindings(&[
                PipelineResourceBindingLayout {
                    binding_type: PipelineBindingType::UniformBuffer,
                    shader_stages: vec![PipelineShaderStage::Vertex],
                },
                PipelineResourceBindingLayout {
                    binding_type: PipelineBindingType::UniformBuffer,
                    shader_stages: vec![PipelineShaderStage::Vertex],
                },
                PipelineResourceBindingLayout {
                    binding_type: PipelineBindingType::UniformBuffer,
                    shader_stages: vec![PipelineShaderStage::Vertex],
                },
            ])
            .build(backend)
            .unwrap();
        let mut look_at_layout = PipelineResourceLayoutBuilder::new()
            .bindings(&[PipelineResourceBindingLayout {
                binding_type: PipelineBindingType::UniformBuffer,
                shader_stages: vec![PipelineShaderStage::Vertex],
            }])
            .build(backend)
            .unwrap();
        let model_resource = model_layout.create_resource(backend).unwrap();
        let look_at_resource = look_at_layout.create_resource(backend).unwrap();
        model_resource
            .bind_buffer(backend, translation_buffer.handle(), 0)
            .unwrap();
        model_resource
            .bind_buffer(backend, scale_buffer.handle(), 1)
            .unwrap();
        model_resource
            .bind_buffer(backend, rotation_buffer.handle(), 2)
            .unwrap();
        look_at_resource
            .bind_buffer(backend, look_at_buffer.handle(), 0)
            .unwrap();
        TransformData {
            transform_data_id: ID_COUNTER.next(),
            translation_matrix,
            scale_matrix,
            rotation_matrix,
            translation_buffer,
            scale_buffer,
            rotation_buffer,
            model_layout,
            model_resource,
            look_at_matrix,
            look_at_buffer,
            look_at_layout,
            look_at_resource,
        }
    }

    fn update_buffers_from_data(
        &mut self,
        backend: &mut ActiveGraphicsBackend,
    ) {
        backend
            .update_buffer(
                self.translation_buffer.handle(),
                BufferContent::from_slice(&[self.translation_matrix]),
            )
            .unwrap();
        backend
            .update_buffer(
                self.scale_buffer.handle(),
                BufferContent::from_slice(&[self.scale_matrix]),
            )
            .unwrap();
        backend
            .update_buffer(
                self.rotation_buffer.handle(),
                BufferContent::from_slice(&[self.rotation_matrix]),
            )
            .unwrap();
        backend
            .update_buffer(
                self.look_at_buffer.handle(),
                BufferContent::from_slice(&[self.look_at_matrix]),
            )
            .unwrap();
    }
}

impl Identifiable for TransformData {
    fn id(&self) -> IdType { self.transform_data_id }
}

#[system]
fn add_transform_data(
    mut backend: Resource<ActiveGraphicsBackend>,
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
                &mut *backend,
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
    mut backend: Resource<ActiveGraphicsBackend>,
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
            data.update_buffers_from_data(&mut *backend);
        }
    }
}
