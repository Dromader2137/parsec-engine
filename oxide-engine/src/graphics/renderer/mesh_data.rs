use std::{marker::PhantomData, sync::Arc};

use crate::{
    ecs::system::system,
    graphics::{
        renderer::{DefaultVertex, assets::mesh::Mesh},
        vulkan::{
            VulkanError,
            buffer::{Buffer, BufferUsage},
            command_buffer::CommandBuffer,
            device::Device,
            graphics_pipeline::Vertex,
            physical_device::PhysicalDevice,
        },
    },
    resources::Resource,
    utils::id_vec::IdVec,
};

pub struct MeshBuffer<V: Vertex> {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    _marker: PhantomData<V>,
}

impl<V: Vertex> MeshBuffer<V> {
    pub fn new(
        physical_device: &PhysicalDevice,
        device: &Device,
        vertices: &[V],
        indices: &[u32],
    ) -> Result<MeshBuffer<V>, VulkanError> {
        Ok(MeshBuffer {
            vertex_buffer: Buffer::from_vec(
                physical_device,
                device,
                vertices,
                BufferUsage::VERTEX_BUFFER,
            )?,
            index_buffer: Buffer::from_vec(
                physical_device,
                device,
                indices,
                BufferUsage::INDEX_BUFFER,
            )?,
            _marker: PhantomData::default(),
        })
    }

    pub fn record_draw_commands(
        &self,
        device: &Device,
        command_buffer: &CommandBuffer,
    ) {
        command_buffer.bind_vertex_buffer(device, &self.vertex_buffer);
        command_buffer.bind_index_buffer(device, &self.index_buffer);
        command_buffer.draw_indexed(
            device,
            self.index_buffer.len as u32,
            1,
            0,
            0,
            1,
        );
    }
}

pub struct MeshData<V: Vertex> {
    pub buffer: MeshBuffer<V>,
}

impl<V: Vertex> MeshData<V> {
    pub fn new(
        physical_device: &PhysicalDevice,
        device: &Device,
        vertices: &[V],
        indices: &[u32],
    ) -> Result<MeshData<V>, VulkanError> {
        let buffer =
            MeshBuffer::new(physical_device, device, vertices, indices)?;
        Ok(MeshData { buffer })
    }

    pub fn record_commands(
        &self,
        device: &Device,
        command_buffer: &CommandBuffer,
    ) {
        self.buffer.record_draw_commands(device, command_buffer);
    }
}

#[system]
fn add_mesh_data(
    physical_device: Resource<PhysicalDevice>,
    device: Resource<Device>,
    mut meshes_data: Resource<IdVec<MeshData<DefaultVertex>>>,
    mut meshes: Resource<IdVec<Mesh>>,
) {
    for mesh in meshes.iter_mut() {
        if mesh.data_id.is_none() {
            let mesh_data = MeshData::new(
                &physical_device,
                &device,
                &mesh.vertices,
                &mesh.indices,
            )
            .unwrap();
            let data_id = meshes_data.push(mesh_data);
            mesh.data_id = Some(data_id);
        }
    }
}
