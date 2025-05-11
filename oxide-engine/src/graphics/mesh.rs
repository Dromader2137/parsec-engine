use crate::graphics::vulkan::buffer::BufferUsage;

use super::vulkan::{
    VulkanError, buffer::Buffer, command_buffer::CommandBuffer, device::Device,
    graphics_pipeline::Vertex, instance::Instance, physical_device::PhysicalDevice,
};

pub struct MeshBuffer<V: Vertex> {
    vertex_buffer: Buffer<V>,
    index_buffer: Buffer<u32>,
}

impl<V: Vertex> MeshBuffer<V> {
    pub fn new(
        instance: &Instance,
        physical_device: &PhysicalDevice,
        device: &Device,
        vertices: Vec<V>,
        indices: Vec<u32>,
    ) -> Result<MeshBuffer<V>, VulkanError> {
        Ok(MeshBuffer {
            vertex_buffer: Buffer::from_vec(
                instance,
                physical_device,
                device,
                vertices,
                BufferUsage::VERTEX_BUFFER,
            )?,
            index_buffer: Buffer::from_vec(
                instance,
                physical_device,
                device,
                indices,
                BufferUsage::INDEX_BUFFER,
            )?,
        })
    }

    pub fn record_draw_commands(&self, device: &Device, command_buffer: &CommandBuffer) {
        command_buffer.bind_vertex_buffer(device, &self.vertex_buffer);
        command_buffer.bind_index_buffer(device, &self.index_buffer);
        command_buffer.draw_indexed(device, self.index_buffer.len as u32, 1, 0, 0, 1);
    }

    pub fn cleanup(&self, device: &Device) {
        self.vertex_buffer.cleanup(device);
        self.index_buffer.cleanup(device);
    }
}

pub struct MeshData<V: Vertex> {
    buffer: MeshBuffer<V>,
}

impl<V: Vertex> MeshData<V> {
    pub fn new(
        instance: &Instance,
        physical_device: &PhysicalDevice,
        device: &Device,
        vertices: Vec<V>,
        indices: Vec<u32>,
    ) -> Result<MeshData<V>, VulkanError> {
        let buffer = MeshBuffer {
            vertex_buffer: Buffer::from_vec(
                instance,
                physical_device,
                device,
                vertices,
                BufferUsage::VERTEX_BUFFER,
            )?,
            index_buffer: Buffer::from_vec(
                instance,
                physical_device,
                device,
                indices,
                BufferUsage::INDEX_BUFFER,
            )?,
        };

        Ok(MeshData { buffer })
    }

    pub fn record_commands(&self, device: &Device, command_buffer: &CommandBuffer) {
        self.buffer.record_draw_commands(device, command_buffer);
    }

    pub fn cleanup(&self, device: &Device) {
        self.buffer.cleanup(device);
    }
}
