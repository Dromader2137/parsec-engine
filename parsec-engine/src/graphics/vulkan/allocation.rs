use std::collections::{
    LinkedList,
    linked_list::{Cursor, CursorMut},
};

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

#[derive(Debug)]
pub struct VulkanAllocation {
    id: u32,
    memory: VulkanMemory,
    free_memory_blocks: LinkedList<VulkanMemoryBlock>,
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

        Ok(VulkanAllocation {
            id: ALLOCATION_ID_COUNTER.next(),
            memory,
            free_memory_blocks: LinkedList::from([VulkanMemoryBlock {
                start: 0,
                end: memory_size - 1,
            }]),
        })
    }

    pub fn free(self, device: &VulkanDevice) {
        unsafe {
            device
                .raw_device()
                .free_memory(self.memory.raw_memory(), None)
        };
    }

    fn find_block_with_size(
        &mut self,
        size: u64,
        alignment: u64,
    ) -> Option<(CursorMut<'_, VulkanMemoryBlock>, u64, u64)> {
        let mut list_cursor = self.free_memory_blocks.cursor_front_mut();
        while let Some(block) = list_cursor.current() {
            let start = block.start.next_multiple_of(alignment);
            let end = start + size.next_multiple_of(alignment) - 1;
            if end > block.end {
                list_cursor.move_next();
                continue;
            }
            return Some((list_cursor, start, end));
        }
        None
    }

    pub fn try_get_free_memory(
        &mut self,
        req: VulkanMemoryRequirements,
    ) -> Option<(VulkanMemory, u64, u64, u32)> {
        let (mut cursor, start, end) =
            self.find_block_with_size(req.size, req.alignment)?;
        let block = cursor.remove_current()?;
        if block.end != end {
            cursor.insert_before(VulkanMemoryBlock {
                start: end + 1,
                end: block.end,
            });
        }
        if block.start != start {
            cursor.insert_before(VulkanMemoryBlock {
                start: block.start,
                end: start - 1,
            });
        }
        Some((self.memory.clone(), start, end - start + 1, self.id))
    }

    fn find_offset_block(
        &mut self,
        offset: u64,
    ) -> Option<CursorMut<'_, VulkanMemoryBlock>> {
        let mut list_cursor = self.free_memory_blocks.cursor_front_mut();
        while let Some(block) = list_cursor.current() {
            if block.start >= offset {
                break;
            }
            list_cursor.move_next();
        }
        Some(list_cursor)
    }

    pub fn is_empty(&self) -> bool {
        self.free_memory_blocks.len() == 1
            && self.free_memory_blocks.front().map(|x| x.start) == Some(0)
            && self.free_memory_blocks.front().map(|x| x.end + 1)
                == Some(self.memory.size())
    }

    pub fn free_part(&mut self, offset: u64, size: u64) -> Option<()> {
        let mut cursor = self.find_offset_block(offset)?;
        let end = offset + size - 1;
        let start = offset;
        cursor.insert_before(VulkanMemoryBlock { start, end });
        cursor.move_prev();
        let mut merge_prev = false;
        let mut merge_next = false;
        if let Some(prev_block) = cursor.peek_prev() {
            merge_prev = prev_block.end + 1 == start;
        }
        if let Some(next_block) = cursor.peek_next() {
            merge_next = end + 1 == next_block.start;
        }
        if merge_prev {
            cursor.move_prev();
            let prev_block = cursor.remove_current()?;
            cursor.current()?.start = prev_block.start;
        }
        if merge_next {
            let curr_block = cursor.remove_current()?;
            cursor.current()?.start = curr_block.start;
        }
        Some(())
    }

    pub fn id(&self) -> u32 { self.id }
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
