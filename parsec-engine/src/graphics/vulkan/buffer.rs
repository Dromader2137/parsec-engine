use std::fmt::Debug;

use crate::graphics::vulkan::{
        allocation::VulkanAllocationError, allocator::{
            VulkanAllocator, VulkanMemoryProperties, VulkanMemoryRequirements,
        }, buffer_usage::VulkanBufferUsage, device::VulkanDevice, memory::VulkanMemory
    };

pub struct VulkanBuffer {
    id: u32,
    buffer: ash::vk::Buffer,
    memory: VulkanMemory,
    memory_offset: u64,
    pub size: u64,
    pub len: u32,
}

impl Debug for VulkanBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Buffer")
            .field("size", &self.size)
            .field("len", &self.len)
            .finish()
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
    SizaMismatch,
    #[error("New data len doesen't match current buffer len")]
    LenMismatch,
}

crate::create_counter! {ID_COUNTER}
impl VulkanBuffer {
    pub fn from_vec<T: Clone + Copy>(
        device: &VulkanDevice,
        allocator: &mut VulkanAllocator,
        data: &[T],
        usage: &[VulkanBufferUsage],
        memory_properties: VulkanMemoryProperties,
    ) -> Result<VulkanBuffer, VulkanBufferError> {
        let size = data.len() * size_of::<T>();

        let index_buffer_info = ash::vk::BufferCreateInfo::default()
            .size(size as u64)
            .usage(VulkanBufferUsage::raw_combined_buffer_usage(usage))
            .sharing_mode(ash::vk::SharingMode::EXCLUSIVE);

        let buffer = match unsafe {
            device.raw_handle().create_buffer(&index_buffer_info, None)
        } {
            Ok(val) => val,
            Err(err) => return Err(VulkanBufferError::CreationError(err)),
        };

        let memory_requirements =
            VulkanMemoryRequirements::from_raw_requirements(unsafe {
                device.raw_handle().get_buffer_memory_requirements(buffer)
            });

        let (memory, memory_offset) = allocator
            .get_memory(device, memory_properties, memory_requirements)
            .map_err(|err| VulkanBufferError::AllocationError(err))?;

        let memory_ptr = match unsafe {
            device.raw_handle().map_memory(
                memory.raw_memory(),
                memory_offset,
                memory_requirements
                    .size
                    .next_multiple_of(memory_requirements.alignment),
                ash::vk::MemoryMapFlags::empty(),
            )
        } {
            Ok(val) => val,
            Err(err) => return Err(VulkanBufferError::MapError(err)),
        };

        let mut slice = unsafe {
            ash::util::Align::new(
                memory_ptr,
                align_of::<T>() as u64,
                memory_requirements.size,
            )
        };

        slice.copy_from_slice(&data);
        unsafe { device.raw_handle().unmap_memory(memory.raw_memory()) };
        if let Err(err) = unsafe {
            device.raw_handle().bind_buffer_memory(
                buffer,
                memory.raw_memory(),
                memory_offset,
            )
        } {
            return Err(VulkanBufferError::BindError(err));
        }

        Ok(VulkanBuffer {
            id: ID_COUNTER.next(),
            buffer,
            memory,
            size: size as u64,
            len: data.len() as u32,
            memory_offset,
        })
    }

    pub fn update<T: Clone + Copy>(
        &self,
        device: &VulkanDevice,
        data: &[T],
    ) -> Result<(), VulkanBufferError> {
        let size = (data.len() * size_of::<T>()) as u64;

        if data.len() as u32 != self.len {
            return Err(VulkanBufferError::LenMismatch);
        }

        if size != self.size {
            return Err(VulkanBufferError::SizaMismatch);
        }

        let memory_ptr = match unsafe {
            device.raw_handle().map_memory(
                self.memory.raw_memory(),
                self.memory_offset,
                self.size,
                ash::vk::MemoryMapFlags::empty(),
            )
        } {
            Ok(val) => val,
            Err(err) => return Err(VulkanBufferError::MapError(err)),
        };

        let mut slice = unsafe {
            ash::util::Align::<T>::new(
                memory_ptr,
                align_of::<u32>() as u64,
                self.size,
            )
        };

        slice.copy_from_slice(&data);

        unsafe { device.raw_handle().unmap_memory(self.memory.raw_memory()) };

        Ok(())
    }

    pub fn destroy(self, device: &VulkanDevice) {
        unsafe {
            device
                .raw_handle()
                .destroy_buffer(*self.get_buffer_raw(), None)
        }
    }

    pub fn get_buffer_raw(&self) -> &ash::vk::Buffer { &self.buffer }

    pub fn id(&self) -> u32 { self.id }
}
