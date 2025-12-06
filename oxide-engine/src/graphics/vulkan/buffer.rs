use std::{
    fmt::Debug,
    sync::atomic::{AtomicU32, Ordering},
};

use crate::graphics::vulkan::{
    VulkanError, device::Device, physical_device::PhysicalDevice,
};

#[allow(unused)]
pub struct Buffer {
    id: u32,
    device_id: u32,
    buffer: ash::vk::Buffer,
    memory: ash::vk::DeviceMemory,
    memory_size: ash::vk::DeviceSize,
    pub size: u64,
    pub len: u32,
}

impl Debug for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Buffer")
            .field("size", &self.size)
            .field("len", &self.len)
            .finish()
    }
}

#[derive(Debug)]
pub enum BufferError {
    CreationError(ash::vk::Result),
    UnableToFindSuitableMemory,
    AllocationError(ash::vk::Result),
    BindError(ash::vk::Result),
    MapError(ash::vk::Result),
    SizaMismatch,
    LenMismatch,
    PhysicalDeviceMismatch,
    DeviceMismatch,
}

impl From<BufferError> for VulkanError {
    fn from(value: BufferError) -> Self { VulkanError::BufferError(value) }
}

pub type BufferUsage = ash::vk::BufferUsageFlags;

impl Buffer {
    const ID_COUNTER: AtomicU32 = AtomicU32::new(0);

    pub fn from_vec<T: Clone + Copy>(
        physical_device: &PhysicalDevice,
        device: &Device,
        data: &[T],
        usage: BufferUsage,
    ) -> Result<Buffer, BufferError> {
        if physical_device.id() != device.physical_device_id() {
            return Err(BufferError::PhysicalDeviceMismatch);
        }

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
            Err(err) => return Err(BufferError::CreationError(err)),
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
            physical_device,
        ) {
            Some(val) => val,
            None => return Err(BufferError::UnableToFindSuitableMemory),
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
            Err(err) => return Err(BufferError::AllocationError(err)),
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
            Err(err) => return Err(BufferError::MapError(err)),
        };

        let mut slice = unsafe {
            ash::util::Align::new(
                memory_ptr,
                align_of::<u32>() as u64,
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
            return Err(BufferError::BindError(err));
        }

        let id = Self::ID_COUNTER.load(Ordering::Acquire);
        Self::ID_COUNTER.store(id + 1, Ordering::Release);

        Ok(Buffer {
            id,
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
        device: &Device,
        data: &[T],
    ) -> Result<(), BufferError> {
        if self.device_id != device.id() {
            return Err(BufferError::DeviceMismatch);
        }

        let size = (data.len() * size_of::<T>()) as u64;

        if data.len() as u32 != self.len {
            return Err(BufferError::LenMismatch);
        }

        if size != self.size {
            return Err(BufferError::SizaMismatch);
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
            Err(err) => return Err(BufferError::MapError(err)),
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

    pub fn get_buffer_raw(&self) -> &ash::vk::Buffer { &self.buffer }

    pub fn get_memory_raw(&self) -> &ash::vk::DeviceMemory { &self.memory }

    pub fn device_id(&self) -> u32 { self.device_id }
}

pub fn find_memorytype_index(
    memory_req: &ash::vk::MemoryRequirements,
    flags: ash::vk::MemoryPropertyFlags,
    physical_device: &PhysicalDevice,
) -> Option<u32> {
    let memory_prop = physical_device.physical_memory_properties();
    memory_prop.memory_types[..memory_prop.memory_type_count as _]
        .iter()
        .enumerate()
        .find(|(index, memory_type)| {
            (1 << index) & memory_req.memory_type_bits != 0
                && memory_type.property_flags & flags == flags
        })
        .map(|(index, _memory_type)| index as _)
}
