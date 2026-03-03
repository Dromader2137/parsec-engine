#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulkanAccess {
    IndirectCommandRead,
    IndexRead,
    VertexAttributeRead,
    UniformRead,
    InputAttachmentRead,
    ShaderRead,
    ShaderWrite,
    ColorAttachmentRead,
    ColorAttachmentWrite,
    DepthAttachmentRead,
    DepthAttachmentWrite,
    TransferRead,
    TransferWrite,
    HostRead,
    HostWrite,
    MemoryRead,
    MemoryWrite,
    None,
}

impl VulkanAccess {
    fn raw_access_flag(&self) -> ash::vk::AccessFlags {
        match self {
            VulkanAccess::IndirectCommandRead => {
                ash::vk::AccessFlags::INDIRECT_COMMAND_READ
            },
            VulkanAccess::IndexRead => ash::vk::AccessFlags::INDEX_READ,
            VulkanAccess::VertexAttributeRead => {
                ash::vk::AccessFlags::VERTEX_ATTRIBUTE_READ
            },
            VulkanAccess::UniformRead => ash::vk::AccessFlags::UNIFORM_READ,
            VulkanAccess::InputAttachmentRead => {
                ash::vk::AccessFlags::INPUT_ATTACHMENT_READ
            },
            VulkanAccess::ShaderRead => ash::vk::AccessFlags::SHADER_READ,
            VulkanAccess::ShaderWrite => ash::vk::AccessFlags::SHADER_WRITE,
            VulkanAccess::ColorAttachmentRead => {
                ash::vk::AccessFlags::COLOR_ATTACHMENT_READ
            },
            VulkanAccess::ColorAttachmentWrite => {
                ash::vk::AccessFlags::COLOR_ATTACHMENT_WRITE
            },
            VulkanAccess::DepthAttachmentRead => {
                ash::vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
            },
            VulkanAccess::DepthAttachmentWrite => {
                ash::vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE
            },
            VulkanAccess::TransferRead => ash::vk::AccessFlags::TRANSFER_READ,
            VulkanAccess::TransferWrite => ash::vk::AccessFlags::TRANSFER_WRITE,
            VulkanAccess::HostRead => ash::vk::AccessFlags::HOST_READ,
            VulkanAccess::HostWrite => ash::vk::AccessFlags::HOST_WRITE,
            VulkanAccess::MemoryRead => ash::vk::AccessFlags::MEMORY_READ,
            VulkanAccess::MemoryWrite => ash::vk::AccessFlags::MEMORY_WRITE,
            VulkanAccess::None => ash::vk::AccessFlags::NONE,
        }
    }

    pub fn raw_combined_access_flag(access: &[Self]) -> ash::vk::AccessFlags {
        access.iter().fold(ash::vk::AccessFlags::empty(), |acc, x| {
            acc | x.raw_access_flag()
        })
    }
}

