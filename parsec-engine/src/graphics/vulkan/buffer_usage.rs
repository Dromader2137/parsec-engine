use crate::graphics::buffer::BufferUsage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(unused)]
pub enum VulkanBufferUsage {
    TransferSrc,
    TransferDst,
    UniformBuffer,
    StorageBuffer,
    IndexBuffer,
    VertexBuffer,
    IndirectBuffer,
}

impl VulkanBufferUsage {
    pub fn new(value: BufferUsage) -> Self {
        match value {
            BufferUsage::TransferSrc => Self::TransferSrc,
            BufferUsage::TransferDst => Self::TransferDst,
            BufferUsage::Uniform => Self::UniformBuffer,
            BufferUsage::Storage => Self::StorageBuffer,
            BufferUsage::Index => Self::IndexBuffer,
            BufferUsage::Vertex => Self::VertexBuffer,
        }
    }

    pub fn raw_buffer_usage(&self) -> ash::vk::BufferUsageFlags {
        match self {
            VulkanBufferUsage::TransferSrc => {
                ash::vk::BufferUsageFlags::TRANSFER_SRC
            },
            VulkanBufferUsage::TransferDst => {
                ash::vk::BufferUsageFlags::TRANSFER_DST
            },
            VulkanBufferUsage::UniformBuffer => {
                ash::vk::BufferUsageFlags::UNIFORM_BUFFER
            },
            VulkanBufferUsage::StorageBuffer => {
                ash::vk::BufferUsageFlags::STORAGE_BUFFER
            },
            VulkanBufferUsage::IndexBuffer => {
                ash::vk::BufferUsageFlags::INDEX_BUFFER
            },
            VulkanBufferUsage::VertexBuffer => {
                ash::vk::BufferUsageFlags::VERTEX_BUFFER
            },
            VulkanBufferUsage::IndirectBuffer => {
                ash::vk::BufferUsageFlags::INDIRECT_BUFFER
            },
        }
    }

    pub fn raw_combined_buffer_usage(
        usage: &[Self],
    ) -> ash::vk::BufferUsageFlags {
        usage
            .iter()
            .fold(ash::vk::BufferUsageFlags::empty(), |acc, v| {
                acc | v.raw_buffer_usage()
            })
    }
}
