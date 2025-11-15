use std::sync::Arc;

use crate::graphics::vulkan::{VulkanError, buffer::Buffer, device::Device, image::ImageView};

pub struct DescriptorPool {
    pub device: Arc<Device>,
    pool: ash::vk::DescriptorPool,
}

pub struct DescriptorPoolSize {
    size: ash::vk::DescriptorPoolSize,
}

pub struct DescriptorSet {
    pub descriptor_pool: Arc<DescriptorPool>,
    pub descriptor_layout: Arc<DescriptorSetLayout>,
    set: ash::vk::DescriptorSet,
}

#[derive(Clone)]
pub struct DescriptorSetLayout {
    pub device: Arc<Device>,
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
}

impl From<DescriptorError> for VulkanError {
    fn from(value: DescriptorError) -> Self { VulkanError::DescriptorError(value) }
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
    pub fn new(descriptor_count: u32, descriptor_type: DescriptorType) -> DescriptorPoolSize {
        let size = ash::vk::DescriptorPoolSize::default()
            .descriptor_count(descriptor_count)
            .ty(descriptor_type);
        DescriptorPoolSize { size }
    }
}

impl DescriptorPool {
    pub fn new(
        device: Arc<Device>,
        descriptor_max_count: u32,
        descriptor_sizes: &[DescriptorPoolSize],
    ) -> Result<Arc<DescriptorPool>, DescriptorError> {
        let pool_sizes: Vec<_> = descriptor_sizes.iter().map(|x| x.size).collect();

        let create_info = ash::vk::DescriptorPoolCreateInfo::default()
            .flags(ash::vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET)
            .max_sets(descriptor_max_count)
            .pool_sizes(&pool_sizes);

        let pool = match unsafe {
            device
                .get_device_raw()
                .create_descriptor_pool(&create_info, None)
        } {
            Ok(val) => val,
            Err(err) => return Err(DescriptorError::PoolCreationError(err)),
        };

        Ok(Arc::new(DescriptorPool { device, pool }))
    }

    pub fn get_pool_raw(&self) -> &ash::vk::DescriptorPool { &self.pool }
}

impl Drop for DescriptorPool {
    fn drop(&mut self) {
        unsafe {
            self.device
                .get_device_raw()
                .destroy_descriptor_pool(self.pool, None);
        }
    }
}

impl<'a> DescriptorSetLayout {
    pub fn new(
        device: Arc<Device>,
        bindings: Vec<DescriptorSetBinding>,
    ) -> Result<Arc<DescriptorSetLayout>, DescriptorError> {
        let bindings_raw: Vec<_> = bindings.iter().map(|x| x.binding).collect();

        let create_info = ash::vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings_raw);

        let layout = match unsafe {
            device
                .get_device_raw()
                .create_descriptor_set_layout(&create_info, None)
        } {
            Ok(val) => val,
            Err(err) => return Err(DescriptorError::SetLayoutCreationError(err)),
        };

        Ok(Arc::new(DescriptorSetLayout {
            device,
            layout,
            bindings,
        }))
    }

    pub fn get_layout_raw(&self) -> &ash::vk::DescriptorSetLayout { &self.layout }
}

impl Drop for DescriptorSetLayout {
    fn drop(&mut self) {
        unsafe {
            self.device
                .get_device_raw()
                .destroy_descriptor_set_layout(self.layout, None);
        }
    }
}

impl DescriptorSet {
    pub fn new(
        descriptor_layout: Arc<DescriptorSetLayout>,
        descriptor_pool: Arc<DescriptorPool>,
    ) -> Result<Arc<DescriptorSet>, DescriptorError> {
        let layout_raw = [*descriptor_layout.get_layout_raw()];

        let create_info = ash::vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(*descriptor_pool.get_pool_raw())
            .set_layouts(&layout_raw);

        let set = match unsafe {
            descriptor_pool
                .device
                .get_device_raw()
                .allocate_descriptor_sets(&create_info)
        } {
            Ok(val) => val,
            Err(err) => return Err(DescriptorError::SetCreationError(err)),
        }[0];

        Ok(Arc::new(DescriptorSet {
            descriptor_pool,
            descriptor_layout,
            set,
        }))
    }

    pub fn bind_buffer(
        &self,
        buffer: Arc<Buffer>,
        dst_binding: u32,
    ) -> Result<(), DescriptorError> {
        let buffer_info = [ash::vk::DescriptorBufferInfo::default()
            .buffer(*buffer.get_buffer_raw())
            .offset(0)
            .range(buffer.size)];

        let binding_type = match self.descriptor_layout.bindings.get(dst_binding as usize) {
            Some(val) => val.binding_type,
            None => return Err(DescriptorError::BindingDoesntExist),
        };

        let write_info = ash::vk::WriteDescriptorSet::default()
            .descriptor_type(binding_type)
            .dst_binding(dst_binding)
            .dst_set(self.set)
            .descriptor_count(self.descriptor_layout.bindings.len() as u32)
            .buffer_info(&buffer_info)
            .dst_array_element(0);

        unsafe {
            self.descriptor_pool
                .device
                .get_device_raw()
                .update_descriptor_sets(&[write_info], &[]);
        }

        Ok(())
    }

    pub fn bind_image_view(
        &self,
        view: Arc<ImageView>,
        dst_binding: u32,
    ) -> Result<(), DescriptorError> {
        let image_info =
            [ash::vk::DescriptorImageInfo::default().image_view(*view.get_image_view_raw())];

        let binding_type = match self.descriptor_layout.bindings.get(dst_binding as usize) {
            Some(val) => val.binding_type,
            None => return Err(DescriptorError::BindingDoesntExist),
        };

        let write_info = ash::vk::WriteDescriptorSet::default()
            .descriptor_type(binding_type)
            .dst_binding(dst_binding)
            .dst_set(self.set)
            .descriptor_count(self.descriptor_layout.bindings.len() as u32)
            .image_info(&image_info)
            .dst_array_element(0);

        unsafe {
            self.descriptor_pool
                .device
                .get_device_raw()
                .update_descriptor_sets(&[write_info], &[]);
        }

        Ok(())
    }

    pub fn get_set_raw(&self) -> &ash::vk::DescriptorSet { &self.set }
}

impl Drop for DescriptorSet {
    fn drop(&mut self) {
        unsafe {
            self.descriptor_pool
                .device
                .get_device_raw()
                .free_descriptor_sets(*self.descriptor_pool.get_pool_raw(), &[self.set])
                .unwrap_or(())
        }
    }
}
