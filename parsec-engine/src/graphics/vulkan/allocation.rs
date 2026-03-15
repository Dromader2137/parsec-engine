use crate::graphics::vulkan::{
    allocator::{VulkanMemoryProperties, VulkanMemoryRequirements},
    device::VulkanDevice,
};

struct VulkanMemoryBlock {
    start: u64,
    size: u64,
}

pub struct VulkanAllocation {
    memory: ash::vk::DeviceMemory,
    memory_index: u32,
    memory_size: u64,
    memory_blocks: Vec<VulkanMemoryBlock>,
}

impl VulkanAllocation {
    pub fn new(
        device: &VulkanDevice,
        memory_index: u32,
        memory_size: u64,
    ) -> Result<VulkanAllocation, VulkanAllocationError> {
        let allocate_info = ash::vk::MemoryAllocateInfo {
            allocation_size: memory_size,
            memory_type_index: memory_index,
            ..Default::default()
        };

        let memory = unsafe {
            device
                .raw_device()
                .allocate_memory(&allocate_info, None)
                .map_err(|err| {
                    VulkanAllocationError::AllocateMemoryError(err)
                })?
        };

        Ok(VulkanAllocation {
            memory,
            memory_index,
            memory_size,
            memory_blocks: vec![VulkanMemoryBlock {
                start: 0,
                size: memory_size,
            }],
        })
    }

    pub fn try_bind_buffer_memory(

    ) -> Result<>
}


#[derive(Debug)]
pub enum VulkanAllocationError {
    UnableToFindSuitableMemory,
    AllocateMemoryError(ash::vk::Result),
}
