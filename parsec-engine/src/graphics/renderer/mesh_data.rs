use std::marker::PhantomData;

use crate::{
    ecs::system::system,
    graphics::{
        ActiveGraphicsBackend,
        buffer::{Buffer, BufferBuilder, BufferContent, BufferUsage},
        command_list::{Command, CommandList},
        pipeline::Vertex,
        renderer::{DefaultVertex, assets::mesh::Mesh},
    },
    resources::Resource,
    utils::{
        IdType,
        identifiable::{IdStore, Identifiable},
    },
};

pub struct MeshBuffer<V: Vertex> {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    len: u32,
    _marker: PhantomData<V>,
}

impl<V: Vertex> MeshBuffer<V> {
    pub fn new(
        backend: &mut ActiveGraphicsBackend,
        vertices: &[V],
        indices: &[u32],
    ) -> MeshBuffer<V> {
        let vertex_buffer = BufferBuilder::new()
            .usage(&[BufferUsage::Vertex])
            .data(BufferContent::from_slice(vertices))
            .build(backend)
            .unwrap();
        let index_buffer = BufferBuilder::new()
            .usage(&[BufferUsage::Index])
            .data(BufferContent::from_slice(indices))
            .build(backend)
            .unwrap();

        MeshBuffer {
            vertex_buffer,
            index_buffer,
            len: indices.len() as u32,
            _marker: PhantomData::default(),
        }
    }

    pub fn record_draw_commands(&self, command_list: &mut CommandList) {
        command_list
            .cmd(Command::BindVertexBuffer(self.vertex_buffer.handle()));
        command_list.cmd(Command::BindIndexBuffer(self.index_buffer.handle()));
        command_list.cmd(Command::DrawIndexed(self.len, 1, 0, 0, 1));
    }
}

pub struct MeshData<V: Vertex> {
    mesh_data_id: IdType,
    pub buffer: MeshBuffer<V>,
}

crate::create_counter! {ID_COUNTER}
impl<V: Vertex> MeshData<V> {
    pub fn new(
        backend: &mut ActiveGraphicsBackend,
        vertices: &[V],
        indices: &[u32],
    ) -> MeshData<V> {
        let buffer = MeshBuffer::new(backend, vertices, indices);
        MeshData {
            mesh_data_id: ID_COUNTER.next(),
            buffer,
        }
    }

    pub fn record_commands(&self, command_list: &mut CommandList) {
        self.buffer.record_draw_commands(command_list);
    }
}

impl<V: Vertex> Identifiable for MeshData<V> {
    fn id(&self) -> IdType { self.mesh_data_id }
}

#[system]
fn add_mesh_data(
    mut backend: Resource<ActiveGraphicsBackend>,
    mut meshes_data: Resource<IdStore<MeshData<DefaultVertex>>>,
    mut meshes: Resource<IdStore<Mesh>>,
) {
    for mesh in meshes.iter_mut() {
        if mesh.data_id.is_none() {
            let mesh_data =
                MeshData::new(&mut *backend, &mesh.vertices, &mesh.indices);
            let data_id = meshes_data.push(mesh_data);
            mesh.data_id = Some(data_id);
        }
    }
}
