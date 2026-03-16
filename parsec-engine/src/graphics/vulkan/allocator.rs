use std::collections::HashMap;

use crate::graphics::vulkan::{
    allocation::{VulkanAllocation, VulkanAllocationError},
    device::VulkanDevice, memory::VulkanMemory,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VulkanMemoryProperties {
    Host,
    Device,
    DeviceHostVisible,
}

impl VulkanMemoryProperties {
    pub fn raw_memory_properties(&self) -> ash::vk::MemoryPropertyFlags {
        match self {
            VulkanMemoryProperties::Host => {
                ash::vk::MemoryPropertyFlags::HOST_VISIBLE
                    | ash::vk::MemoryPropertyFlags::HOST_COHERENT
            },
            VulkanMemoryProperties::Device => {
                ash::vk::MemoryPropertyFlags::DEVICE_LOCAL
            },
            VulkanMemoryProperties::DeviceHostVisible => {
                ash::vk::MemoryPropertyFlags::HOST_VISIBLE
                    | ash::vk::MemoryPropertyFlags::HOST_VISIBLE
                    | ash::vk::MemoryPropertyFlags::HOST_COHERENT
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VulkanMemoryRequirements {
    pub size: u64,
    pub alignment: u64,
    pub memory_type_bits: u32,
}

impl VulkanMemoryRequirements {
    pub fn from_raw_requirements(
        req: ash::vk::MemoryRequirements,
    ) -> VulkanMemoryRequirements {
        VulkanMemoryRequirements {
            size: req.size,
            alignment: req.alignment,
            memory_type_bits: req.memory_type_bits,
        }
    }

    pub fn raw_memory_requirements(&self) -> ash::vk::MemoryRequirements {
        ash::vk::MemoryRequirements {
            size: self.size,
            alignment: self.alignment,
            memory_type_bits: self.memory_type_bits,
        }
    }
}

pub struct VulkanAllocator {
    allocations_by_type: [Vec<VulkanAllocation>; 32],
}

impl VulkanAllocator {
    pub fn new() -> VulkanAllocator {
        VulkanAllocator {
            allocations_by_type: std::array::from_fn(|_| Vec::new()),
        }
    }

    pub fn get_memory(
        &mut self,
        device: &VulkanDevice,
        memory_type: VulkanMemoryProperties,
        memory_requirements: VulkanMemoryRequirements,
    ) -> Result<(VulkanMemory, u64), VulkanAllocationError> {
        let memory_ids =
            find_memorytype_indices(&memory_requirements, &memory_type, device);

        if memory_ids.is_empty() {
            return Err(VulkanAllocationError::UnableToFindSuitableMemory);
        }

        let memory_id = *memory_ids
            .iter()
            .find(|x| !self.allocations_by_type[**x as usize].is_empty())
            .unwrap_or(&memory_ids[0]);

        let allocations = &mut self.allocations_by_type[memory_id as usize];

        for allocation in allocations.iter_mut() {
            if let Some(offset_mem) =
                allocation.try_get_free_memory(memory_requirements)
            {
                return Ok(offset_mem);
            }
        }

        allocations.push(VulkanAllocation::new(device, memory_id, 1 << 26)?);

        let allocation = allocations
            .last_mut()
            .expect("There always is an allocation at this point!");

        allocation.try_get_free_memory(memory_requirements)
            .ok_or(VulkanAllocationError::UnableToAllocateThisSize)
    }
}

fn find_memorytype_indices(
    memory_requirements: &VulkanMemoryRequirements,
    memory_type: &VulkanMemoryProperties,
    device: &VulkanDevice,
) -> Vec<u32> {
    let device_memory_prop = device.raw_memory_properties();
    let memory_prop = memory_type.raw_memory_properties();
    let memory_req = memory_requirements.raw_memory_requirements();
    device_memory_prop.memory_types[..device_memory_prop.memory_type_count as _]
        .iter()
        .enumerate()
        .filter(|(index, memory_type)| {
            (1 << index) & memory_req.memory_type_bits != 0
                && memory_type.property_flags & memory_prop == memory_prop
        })
        .map(|(index, _memory_type)| index as _)
        .collect()
}
