use std::sync::{nonpoison::Mutex, Arc};

use crate::graphics::vulkan::{allocator::Allocator, buffer::Buffer};

pub enum AllocationError {
    
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryLocation {
    CpuSide,
    MappableGpuSide,
    GpuSide
}

pub struct Allocation {
    allocator: Allocator,
    buffers: Vec<Arc<RwL>>,
    memory: ash::vk::DeviceMemory,
    memory_location: u64,
    memory_size: u64,
    free_memory_start: u64,
}

impl Allocation {
    pub fn create_buffer(&mut self) -> Result<Arc<Buffer>, AllocationError> {
    }
}
