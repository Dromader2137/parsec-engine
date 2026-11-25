use std::collections::HashMap;

use crate::{
    create_counter,
    ctx::Ctx,
    ecs::world::fetch::Mut,
    error::SystemError,
    graphics::{
        ActiveGraphicsBackend,
        buffer::{Buffer, BufferBuilder, BufferContent, BufferUsage},
        pipeline::{
            PipelineBindingType, PipelineResource, PipelineResourceBindingLayout,
            PipelineResourceLayout, PipelineResourceLayoutBuilder, PipelineShaderStage,
        },
    },
    math::{mat::Matrix4f, quat::Quat, vec::Vec3f},
    renderer::{RendererError, components::transform::Transform, get_resource, get_resource_mut},
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

create_counter! {ID_COUNTER}
impl TransformData {
    pub fn new(
        backend: &mut ActiveGraphicsBackend,
        position: Vec3f,
        scale: Vec3f,
        rotation: Quat,
    ) -> Result<TransformData, RendererError> {
        let translation_matrix = Matrix4f::translation(position);
        let translation_buffer = BufferBuilder::new()
            .usage(&[BufferUsage::Uniform])
            .data(BufferContent::from_slice(&[translation_matrix]))
            .build(backend)?;
        let scale_matrix = Matrix4f::scale(scale);
        let scale_buffer = BufferBuilder::new()
            .usage(&[BufferUsage::Uniform])
            .data(BufferContent::from_slice(&[scale_matrix]))
            .build(backend)?;
        let rotation_matrix = rotation.into_matrix();
        let rotation_buffer = BufferBuilder::new()
            .usage(&[BufferUsage::Uniform])
            .data(BufferContent::from_slice(&[scale_matrix]))
            .build(backend)?;
        let look_at_matrix =
            Matrix4f::look_at(position, Vec3f::FORWARD * rotation, Vec3f::UP * rotation);
        let look_at_buffer = BufferBuilder::new()
            .usage(&[BufferUsage::Uniform])
            .data(BufferContent::from_slice(&[scale_matrix]))
            .build(backend)?;
        let model_layout = PipelineResourceLayoutBuilder::new()
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
            .build(backend)?;
        let look_at_layout = PipelineResourceLayoutBuilder::new()
            .bindings(&[PipelineResourceBindingLayout {
                binding_type: PipelineBindingType::UniformBuffer,
                shader_stages: vec![PipelineShaderStage::Vertex],
            }])
            .build(backend)?;
        let model_resource = model_layout.create_resource(backend)?;
        let look_at_resource = look_at_layout.create_resource(backend)?;
        model_resource.bind_buffer(backend, translation_buffer.handle(), 0)?;
        model_resource.bind_buffer(backend, scale_buffer.handle(), 1)?;
        model_resource.bind_buffer(backend, rotation_buffer.handle(), 2)?;
        look_at_resource.bind_buffer(backend, look_at_buffer.handle(), 0)?;
        Ok(TransformData {
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
        })
    }

    fn update_buffers_from_data(
        &mut self,
        backend: &mut ActiveGraphicsBackend,
    ) -> Result<(), RendererError> {
        backend.update_buffer(
            self.translation_buffer.handle(),
            BufferContent::from_slice(&[self.translation_matrix]),
        )?;
        backend.update_buffer(
            self.scale_buffer.handle(),
            BufferContent::from_slice(&[self.scale_matrix]),
        )?;
        backend.update_buffer(
            self.rotation_buffer.handle(),
            BufferContent::from_slice(&[self.rotation_matrix]),
        )?;
        backend.update_buffer(
            self.look_at_buffer.handle(),
            BufferContent::from_slice(&[self.look_at_matrix]),
        )?;
        Ok(())
    }

    pub fn destroy(self, backend: &mut ActiveGraphicsBackend) -> Result<(), RendererError> {
        self.translation_buffer.destroy(backend)?;
        self.scale_buffer.destroy(backend)?;
        self.rotation_buffer.destroy(backend)?;
        self.look_at_buffer.destroy(backend)?;
        self.model_resource.destroy(backend)?;
        self.look_at_resource.destroy(backend)?;
        self.model_layout.destroy(backend)?;
        self.look_at_layout.destroy(backend)?;
        Ok(())
    }
}

impl Identifiable for TransformData {
    fn id(&self) -> IdType { self.transform_data_id }
}

pub fn add_transform_data(ctx: Ctx) -> Result<(), SystemError> {
    let mut backend = get_resource_mut::<ActiveGraphicsBackend>(ctx.resources)?;
    let mut transforms_data = get_resource_mut::<IdStore<TransformData>>(ctx.resources)?;
    let mut transforms_data_manager = get_resource_mut::<TransformDataManager>(ctx.resources)?;
    let mut transforms = ctx.world.query::<Mut<Transform>>().map_err(RendererError::from)?;

    for (_, transform) in transforms.iter() {
        if let std::collections::hash_map::Entry::Vacant(e) =
            transforms_data_manager.component_to_data.entry(transform.transform_id())
        {
            let transform_data = TransformData::new(
                &mut backend,
                transform.position,
                transform.scale,
                transform.rotation,
            )?;
            let data_id = transforms_data.push(transform_data);
            e.insert(data_id);
        }
    }
    Ok(())
}

pub fn update_transform_data(ctx: Ctx) -> Result<(), SystemError> {
    let mut backend = get_resource_mut::<ActiveGraphicsBackend>(ctx.resources)?;
    let mut transforms_data = get_resource_mut::<IdStore<TransformData>>(ctx.resources)?;
    let transforms_data_manager = get_resource::<TransformDataManager>(ctx.resources)?;
    let mut transforms = ctx.world.query::<Transform>().map_err(RendererError::from)?;

    for (_, transform) in transforms.iter() {
        if let Some(data_id) =
            transforms_data_manager.component_to_data.get(&transform.transform_id())
        {
            let data = transforms_data
                .get_mut(*data_id)
                .ok_or(RendererError::TransformDataNotFound(*data_id))?;
            data.translation_matrix = Matrix4f::translation(transform.position);
            data.scale_matrix = Matrix4f::scale(transform.scale);
            data.rotation_matrix = transform.rotation.into_matrix();
            data.look_at_matrix = Matrix4f::look_at(
                transform.position,
                Vec3f::FORWARD * transform.rotation,
                Vec3f::UP * transform.rotation,
            );
            data.update_buffers_from_data(&mut backend)?;
        }
    }
    Ok(())
}
