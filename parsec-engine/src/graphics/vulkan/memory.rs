use crate::graphics::vulkan::device::VulkanDevice;

#[derive(Debug, Clone)]
pub struct VulkanMemory {
    memory: ash::vk::DeviceMemory,
    size: u64,
}

impl VulkanMemory {
    pub fn new(
        device: &VulkanDevice,
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
                .raw_handle()
                .allocate_memory(&allocate_info, None)
                .map_err(|err| VulkanMemoryError::AllocateMemoryError(err))?
        };

        Ok(VulkanMemory {
            memory,
            size: memory_size,
        })
    }

    pub fn size(&self) -> u64 { self.size }

    pub fn raw_memory(&self) -> ash::vk::DeviceMemory { self.memory }
}

#[derive(Debug)]
pub enum VulkanMemoryError {
    AllocateMemoryError(ash::vk::Result),
}
