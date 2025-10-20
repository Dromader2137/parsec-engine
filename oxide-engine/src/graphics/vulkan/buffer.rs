use std::{fmt::Debug, sync::Arc};

use super::{VulkanError, device::Device, instance::Instance, physical_device::PhysicalDevice};

pub struct Buffer {
    pub device: Arc<Device>,
    buffer: ash::vk::Buffer,
    memory: ash::vk::DeviceMemory,
    memory_size: u64,
    pub size: u64,
    pub len: u32,
}

impl Debug for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Buffer")
            .field("memory_size", &self.memory_size)
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
    LenMismatch
}

impl From<BufferError> for VulkanError {
    fn from(value: BufferError) -> Self {
        VulkanError::BufferError(value)
    }
}

pub type BufferUsage = ash::vk::BufferUsageFlags;

impl Buffer {
    pub fn from_vec<T: Clone + Copy>(device: Arc<Device>, data: Vec<T>, usage: BufferUsage) -> Result<Arc<Buffer>, BufferError> {
        let size = data.len() * size_of::<T>();

        let index_buffer_info = ash::vk::BufferCreateInfo::default()
            .size(size as u64)
            .usage(usage.into())
            .sharing_mode(ash::vk::SharingMode::EXCLUSIVE);

        let buffer = match unsafe { device.get_device_raw().create_buffer(&index_buffer_info, None) } {
            Ok(val) => val,
            Err(err) => return Err(BufferError::CreationError(err)),
        };

        let memory_req = unsafe { device.get_device_raw().get_buffer_memory_requirements(buffer) };
        let memory_index = match find_memorytype_index(
            &memory_req,
            ash::vk::MemoryPropertyFlags::HOST_VISIBLE | ash::vk::MemoryPropertyFlags::HOST_COHERENT,
            device.physical_device.instance.clone(),
            device.physical_device.clone(),
        ) {
            Some(val) => val,
            None => return Err(BufferError::UnableToFindSuitableMemory),
        };

        let allocate_info = ash::vk::MemoryAllocateInfo {
            allocation_size: memory_req.size,
            memory_type_index: memory_index,
            ..Default::default()
        };
        let memory = match unsafe { device.get_device_raw().allocate_memory(&allocate_info, None) } {
            Ok(val) => val,
            Err(err) => return Err(BufferError::AllocationError(err)),
        };

        let memory_ptr = match unsafe {
            device
                .get_device_raw()
                .map_memory(memory, 0, memory_req.size, ash::vk::MemoryMapFlags::empty())
        } {
            Ok(val) => val,
            Err(err) => return Err(BufferError::MapError(err)),
        };

        let mut slice = unsafe { ash::util::Align::new(memory_ptr, align_of::<u32>() as u64, memory_req.size) };

        slice.copy_from_slice(&data);
        unsafe { device.get_device_raw().unmap_memory(memory) };
        if let Err(err) = unsafe { device.get_device_raw().bind_buffer_memory(buffer, memory, 0) } {
            return Err(BufferError::BindError(err));
        }

        Ok(Arc::new(Buffer {
            device,
            buffer,
            memory,
            memory_size: memory_req.size,
            size: size as u64,
            len: data.len() as u32,
        }))
    }

    pub fn update<T: Clone + Copy>(&self, data: Vec<T>) -> Result<(), BufferError> {
        let size = (data.len() * size_of::<T>()) as u64;

        if data.len() as u32 != self.len {
            return Err(BufferError::LenMismatch);
        }

        if size != self.size {
            return Err(BufferError::SizaMismatch);
        }

        let memory_ptr = match unsafe {
            self.device
                .get_device_raw()
                .map_memory(self.memory, 0, self.memory_size, ash::vk::MemoryMapFlags::empty())
        } {
            Ok(val) => val,
            Err(err) => return Err(BufferError::MapError(err)),
        };

        let mut slice = unsafe { ash::util::Align::<T>::new(memory_ptr, align_of::<u32>() as u64, self.memory_size) };

        slice.copy_from_slice(&data);

        unsafe { self.device.get_device_raw().unmap_memory(self.memory) };

        Ok(())
    }

    pub fn get_buffer_raw(&self) -> &ash::vk::Buffer {
        &self.buffer
    }

    pub fn get_memory_raw(&self) -> &ash::vk::DeviceMemory {
        &self.memory
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe { self.device.get_device_raw().destroy_buffer(self.buffer, None) };
        unsafe { self.device.get_device_raw().free_memory(self.memory, None) };
    }
}

pub fn find_memorytype_index(
    memory_req: &ash::vk::MemoryRequirements,
    flags: ash::vk::MemoryPropertyFlags,
    instance: Arc<Instance>,
    physical_device: Arc<PhysicalDevice>,
) -> Option<u32> {
    let memory_prop = unsafe {
        instance
            .get_instance_raw()
            .get_physical_device_memory_properties(*physical_device.get_physical_device_raw())
    };

    memory_prop.memory_types[..memory_prop.memory_type_count as _]
        .iter()
        .enumerate()
        .find(|(index, memory_type)| {
            (1 << index) & memory_req.memory_type_bits != 0 && memory_type.property_flags & flags == flags
        })
        .map(|(index, _memory_type)| index as _)
}

#[derive(Debug)]
pub struct AutoSyncingBuffer<T: Clone + Copy + PartialEq> {
    pub data: Vec<T>,
    old_data: Vec<T>,
    buffer: Arc<Buffer>
}

impl<T: Clone + Copy + PartialEq> AutoSyncingBuffer<T> {
    pub fn new(device: Arc<Device>, data: Vec<T>, usage: BufferUsage) -> Result<AutoSyncingBuffer<T>, BufferError> {
        let buffer = Buffer::from_vec(device, data.clone(), usage)?;
        Ok(AutoSyncingBuffer {
            data: data.clone(),
            old_data: data,
            buffer
        })
    }

    pub fn update(&mut self) -> Result<(), BufferError> {
        if self.data == self.old_data {
            return Ok(());
        }

        self.buffer.update(self.data.clone())?;

        Ok(())
    }
}
