use std::{marker::PhantomData, ops::DerefMut};

use crate::{
    ecs::system::system,
    graphics::{
        backend::GraphicsBackend,
        buffer::{Buffer, BufferUsage},
        command_list::CommandList,
        renderer::{DefaultVertex, assets::mesh::Mesh},
        vulkan::VulkanBackend,
    },
    resources::Resource,
    utils::{
        IdType,
        identifiable::{IdStore, Identifiable},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VertexFieldFormat {
    Float,
    Vec2,
    Vec3,
    Vec4,
}

pub struct VertexField {
    pub format: VertexFieldFormat,
}

pub trait Vertex: Clone + Copy {
    fn fields() -> Vec<VertexField>;
}

pub struct MeshBuffer<V: Vertex> {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    _marker: PhantomData<V>,
}

impl<V: Vertex> MeshBuffer<V> {
    pub fn new(
        backend: &mut impl GraphicsBackend,
        vertices: &[V],
        indices: &[u32],
    ) -> MeshBuffer<V> {
        MeshBuffer {
            vertex_buffer: backend
                .create_buffer(vertices, &[BufferUsage::Vertex])
                .unwrap(),
            index_buffer: backend
                .create_buffer(indices, &[BufferUsage::Index])
                .unwrap(),
            _marker: PhantomData::default(),
        }
    }

    pub fn record_draw_commands(
        &self,
        backend: &mut impl GraphicsBackend,
        command_list: CommandList,
    ) {
        backend
            .command_draw(command_list, self.vertex_buffer, self.index_buffer)
            .unwrap();
    }
}

pub struct MeshData<V: Vertex> {
    mesh_data_id: IdType,
    pub buffer: MeshBuffer<V>,
}

crate::create_counter! {ID_COUNTER}
impl<V: Vertex> MeshData<V> {
    pub fn new(
        backend: &mut impl GraphicsBackend,
        vertices: &[V],
        indices: &[u32],
    ) -> MeshData<V> {
        let buffer = MeshBuffer::new(backend, vertices, indices);
        MeshData {
            mesh_data_id: ID_COUNTER.next(),
            buffer,
        }
    }

    pub fn record_commands(
        &self,
        backend: &mut impl GraphicsBackend,
        command_list: CommandList,
    ) {
        self.buffer.record_draw_commands(backend, command_list);
    }
}

impl<V: Vertex> Identifiable for MeshData<V> {
    fn id(&self) -> IdType { self.mesh_data_id }
}

#[system]
fn add_mesh_data(
    mut backend: Resource<VulkanBackend>,
    mut meshes_data: Resource<IdStore<MeshData<DefaultVertex>>>,
    mut meshes: Resource<IdStore<Mesh>>,
) {
    for mesh in meshes.iter_mut() {
        if mesh.data_id.is_none() {
            let mesh_data = MeshData::new(
                backend.deref_mut(),
                &mesh.vertices,
                &mesh.indices,
            );
            let data_id = meshes_data.push(mesh_data);
            mesh.data_id = Some(data_id);
        }
    }
}
