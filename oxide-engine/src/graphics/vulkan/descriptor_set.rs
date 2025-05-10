use super::{VulkanError, buffer::Buffer, device::Device};

pub struct DescriptorPool {
    pool: ash::vk::DescriptorPool,
}

pub struct DescriptorPoolSize {
    size: ash::vk::DescriptorPoolSize,
}

pub struct DescriptorSet {
    set: ash::vk::DescriptorSet,
    pub layout: DescriptorSetLayout,
}

#[derive(Clone)]
pub struct DescriptorSetLayout {
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
    pub fn new(descriptor_count: u32, descriptor_type: DescriptorType) -> DescriptorPoolSize {
        let size = ash::vk::DescriptorPoolSize::default()
            .descriptor_count(descriptor_count)
            .ty(descriptor_type);
        DescriptorPoolSize { size }
    }
}

impl DescriptorPool {
    pub fn new(
        device: &Device,
        descriptor_max_count: u32,
        descriptor_sizes: &[DescriptorPoolSize],
    ) -> Result<DescriptorPool, DescriptorError> {
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

        Ok(DescriptorPool { pool })
    }

    pub fn get_pool_raw(&self) -> &ash::vk::DescriptorPool {
        &self.pool
    }

    pub fn cleanup(&self, device: &Device) {
        unsafe {
            device
                .get_device_raw()
                .destroy_descriptor_pool(self.pool, None);
        }
    }
}

impl<'a> DescriptorSetLayout {
    pub fn new(
        device: &Device,
        bindings: Vec<DescriptorSetBinding>,
    ) -> Result<DescriptorSetLayout, DescriptorError> {
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

        Ok(DescriptorSetLayout { layout, bindings })
    }

    pub fn get_layout_raw(&self) -> &ash::vk::DescriptorSetLayout {
        &self.layout
    }

    pub fn cleanup(&self, device: &Device) {
        unsafe {
            device
                .get_device_raw()
                .destroy_descriptor_set_layout(self.layout, None);
        }
    }
}

impl DescriptorSet {
    pub fn new(
        device: &Device,
        descriptor_layout: DescriptorSetLayout,
        descriptor_pool: &DescriptorPool,
    ) -> Result<DescriptorSet, DescriptorError> {
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

        Ok(DescriptorSet {
            set,
            layout: descriptor_layout,
        })
    }

    pub fn bind_buffer<T: Clone + Copy>(
        &self,
        device: &Device,
        buffer: &Buffer<T>,
        dst_binding: u32,
    ) -> Result<(), DescriptorError> {
        let buffer_info = [ash::vk::DescriptorBufferInfo::default()
            .buffer(*buffer.get_buffer_raw())
            .offset(0)
            .range(size_of::<T>() as u64)];

        let binding_type = match self.layout.bindings.get(dst_binding as usize) {
            Some(val) => val.binding_type,
            None => return Err(DescriptorError::BindingDoesntExist),
        };

        let write_info = ash::vk::WriteDescriptorSet::default()
            .descriptor_type(binding_type)
            .dst_binding(dst_binding)
            .dst_set(self.set)
            .descriptor_count(self.layout.bindings.len() as u32)
            .buffer_info(&buffer_info)
            .dst_array_element(0);

        unsafe {
            device
                .get_device_raw()
                .update_descriptor_sets(&[write_info], &[]);
        }

        Ok(())
    }

    pub fn get_set_raw(&self) -> &ash::vk::DescriptorSet {
        &self.set
    }

    pub fn cleanup(
        &self,
        device: &Device,
        descriptor_pool: &DescriptorPool,
    ) -> Result<(), DescriptorError> {
        self.layout.cleanup(device);
        if let Err(err) = unsafe {
            device
                .get_device_raw()
                .free_descriptor_sets(*descriptor_pool.get_pool_raw(), &[self.set])
        } {
            return Err(DescriptorError::SetCleanupError(err));
        }
        Ok(())
    }
}
