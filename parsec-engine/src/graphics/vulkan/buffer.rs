use std::fmt::Debug;

use crate::{
    graphics::{buffer::BufferUsage, vulkan::device::VulkanDevice},
    utils::id_counter::IdCounter,
};

#[allow(unused)]
pub struct VulkanBuffer {
    id: u32,
    device_id: u32,
    buffer: ash::vk::Buffer,
    memory: ash::vk::DeviceMemory,
    memory_size: ash::vk::DeviceSize,
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
    #[error("Failed to find suitable memory for a Vulkan buffer")]
    UnableToFindSuitableMemory,
    #[error("Failed to allocate memory for a Vulkan buffer: {0}")]
    AllocationError(ash::vk::Result),
    #[error("Failed to bind a Vulkan buffer: {0}")]
    BindError(ash::vk::Result),
    #[error("Failed to map memory for a Vulkan buffer: {0}")]
    MapError(ash::vk::Result),
    #[error("New data size doesen't match current buffer size")]
    SizaMismatch,
    #[error("New data len doesen't match current buffer len")]
    LenMismatch,
    #[error("Buffers created to different devices")]
    DeviceMismatch,
}

pub type VulkanBufferUsage = ash::vk::BufferUsageFlags;

impl From<BufferUsage> for VulkanBufferUsage {
    fn from(value: BufferUsage) -> Self {
        match value {
            BufferUsage::Uniform => VulkanBufferUsage::UNIFORM_BUFFER,
            BufferUsage::Index => VulkanBufferUsage::INDEX_BUFFER,
            BufferUsage::Vertex => VulkanBufferUsage::VERTEX_BUFFER,
            BufferUsage::TransferSrc => VulkanBufferUsage::TRANSFER_SRC,
            BufferUsage::TransferDst => VulkanBufferUsage::TRANSFER_DST,
        }
    }
}

static ID_COUNTER: once_cell::sync::Lazy<IdCounter> =
    once_cell::sync::Lazy::new(|| IdCounter::new(0));
impl VulkanBuffer {
    pub fn from_vec<T: Clone + Copy>(
        device: &VulkanDevice,
        data: &[T],
        usage: VulkanBufferUsage,
    ) -> Result<VulkanBuffer, VulkanBufferError> {
        let size = data.len() * size_of::<T>();

        let index_buffer_info = ash::vk::BufferCreateInfo::default()
            .size(size as u64)
            .usage(usage.into())
            .sharing_mode(ash::vk::SharingMode::EXCLUSIVE);

        let buffer = match unsafe {
            device
                .get_device_raw()
                .create_buffer(&index_buffer_info, None)
        } {
            Ok(val) => val,
            Err(err) => return Err(VulkanBufferError::CreationError(err)),
        };

        let memory_req = unsafe {
            device
                .get_device_raw()
                .get_buffer_memory_requirements(buffer)
        };
        let memory_index = match find_memorytype_index(
            &memory_req,
            ash::vk::MemoryPropertyFlags::HOST_VISIBLE
                | ash::vk::MemoryPropertyFlags::DEVICE_LOCAL,
            device,
        ) {
            Some(val) => val,
            None => return Err(VulkanBufferError::UnableToFindSuitableMemory),
        };

        let allocate_info = ash::vk::MemoryAllocateInfo {
            allocation_size: memory_req.size,
            memory_type_index: memory_index,
            ..Default::default()
        };
        let memory = match unsafe {
            device
                .get_device_raw()
                .allocate_memory(&allocate_info, None)
        } {
            Ok(val) => val,
            Err(err) => return Err(VulkanBufferError::AllocationError(err)),
        };

        let memory_ptr = match unsafe {
            device.get_device_raw().map_memory(
                memory,
                0,
                memory_req.size,
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
                memory_req.size,
            )
        };

        slice.copy_from_slice(&data);
        unsafe { device.get_device_raw().unmap_memory(memory) };
        if let Err(err) = unsafe {
            device
                .get_device_raw()
                .bind_buffer_memory(buffer, memory, 0)
        } {
            return Err(VulkanBufferError::BindError(err));
        }

        Ok(VulkanBuffer {
            id: ID_COUNTER.next(),
            device_id: device.id(),
            buffer,
            memory,
            memory_size: memory_req.size,
            size: size as u64,
            len: data.len() as u32,
        })
    }

    pub fn update<T: Clone + Copy>(
        &self,
        device: &VulkanDevice,
        data: &[T],
    ) -> Result<(), VulkanBufferError> {
        if self.device_id != device.id() {
            return Err(VulkanBufferError::DeviceMismatch);
        }

        let size = (data.len() * size_of::<T>()) as u64;

        if data.len() as u32 != self.len {
            return Err(VulkanBufferError::LenMismatch);
        }

        if size != self.size {
            return Err(VulkanBufferError::SizaMismatch);
        }

        let memory_ptr = match unsafe {
            device.get_device_raw().map_memory(
                self.memory,
                0,
                self.memory_size,
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
                self.memory_size,
            )
        };

        slice.copy_from_slice(&data);

        unsafe { device.get_device_raw().unmap_memory(self.memory) };

        Ok(())
    }

    pub fn delete_buffer(
        self,
        device: &VulkanDevice,
    ) -> Result<(), VulkanBufferError> {
        if self.device_id != device.id() {
            return Err(VulkanBufferError::DeviceMismatch);
        }

        unsafe {
            device
                .get_device_raw()
                .destroy_buffer(*self.get_buffer_raw(), None);
            device
                .get_device_raw()
                .free_memory(*self.get_memory_raw(), None);
        }
        Ok(())
    }

    pub fn get_buffer_raw(&self) -> &ash::vk::Buffer { &self.buffer }

    pub fn get_memory_raw(&self) -> &ash::vk::DeviceMemory { &self.memory }

    pub fn device_id(&self) -> u32 { self.device_id }

    pub fn id(&self) -> u32 { self.id }
}

pub fn find_memorytype_index(
    memory_req: &ash::vk::MemoryRequirements,
    flags: ash::vk::MemoryPropertyFlags,
    device: &VulkanDevice,
) -> Option<u32> {
    let memory_prop = device.memory_properties();
    memory_prop.memory_types[..memory_prop.memory_type_count as _]
        .iter()
        .enumerate()
        .find(|(index, memory_type)| {
            (1 << index) & memory_req.memory_type_bits != 0
                && memory_type.property_flags & flags == flags
        })
        .map(|(index, _memory_type)| index as _)
}
