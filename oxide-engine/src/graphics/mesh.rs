use std::sync::Arc;

use crate::graphics::vulkan::buffer::BufferUsage;

use super::vulkan::{
    VulkanError, buffer::Buffer, command_buffer::CommandBuffer, device::Device,
    graphics_pipeline::Vertex,
};

pub struct MeshBuffer<V: Vertex> {
    vertex_buffer: Arc<Buffer<V>>,
    index_buffer: Arc<Buffer<u32>>,
}

impl<V: Vertex> MeshBuffer<V> {
    pub fn new(
        device: Arc<Device>,
        vertices: Vec<V>,
        indices: Vec<u32>,
    ) -> Result<MeshBuffer<V>, VulkanError> {
        Ok(MeshBuffer {
            vertex_buffer: Buffer::from_vec(device.clone(), vertices, BufferUsage::VERTEX_BUFFER)?,
            index_buffer: Buffer::from_vec(device, indices, BufferUsage::INDEX_BUFFER)?,
        })
    }

    pub fn record_draw_commands(&self, command_buffer: Arc<CommandBuffer>) {
        command_buffer.bind_vertex_buffer(self.vertex_buffer.clone());
        command_buffer.bind_index_buffer(self.index_buffer.clone());
        command_buffer.draw_indexed(self.index_buffer.len as u32, 1, 0, 0, 1);
    }
}

pub struct MeshData<V: Vertex> {
    buffer: MeshBuffer<V>,
}

impl<V: Vertex> MeshData<V> {
    pub fn new(
        device: Arc<Device>,
        vertices: Vec<V>,
        indices: Vec<u32>,
    ) -> Result<MeshData<V>, VulkanError> {
        let buffer = MeshBuffer::new(device, vertices, indices)?;
        Ok(MeshData { buffer })
    }

    pub fn record_commands(&self, command_buffer: Arc<CommandBuffer>) {
        self.buffer.record_draw_commands(command_buffer);
    }
}
