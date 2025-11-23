use std::{marker::PhantomData, sync::Arc};

use crate::{
    graphics::{
        renderer::{DefaultVertex, assets::mesh::Mesh},
        vulkan::{
            VulkanError,
            buffer::{Buffer, BufferUsage},
            command_buffer::CommandBuffer,
            device::Device,
            graphics_pipeline::Vertex,
        },
    },
    resources::Resource,
    system,
    utils::id_vec::IdVec,
};

pub struct MeshBuffer<V: Vertex> {
    vertex_buffer: Arc<Buffer>,
    index_buffer: Arc<Buffer>,
    _marker: PhantomData<V>,
}

impl<V: Vertex> MeshBuffer<V> {
    pub fn new(
        device: Arc<Device>,
        vertices: &[V],
        indices: &[u32],
    ) -> Result<MeshBuffer<V>, VulkanError> {
        Ok(MeshBuffer {
            vertex_buffer: Buffer::from_vec(device.clone(), vertices, BufferUsage::VERTEX_BUFFER)?,
            index_buffer: Buffer::from_vec(device, indices, BufferUsage::INDEX_BUFFER)?,
            _marker: PhantomData::default(),
        })
    }

    pub fn record_draw_commands(&self, command_buffer: Arc<CommandBuffer>) {
        command_buffer.bind_vertex_buffer(self.vertex_buffer.clone());
        command_buffer.bind_index_buffer(self.index_buffer.clone());
        command_buffer.draw_indexed(self.index_buffer.len as u32, 1, 0, 0, 1);
    }
}

pub struct MeshData<V: Vertex> {
    pub buffer: MeshBuffer<V>,
}

impl<V: Vertex> MeshData<V> {
    pub fn new(
        device: Arc<Device>,
        vertices: &[V],
        indices: &[u32],
    ) -> Result<MeshData<V>, VulkanError> {
        let buffer = MeshBuffer::new(device, vertices, indices)?;
        Ok(MeshData { buffer })
    }

    pub fn record_commands(&self, command_buffer: Arc<CommandBuffer>) {
        self.buffer.record_draw_commands(command_buffer);
    }
}

#[system]
fn add_mesh_data(
    device: Resource<Arc<Device>>,
    mut meshes_data: Resource<IdVec<MeshData<DefaultVertex>>>,
    mut meshes: Resource<IdVec<Mesh>>,
) {
    for mesh in meshes.iter_mut() {
        if mesh.data_id.is_none() {
            let mesh_data = MeshData::new(device.clone(), &mesh.vertices, &mesh.indices).unwrap();
            let data_id = meshes_data.push(mesh_data);
            mesh.data_id = Some(data_id);
        }
    }
}
