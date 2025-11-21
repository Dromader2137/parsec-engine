use std::{marker::PhantomData, sync::Arc};

use crate::{
    graphics::{
        renderer::DefaultVertex,
        vulkan::{
            VulkanError,
            buffer::{Buffer, BufferUsage},
            command_buffer::CommandBuffer,
            device::Device,
            graphics_pipeline::Vertex,
        },
    },
    resources::{Rsc, RscMut},
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

pub fn create_mesh_data(vertices: &[DefaultVertex], indices: &[u32]) -> Result<u32, VulkanError> {
    let device = Rsc::<Arc<Device>>::get().unwrap();
    let mut meshes = RscMut::<IdVec<MeshData<DefaultVertex>>>::get().unwrap();
    Ok(meshes.push(MeshData::new(device.clone(), vertices, indices)?))
}
