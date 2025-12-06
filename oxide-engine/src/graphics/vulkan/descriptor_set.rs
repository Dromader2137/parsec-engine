use std::sync::atomic::{AtomicU32, Ordering};

use crate::graphics::vulkan::{
    VulkanError, buffer::Buffer, device::Device, image::ImageView,
};

pub struct DescriptorPool {
    id: u32,
    device_id: u32,
    pool: ash::vk::DescriptorPool,
}

pub struct DescriptorPoolSize {
    size: ash::vk::DescriptorPoolSize,
}

pub struct DescriptorSet {
    id: u32,
    device_id: u32,
    descriptor_pool_id: u32,
    descriptor_layout_id: u32,
    set: ash::vk::DescriptorSet,
}

#[derive(Clone)]
pub struct DescriptorSetLayout {
    id: u32,
    device_id: u32,
    layout: ash::vk::DescriptorSetLayout,
    bindings: Vec<DescriptorSetBinding>,
}

#[derive(Clone)]
pub struct DescriptorSetBinding {
    binding_type: DescriptorType,
    binding: ash::vk::DescriptorSetLayoutBinding<'static>,
}

#[derive(Debug)]
pub enum DescriptorError {
    PoolCreationError(ash::vk::Result),
    SetCreationError(ash::vk::Result),
    SetCleanupError(ash::vk::Result),
    SetLayoutCreationError(ash::vk::Result),
    BindingDoesntExist,
    DeviceMismatch,
}

impl From<DescriptorError> for VulkanError {
    fn from(value: DescriptorError) -> Self {
        VulkanError::DescriptorError(value)
    }
}

pub type DescriptorType = ash::vk::DescriptorType;
pub type DescriptorStage = ash::vk::ShaderStageFlags;

impl<'a> DescriptorSetBinding {
    pub fn new(
        binding: u32,
        descriptor_type: DescriptorType,
        descriptor_stage: DescriptorStage,
    ) -> DescriptorSetBinding {
        let binding = ash::vk::DescriptorSetLayoutBinding::default()
            .binding(binding)
            .descriptor_count(1)
            .descriptor_type(descriptor_type)
            .stage_flags(descriptor_stage);

        DescriptorSetBinding {
            binding,
            binding_type: descriptor_type,
        }
    }
}

impl DescriptorPoolSize {
    pub fn new(
        descriptor_count: u32,
        descriptor_type: DescriptorType,
    ) -> DescriptorPoolSize {
        let size = ash::vk::DescriptorPoolSize::default()
            .descriptor_count(descriptor_count)
            .ty(descriptor_type);
        DescriptorPoolSize { size }
    }
}

impl DescriptorPool {
    const ID_COUNTER: AtomicU32 = AtomicU32::new(0);

    pub fn new(
        device: &Device,
        descriptor_max_count: u32,
        descriptor_sizes: &[DescriptorPoolSize],
    ) -> Result<DescriptorPool, DescriptorError> {
        let pool_sizes: Vec<_> =
            descriptor_sizes.iter().map(|x| x.size).collect();

        let create_info = ash::vk::DescriptorPoolCreateInfo::default()
            .flags(ash::vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET)
            .max_sets(descriptor_max_count)
            .pool_sizes(&pool_sizes);

        let id = Self::ID_COUNTER.load(Ordering::Acquire);
        Self::ID_COUNTER.store(id + 1, Ordering::Release);

        let pool = unsafe {
            device
                .get_device_raw()
                .create_descriptor_pool(&create_info, None)
                .map_err(|err| DescriptorError::PoolCreationError(err))?
        };

        Ok(DescriptorPool {
            id,
            device_id: device.id(),
            pool,
        })
    }

    pub fn get_pool_raw(&self) -> &ash::vk::DescriptorPool { &self.pool }

    pub fn id(&self) -> u32 { self.id }
}

impl<'a> DescriptorSetLayout {
    const ID_COUNTER: AtomicU32 = AtomicU32::new(0);

    pub fn new(
        device: &Device,
        bindings: Vec<DescriptorSetBinding>,
    ) -> Result<DescriptorSetLayout, DescriptorError> {
        let bindings_raw: Vec<_> = bindings.iter().map(|x| x.binding).collect();

        let create_info = ash::vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(&bindings_raw);

        let layout = match unsafe {
            device
                .get_device_raw()
                .create_descriptor_set_layout(&create_info, None)
        } {
            Ok(val) => val,
            Err(err) => {
                return Err(DescriptorError::SetLayoutCreationError(err));
            },
        };

        let id = Self::ID_COUNTER.load(Ordering::Acquire);
        Self::ID_COUNTER.store(id + 1, Ordering::Release);

        Ok(DescriptorSetLayout {
            id,
            device_id: device.id(),
            layout,
            bindings,
        })
    }

    pub fn get_layout_raw(&self) -> &ash::vk::DescriptorSetLayout {
        &self.layout
    }

    pub fn id(&self) -> u32 { self.id }

    pub fn device_id(&self) -> u32 { self.device_id }
}

