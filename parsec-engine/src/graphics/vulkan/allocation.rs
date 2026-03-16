use crate::graphics::vulkan::{
    allocator::{VulkanMemoryProperties, VulkanMemoryRequirements},
    device::VulkanDevice,
    memory::{VulkanMemory, VulkanMemoryError},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VulkanMemoryBlock {
    start: u64,
    end: u64,
}

pub struct VulkanAllocation {
    memory: VulkanMemory,
    memory_index: u32,
    memory_blocks: Vec<VulkanMemoryBlock>,
}

impl VulkanAllocation {
    pub fn new(
        device: &VulkanDevice,
        memory_index: u32,
        memory_size: u64,
    ) -> Result<VulkanAllocation, VulkanAllocationError> {
        let memory = VulkanMemory::new(device, memory_index, memory_size)
            .map_err(|err| VulkanAllocationError::AllocateMemoryError(err))?;

        Ok(VulkanAllocation {
            memory,
            memory_index,
            memory_blocks: vec![VulkanMemoryBlock {
                start: 0,
                end: memory_size - 1,
            }],
        })
    }

    /// (memory, offset)
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
            return Some((self.memory.clone(), start))
        }
        None
    }
}

#[derive(Debug)]
pub enum VulkanAllocationError {
    UnableToFindSuitableMemory,
    AllocateMemoryError(VulkanMemoryError),
    UnableToAllocateThisSize,
}
