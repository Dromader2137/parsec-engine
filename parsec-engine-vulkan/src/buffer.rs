use std::fmt::Debug;

use parsec_engine_graphics::buffer::BufferContent;
use parsec_engine_utils::create_counter;

use crate::{
    allocation::VulkanAllocationError,
    allocator::{
        VulkanAllocator, VulkanMemoryProperties, VulkanMemoryRequirements,
    },
    buffer_usage::VulkanBufferUsage,
    device::VulkanDevice,
    memory::VulkanMemory,
};

pub struct VulkanBuffer {
    id: u32,
    allocation_id: u32,
    memory: VulkanMemory,
    memory_offset: u64,
    memory_size: u64,
    size: u64,
    raw_buffer: ash::vk::Buffer,
}

impl Debug for VulkanBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Buffer").field("size", &self.size).finish()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum VulkanBufferError {
    #[error("Failed to create a Vulkan buffer: {0}")]
    CreationError(ash::vk::Result),
    #[error("Failed to allocate memory for a Vulkan buffer: {0:?}")]
    AllocationError(VulkanAllocationError),
    #[error("Failed to bind a Vulkan buffer: {0}")]
    BindError(ash::vk::Result),
    #[error("Failed to map memory for a Vulkan buffer: {0}")]
    MapError(ash::vk::Result),
    #[error("New data size doesen't match current buffer size")]
    SizeMismatch,
    #[error("Can't map device local memory")]
    CannotMapDeviceLocalMemory,
}

create_counter! {BUFFER_ID_COUNTER}
impl VulkanBuffer {
    pub fn new(
        device: &VulkanDevice,
        allocator: &mut VulkanAllocator,
        size: u64,
        usage: &[VulkanBufferUsage],
        memory_properties: VulkanMemoryProperties,
    ) -> Result<VulkanBuffer, VulkanBufferError> {
        let buffer_info = ash::vk::BufferCreateInfo::default()
            .size(size)
            .usage(VulkanBufferUsage::raw_combined_buffer_usage(usage))
            .sharing_mode(ash::vk::SharingMode::EXCLUSIVE);

        let buffer =
            unsafe { device.raw_device().create_buffer(&buffer_info, None) }
                .map_err(VulkanBufferError::CreationError)?;

        let memory_requirements =
            VulkanMemoryRequirements::from_raw_requirements(unsafe {
                device.raw_device().get_buffer_memory_requirements(buffer)
            });

        let (memory, memory_offset, memory_size, allocation_id) = allocator
            .get_memory(device, memory_properties, memory_requirements)
            .map_err(VulkanBufferError::AllocationError)?;

        unsafe {
            device.raw_device().bind_buffer_memory(
                buffer,
                memory.raw_memory(),
                memory_offset,
            )
        }
        .map_err(VulkanBufferError::BindError)?;

        Ok(VulkanBuffer {
            id: BUFFER_ID_COUNTER.next(),
            allocation_id,
            raw_buffer: buffer,
            memory,
            memory_offset,
            memory_size,
            size,
        })
    }

    pub fn from_vec(
        device: &VulkanDevice,
        allocator: &mut VulkanAllocator,
        data: BufferContent<'_>,
        usage: &[VulkanBufferUsage],
        memory_properties: VulkanMemoryProperties,
    ) -> Result<VulkanBuffer, VulkanBufferError> {
        let size = data.data.len() as u64;

        let buffer_info = ash::vk::BufferCreateInfo::default()
            .size(size)
            .usage(VulkanBufferUsage::raw_combined_buffer_usage(usage))
            .sharing_mode(ash::vk::SharingMode::EXCLUSIVE);

        let buffer =
            unsafe { device.raw_device().create_buffer(&buffer_info, None) }
                .map_err(VulkanBufferError::CreationError)?;

        let memory_requirements =
            VulkanMemoryRequirements::from_raw_requirements(unsafe {
                device.raw_device().get_buffer_memory_requirements(buffer)
            });

        let (memory, memory_offset, memory_size, allocation_id) = allocator
            .get_memory(device, memory_properties, memory_requirements)
            .map_err(VulkanBufferError::AllocationError)?;

        if memory.properties() == VulkanMemoryProperties::Device {
            return Err(VulkanBufferError::CannotMapDeviceLocalMemory);
        } else {
            let memory_ptr = unsafe {
                device.raw_device().map_memory(
                    memory.raw_memory(),
                    memory_offset,
                    memory_size,
                    ash::vk::MemoryMapFlags::empty(),
                )
            }
            .map_err(VulkanBufferError::MapError)?;

            unsafe {
                std::ptr::copy_nonoverlapping(
                    data.data.as_ptr(),
                    memory_ptr as *mut u8,
                    data.data.len(),
                );
            }

            unsafe { device.raw_device().unmap_memory(memory.raw_memory()) };
        }

        if let Err(err) = unsafe {
            device.raw_device().bind_buffer_memory(
                buffer,
                memory.raw_memory(),
                memory_offset,
            )
        } {
            return Err(VulkanBufferError::BindError(err));
        }

        Ok(VulkanBuffer {
            id: BUFFER_ID_COUNTER.next(),
            allocation_id,
            raw_buffer: buffer,
            memory,
            memory_offset,
            memory_size,
            size,
        })
    }

    pub fn update(
        &self,
        device: &VulkanDevice,
        data: BufferContent<'_>,
    ) -> Result<(), VulkanBufferError> {
        let size = data.data.len() as u64;

        if size != self.size {
            return Err(VulkanBufferError::SizeMismatch);
        }

        if self.memory.properties() == VulkanMemoryProperties::Device {
            return Err(VulkanBufferError::CannotMapDeviceLocalMemory);
        }

        let memory_ptr = unsafe {
            device.raw_device().map_memory(
                self.memory.raw_memory(),
                self.memory_offset,
                self.memory_size,
                ash::vk::MemoryMapFlags::empty(),
            )
        }
        .map_err(VulkanBufferError::MapError)?;

        unsafe {
            std::ptr::copy_nonoverlapping(
                data.data.as_ptr(),
                memory_ptr as *mut u8,
                data.data.len(),
            );
        }

        unsafe { device.raw_device().unmap_memory(self.memory.raw_memory()) };

        Ok(())
    }

    pub fn destroy(
        self,
        device: &VulkanDevice,
        allocator: &mut VulkanAllocator,
    ) {
        unsafe {
            device
                .raw_device()
                .destroy_buffer(*self.get_buffer_raw(), None)
        }

        allocator.free(
            device,
            self.allocation_id,
            self.memory_offset,
            self.memory_size,
        );
    }

    pub fn get_buffer_raw(&self) -> &ash::vk::Buffer { &self.raw_buffer }

    pub fn id(&self) -> u32 { self.id }

    pub fn size(&self) -> u64 { self.size }
}
