use crate::{allocator::VulkanMemoryProperties, device::VulkanDevice};

#[derive(Debug, Clone)]
pub struct VulkanMemory {
    size: u64,
    properties: VulkanMemoryProperties,
    raw_memory: ash::vk::DeviceMemory,
}

impl VulkanMemory {
    pub fn new(
        device: &VulkanDevice,
        memory_properties: VulkanMemoryProperties,
        memory_index: u32,
        memory_size: u64,
    ) -> Result<VulkanMemory, VulkanMemoryError> {
        let allocate_info = ash::vk::MemoryAllocateInfo {
            allocation_size: memory_size,
            memory_type_index: memory_index,
            ..Default::default()
        };

        let memory = unsafe {
            device
                .raw_device()
                .allocate_memory(&allocate_info, None)
                .map_err(|err| VulkanMemoryError::AllocateMemoryError(err))?
        };

        Ok(VulkanMemory {
            raw_memory: memory,
            properties: memory_properties,
            size: memory_size,
        })
    }

    pub fn size(&self) -> u64 { self.size }

    pub fn raw_memory(&self) -> ash::vk::DeviceMemory { self.raw_memory }

    pub fn properties(&self) -> VulkanMemoryProperties { self.properties }
}

#[derive(Debug, thiserror::Error)]
pub enum VulkanMemoryError {
    #[error("Failed to allocate memory: {0}")]
    AllocateMemoryError(ash::vk::Result),
}
