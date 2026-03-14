use std::fmt::Debug;

use crate::{
    arena::handle::{Handle, WeakHandle},
    graphics::{
        buffer::BufferUsage,
        vulkan::{VulkanBackend, device::VulkanDevice},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulkanBufferUsage {
    TransferSrc,
    TransferDst,
    UniformBuffer,
    StorageBuffer,
    IndexBuffer,
    VertexBuffer,
    IndirectBuffer,
}

impl VulkanBufferUsage {
    pub fn new(value: BufferUsage) -> Self {
        match value {
            BufferUsage::TransferSrc => Self::TransferSrc,
            BufferUsage::TransferDst => Self::TransferDst,
            BufferUsage::Uniform => Self::UniformBuffer,
            BufferUsage::Index => Self::IndexBuffer,
            BufferUsage::Vertex => Self::VertexBuffer,
        }
    }

    pub fn raw_buffer_usage(&self) -> ash::vk::BufferUsageFlags {
        match self {
            VulkanBufferUsage::TransferSrc => {
                ash::vk::BufferUsageFlags::TRANSFER_SRC
            },
            VulkanBufferUsage::TransferDst => {
                ash::vk::BufferUsageFlags::TRANSFER_DST
            },
            VulkanBufferUsage::UniformBuffer => {
                ash::vk::BufferUsageFlags::UNIFORM_BUFFER
            },
            VulkanBufferUsage::StorageBuffer => {
                ash::vk::BufferUsageFlags::STORAGE_BUFFER
            },
            VulkanBufferUsage::IndexBuffer => {
                ash::vk::BufferUsageFlags::INDEX_BUFFER
            },
            VulkanBufferUsage::VertexBuffer => {
                ash::vk::BufferUsageFlags::VERTEX_BUFFER
            },
            VulkanBufferUsage::IndirectBuffer => {
                ash::vk::BufferUsageFlags::INDIRECT_BUFFER
            },
        }
    }

    fn raw_combined_buffer_usage(usage: &[Self]) -> ash::vk::BufferUsageFlags {
        usage
            .iter()
            .fold(ash::vk::BufferUsageFlags::empty(), |acc, v| {
                acc | v.raw_buffer_usage()
            })
    }
}

pub struct VulkanBuffer {
    device_handle: Handle<VulkanDevice>,
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

impl VulkanBuffer {
    pub fn from_vec<T: Clone + Copy>(
        arenas: &mut VulkanBackend,
        device_handle: Handle<VulkanDevice>,
        data: &[T],
        usage: &[VulkanBufferUsage],
    ) -> Result<WeakHandle<VulkanBuffer>, VulkanBufferError> {
        let device = arenas.devices.get_mut(device_handle.clone());

        let size = data.len() * size_of::<T>();

        let index_buffer_info = ash::vk::BufferCreateInfo::default()
            .size(size as u64)
            .usage(VulkanBufferUsage::raw_combined_buffer_usage(usage))
            .sharing_mode(ash::vk::SharingMode::EXCLUSIVE);

        let buffer = match unsafe {
            device.raw_device().create_buffer(&index_buffer_info, None)
        } {
            Ok(val) => val,
            Err(err) => return Err(VulkanBufferError::CreationError(err)),
        };

        let memory_req = unsafe {
            device.raw_device().get_buffer_memory_requirements(buffer)
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
            device.raw_device().allocate_memory(&allocate_info, None)
        } {
            Ok(val) => val,
            Err(err) => return Err(VulkanBufferError::AllocationError(err)),
        };

        let memory_ptr = match unsafe {
            device.raw_device().map_memory(
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
        unsafe { device.raw_device().unmap_memory(memory) };
        if let Err(err) =
            unsafe { device.raw_device().bind_buffer_memory(buffer, memory, 0) }
        {
            return Err(VulkanBufferError::BindError(err));
        }

        let buffer = VulkanBuffer {
            device_handle,
            buffer,
            memory,
            memory_size: memory_req.size,
            size: size as u64,
            len: data.len() as u32,
        };

        let handle = arenas.buffers.add(buffer);
        device.add_buffer(handle.clone());
        Ok(handle.downgrade())
    }

    pub fn update<T: Clone + Copy>(
        &self,
        arenas: &mut VulkanBackend,
        data: &[T],
    ) -> Result<(), VulkanBufferError> {
        let device = arenas.devices.get(self.device_handle.clone());

        let size = (data.len() * size_of::<T>()) as u64;

        if data.len() as u32 != self.len {
            return Err(VulkanBufferError::LenMismatch);
        }

        if size != self.size {
            return Err(VulkanBufferError::SizaMismatch);
        }

        let memory_ptr = match unsafe {
            device.raw_device().map_memory(
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

        unsafe { device.raw_device().unmap_memory(self.memory) };

        Ok(())
    }

    pub fn delete_buffer(
        self,
        arenas: &mut VulkanBackend,
    ) -> Result<(), VulkanBufferError> {
        let device = arenas.devices.get(self.device_handle.clone());

        unsafe {
            device
                .raw_device()
                .destroy_buffer(*self.get_buffer_raw(), None);
            device
                .raw_device()
                .free_memory(*self.get_memory_raw(), None);
        }
        Ok(())
    }

    pub fn get_buffer_raw(&self) -> &ash::vk::Buffer { &self.buffer }

    pub fn get_memory_raw(&self) -> &ash::vk::DeviceMemory { &self.memory }

    pub fn device_handle(&self) -> Handle<VulkanDevice> {
        self.device_handle.clone()
    }
}

pub fn find_memorytype_index(
    memory_req: &ash::vk::MemoryRequirements,
    flags: ash::vk::MemoryPropertyFlags,
    device: &VulkanDevice,
) -> Option<u32> {
    let memory_prop = device.raw_memory_properties();
    memory_prop.memory_types[..memory_prop.memory_type_count as _]
        .iter()
        .enumerate()
        .find(|(index, memory_type)| {
            (1 << index) & memory_req.memory_type_bits != 0
                && memory_type.property_flags & flags == flags
        })
        .map(|(index, _memory_type)| index as _)
}