impl DescriptorSet {
    const ID_COUNTER: AtomicU32 = AtomicU32::new(0);

    pub fn new(
        device: &Device,
        descriptor_layout: &DescriptorSetLayout,
        descriptor_pool: &DescriptorPool,
    ) -> Result<DescriptorSet, DescriptorError> {
        if device.id() != descriptor_layout.device_id
            || device.id() != descriptor_pool.device_id
        {
            return Err(DescriptorError::DeviceMismatch);
        }

        let layout_raw = [*descriptor_layout.get_layout_raw()];

        let create_info = ash::vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(*descriptor_pool.get_pool_raw())
            .set_layouts(&layout_raw);

        let set = match unsafe {
            device
                .get_device_raw()
                .allocate_descriptor_sets(&create_info)
        } {
            Ok(val) => val,
            Err(err) => return Err(DescriptorError::SetCreationError(err)),
        }[0];

        let id = Self::ID_COUNTER.load(Ordering::Acquire);
        Self::ID_COUNTER.store(id + 1, Ordering::Release);

        Ok(DescriptorSet {
            id,
            device_id: device.id(),
            descriptor_pool_id: descriptor_pool.id(),
            descriptor_layout_id: descriptor_layout.id(),
            set,
        })
    }

    pub fn bind_buffer(
        &self,
        buffer: &Buffer,
        device: &Device,
        descriptor_layout: &DescriptorSetLayout,
        dst_binding: u32,
    ) -> Result<(), DescriptorError> {
        if device.id() != descriptor_layout.device_id
            || device.id() != buffer.device_id()
        {
            return Err(DescriptorError::DeviceMismatch);
        }

        let buffer_info = [ash::vk::DescriptorBufferInfo::default()
            .buffer(*buffer.get_buffer_raw())
            .offset(0)
            .range(buffer.size)];

        let binding_type =
            match descriptor_layout.bindings.get(dst_binding as usize) {
                Some(val) => val.binding_type,
                None => return Err(DescriptorError::BindingDoesntExist),
            };

        let write_info = ash::vk::WriteDescriptorSet::default()
            .descriptor_type(binding_type)
            .dst_binding(dst_binding)
            .dst_set(self.set)
            .descriptor_count(descriptor_layout.bindings.len() as u32)
            .buffer_info(&buffer_info)
            .dst_array_element(0);

        unsafe {
            device
                .get_device_raw()
                .update_descriptor_sets(&[write_info], &[]);
        }

        Ok(())
    }

    pub fn bind_image_view(
        &self,
        view: &ImageView,
        device: &Device,
        descriptor_layout: &DescriptorSetLayout,
        dst_binding: u32,
    ) -> Result<(), DescriptorError> {
        let image_info = [ash::vk::DescriptorImageInfo::default()
            .image_view(*view.get_image_view_raw())];

        let binding_type =
            match descriptor_layout.bindings.get(dst_binding as usize) {
                Some(val) => val.binding_type,
                None => return Err(DescriptorError::BindingDoesntExist),
            };

        let write_info = ash::vk::WriteDescriptorSet::default()
            .descriptor_type(binding_type)
            .dst_binding(dst_binding)
            .dst_set(self.set)
            .descriptor_count(descriptor_layout.bindings.len() as u32)
            .image_info(&image_info)
            .dst_array_element(0);

        unsafe {
            device
                .get_device_raw()
                .update_descriptor_sets(&[write_info], &[]);
        }

        Ok(())
    }

    pub fn get_set_raw(&self) -> &ash::vk::DescriptorSet { &self.set }

    pub fn id(&self) -> u32 { self.id }

    pub fn device_id(&self) -> u32 { self.device_id }

    pub fn descriptor_pool_id(&self) -> u32 { self.descriptor_pool_id }

    pub fn descriptor_layout_id(&self) -> u32 { self.descriptor_layout_id }
}
