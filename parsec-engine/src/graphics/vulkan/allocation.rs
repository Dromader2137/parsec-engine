use crate::{graphics::vulkan::{
    allocator::{VulkanMemoryProperties, VulkanMemoryRequirements},
    device::VulkanDevice,
    memory::{VulkanMemory, VulkanMemoryError},
}};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VulkanMemoryBlock {
    start: u64,
    end: u64,
}

pub struct VulkanAllocation {
    id: u32,
    memory: VulkanMemory,
    memory_blocks: Vec<VulkanMemoryBlock>,
}

crate::create_counter! {ALLOCATION_ID_COUNTER}
impl VulkanAllocation {
    pub fn new(
        device: &VulkanDevice,
        memory_properties: VulkanMemoryProperties,
        memory_index: u32,
        memory_size: u64,
    ) -> Result<VulkanAllocation, VulkanAllocationError> {
        let memory = VulkanMemory::new(
            device,
            memory_properties,
            memory_index,
            memory_size,
        )?;
        // .map_err(|_| VulkanAllocationError::MemoryError)?;

        Ok(VulkanAllocation {
            id: ALLOCATION_ID_COUNTER.next(),
            memory,
            memory_blocks: vec![VulkanMemoryBlock {
                start: 0,
                end: memory_size - 1,
            }],
        })
    }

    pub fn free(self, device: &VulkanDevice) {
        unsafe {
            device
                .raw_device()
                .free_memory(self.memory.raw_memory(), None)
        };
    }

    pub fn try_get_free_memory(
        &mut self,
        req: VulkanMemoryRequirements,
    ) -> Option<(VulkanMemory, u64)> {
        for idx in (0..self.memory_blocks.len()).rev() {
            let block = self.memory_blocks[idx];
            let start = block.start.next_multiple_of(req.alignment);
            let end = start + req.size.next_multiple_of(req.alignment) - 1;
            if end > block.end {
                continue;
            }
            self.memory_blocks.remove(idx);
            if block.end != end {
                self.memory_blocks.push(VulkanMemoryBlock {
                    start: end + 1,
                    end: block.end,
                });
            }
            if block.start != start {
                self.memory_blocks.push(VulkanMemoryBlock {
                    start: block.start,
                    end: start - 1,
                });
            }
            return Some((self.memory.clone(), start));
        }
        None
    }
}

#[derive(Debug, thiserror::Error)]
pub enum VulkanAllocationError {
    #[error("Failed to find suitable memory")]
    UnableToFindSuitableMemory,
    #[error("Memory error: {0}")]
    MemoryError(#[from] VulkanMemoryError),
    #[error("Failed to allocate memory of size: {0}")]
    UnableToAllocateThisSize(u64),
}
