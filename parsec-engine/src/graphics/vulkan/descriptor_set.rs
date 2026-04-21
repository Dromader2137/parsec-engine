use crate::graphics::{
    pipeline::PipelineBindingType,
    vulkan::{
        buffer::VulkanBuffer, device::VulkanDevice,
        graphics_pipeline::VulkanShaderStage, image::VulkanImageView,
        sampler::VulkanSampler,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(unused)]
pub enum VulkanDescriptorType {
    CombinedImageSampler,
    UniformBuffer,
    StorageBuffer,
}

impl VulkanDescriptorType {
    pub fn new(value: PipelineBindingType) -> Self {
        match value {
            PipelineBindingType::UniformBuffer => Self::UniformBuffer,
            PipelineBindingType::TextureSampler => Self::CombinedImageSampler,
            PipelineBindingType::StorageBuffer => Self::StorageBuffer,
        }
    }

    pub fn raw_descriptor_type(&self) -> ash::vk::DescriptorType {
        match self {
            VulkanDescriptorType::CombinedImageSampler => {
                ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER
            },
            VulkanDescriptorType::UniformBuffer => {
                ash::vk::DescriptorType::UNIFORM_BUFFER
            },
            VulkanDescriptorType::StorageBuffer => {
                ash::vk::DescriptorType::STORAGE_BUFFER
            },
        }
    }
}

pub struct VulkanDescriptorPool {
    pool: ash::vk::DescriptorPool,
}

pub struct VulkanDescriptorPoolSize {
    size: ash::vk::DescriptorPoolSize,
}

#[derive(Debug, Default)]
enum DescriptorSetBinding {
    #[default]
    None,
    UniformBuffer(Option<u32>),
    StorageBuffer(Option<u32>),
    CombinedSampler(Option<(u32, u32)>),
}

pub struct VulkanDescriptorSet {
    id: u32,
    descriptor_layout_id: u32,
    bindings_state: Vec<DescriptorSetBinding>,
    set: ash::vk::DescriptorSet,
}

#[derive(Clone)]
pub struct VulkanDescriptorSetLayout {
    id: u32,
    layout: ash::vk::DescriptorSetLayout,
    bindings: Vec<VulkanDescriptorSetBinding>,
}

#[derive(Clone)]
pub struct VulkanDescriptorSetBinding {
    binding_type: VulkanDescriptorType,
    binding: ash::vk::DescriptorSetLayoutBinding<'static>,
}

#[derive(Debug, thiserror::Error)]
pub enum VulkanDescriptorError {
    #[error("Failed to create a descriptor pool: {0}")]
    PoolCreationError(ash::vk::Result),
    #[error("Failed to create a descriptor set: {0}")]
    SetCreationError(ash::vk::Result),
    #[error("Failed to delete a descriptor set layout: {0}")]
    SetLayoutCreationError(ash::vk::Result),
    #[error("Failed to bind: binding doesn't exist")]
    BindingDoesntExist,
    #[error("Failed to bind: invalid binding type for this operation")]
    InvalidBindingType,
    #[error("Failed to free a descriptor set: {0}")]
    SetDeletionError(ash::vk::Result),
}

impl<'a> VulkanDescriptorSetBinding {
    pub fn new(
        binding: u32,
        descriptor_type: VulkanDescriptorType,
        descriptor_stage: &[VulkanShaderStage],
    ) -> VulkanDescriptorSetBinding {
        let binding = ash::vk::DescriptorSetLayoutBinding::default()
            .binding(binding)
            .descriptor_count(1)
            .descriptor_type(descriptor_type.raw_descriptor_type())
            .stage_flags(VulkanShaderStage::raw_combined_shader_stage(
                descriptor_stage,
            ));

        VulkanDescriptorSetBinding {
            binding,
            binding_type: descriptor_type,
        }
    }

    pub fn binding_type(&self) -> VulkanDescriptorType { self.binding_type }
}

impl VulkanDescriptorPoolSize {
    pub fn new(
        descriptor_count: u32,
        descriptor_type: VulkanDescriptorType,
    ) -> VulkanDescriptorPoolSize {
        let size = ash::vk::DescriptorPoolSize::default()
            .ty(descriptor_type.raw_descriptor_type())
            .descriptor_count(descriptor_count);
        VulkanDescriptorPoolSize { size }
    }
}

impl VulkanDescriptorPool {
    pub fn new(
        device: &VulkanDevice,
        descriptor_max_count: u32,
        descriptor_sizes: &[VulkanDescriptorPoolSize],
    ) -> Result<VulkanDescriptorPool, VulkanDescriptorError> {
        let pool_sizes: Vec<_> =
            descriptor_sizes.iter().map(|x| x.size).collect();

        let create_info = ash::vk::DescriptorPoolCreateInfo::default()
            .flags(ash::vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET)
            .max_sets(descriptor_max_count)
            .pool_sizes(&pool_sizes);

        let pool = unsafe {
            device
                .raw_device()
                .create_descriptor_pool(&create_info, None)
                .map_err(|err| VulkanDescriptorError::PoolCreationError(err))?
        };

        Ok(VulkanDescriptorPool { pool })
    }

    pub fn destroy(&self, device: &VulkanDevice) {
        unsafe { device.raw_device().destroy_descriptor_pool(self.pool, None) }
    }

    pub fn raw(&self) -> &ash::vk::DescriptorPool { &self.pool }
}

crate::create_counter! {ID_COUNTER_LAYOUT}
impl<'a> VulkanDescriptorSetLayout {
    pub fn new(
        device: &VulkanDevice,
        bindings: Vec<VulkanDescriptorSetBinding>,
    ) -> Result<VulkanDescriptorSetLayout, VulkanDescriptorError> {
        let bindings_raw: Vec<_> = bindings.iter().map(|x| x.binding).collect();

        let create_info = ash::vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(&bindings_raw);

        let layout = match unsafe {
            device
                .raw_device()
                .create_descriptor_set_layout(&create_info, None)
        } {
            Ok(val) => val,
            Err(err) => {
                return Err(VulkanDescriptorError::SetLayoutCreationError(err));
            },
        };

        Ok(VulkanDescriptorSetLayout {
            id: ID_COUNTER_LAYOUT.next(),
            layout,
            bindings,
        })
    }

    pub fn destroy(self, device: &VulkanDevice) {
        unsafe {
            device
                .raw_device()
                .destroy_descriptor_set_layout(self.layout, None)
        }
    }

    pub fn get_layout_raw(&self) -> &ash::vk::DescriptorSetLayout {
        &self.layout
    }

    pub fn id(&self) -> u32 { self.id }

    pub fn bindings(&self) -> &[VulkanDescriptorSetBinding] { &self.bindings }
}

crate::create_counter! {ID_COUNTER_SET}
impl VulkanDescriptorSet {
    pub fn new(
        device: &VulkanDevice,
        descriptor_layout: &VulkanDescriptorSetLayout,
        descriptor_pool: &VulkanDescriptorPool,
    ) -> Result<VulkanDescriptorSet, VulkanDescriptorError> {
        let layout_raw = [*descriptor_layout.get_layout_raw()];

        let create_info = ash::vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(*descriptor_pool.raw())
            .set_layouts(&layout_raw);

        let set = match unsafe {
            device.raw_device().allocate_descriptor_sets(&create_info)
        } {
            Ok(val) => val,
            Err(err) => {
                return Err(VulkanDescriptorError::SetCreationError(err));
            },
        }[0];

        let bindings = descriptor_layout
            .bindings()
            .iter()
            .map(|x| match x.binding_type() {
                VulkanDescriptorType::CombinedImageSampler => {
                    DescriptorSetBinding::CombinedSampler(None)
                },
                VulkanDescriptorType::UniformBuffer => {
                    DescriptorSetBinding::UniformBuffer(None)
                },
                VulkanDescriptorType::StorageBuffer => {
                    DescriptorSetBinding::StorageBuffer(None)
                },
            })
            .collect();

        Ok(VulkanDescriptorSet {
            id: ID_COUNTER_SET.next(),
            descriptor_layout_id: descriptor_layout.id(),
            bindings_state: bindings,
            set,
        })
    }

    pub fn bind_buffer(
        &mut self,
        buffer: &VulkanBuffer,
        device: &VulkanDevice,
        descriptor_layout: &VulkanDescriptorSetLayout,
        dst_binding: u32,
    ) -> Result<(), VulkanDescriptorError> {
        let binding = match self
            .bindings_state
            .get_mut(dst_binding as usize)
            .ok_or(VulkanDescriptorError::BindingDoesntExist)?
        {
            DescriptorSetBinding::None => {
                Err(VulkanDescriptorError::BindingDoesntExist)
            },
            DescriptorSetBinding::UniformBuffer(buffer) => Ok(buffer),
            DescriptorSetBinding::StorageBuffer(buffer) => Ok(buffer),
            DescriptorSetBinding::CombinedSampler(_) => {
                Err(VulkanDescriptorError::InvalidBindingType)
            },
        }?;

        let buffer_info = [ash::vk::DescriptorBufferInfo::default()
            .buffer(*buffer.get_buffer_raw())
            .offset(0)
            .range(buffer.size())];

        let binding_type =
            match descriptor_layout.bindings().get(dst_binding as usize) {
                Some(val) => val.binding_type(),
                None => return Err(VulkanDescriptorError::BindingDoesntExist),
            };

        let write_info = ash::vk::WriteDescriptorSet::default()
            .descriptor_type(binding_type.raw_descriptor_type())
            .dst_binding(dst_binding)
            .dst_set(self.set)
            .descriptor_count(descriptor_layout.bindings().len() as u32)
            .buffer_info(&buffer_info)
            .dst_array_element(0);

        unsafe {
            device
                .raw_device()
                .update_descriptor_sets(&[write_info], &[]);
        }

        binding.replace(buffer.id());

        Ok(())
    }

    pub fn bind_image_view(
        &mut self,
        image_view: &VulkanImageView,
        sampler: &VulkanSampler,
        device: &VulkanDevice,
        descriptor_layout: &VulkanDescriptorSetLayout,
        dst_binding: u32,
    ) -> Result<(), VulkanDescriptorError> {
        let binding = match self
            .bindings_state
            .get_mut(dst_binding as usize)
            .ok_or(VulkanDescriptorError::BindingDoesntExist)?
        {
            DescriptorSetBinding::None => {
                Err(VulkanDescriptorError::BindingDoesntExist)
            },
            DescriptorSetBinding::UniformBuffer(_)
            | DescriptorSetBinding::StorageBuffer(_) => {
                Err(VulkanDescriptorError::InvalidBindingType)
            },
            DescriptorSetBinding::CombinedSampler(val) => Ok(val),
        }?;

        let image_info = [ash::vk::DescriptorImageInfo::default()
            .image_view(*image_view.raw_image_view())
            .sampler(sampler.sampler_raw())
            .image_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)];

        let binding_type =
            match descriptor_layout.bindings().get(dst_binding as usize) {
                Some(val) => val.binding_type(),
                None => return Err(VulkanDescriptorError::BindingDoesntExist),
            };

        let write_info = ash::vk::WriteDescriptorSet::default()
            .descriptor_type(binding_type.raw_descriptor_type())
            .dst_binding(dst_binding)
            .dst_set(self.set)
            .descriptor_count(descriptor_layout.bindings().len() as u32)
            .image_info(&image_info)
            .dst_array_element(0);

        unsafe {
            device
                .raw_device()
                .update_descriptor_sets(&[write_info], &[]);
        }

        binding.replace((image_view.id(), sampler.id()));

        Ok(())
    }

    pub fn raw_descriptor_set(&self) -> &ash::vk::DescriptorSet { &self.set }

    pub fn id(&self) -> u32 { self.id }

    pub fn descriptor_layout_id(&self) -> u32 { self.descriptor_layout_id }

    pub fn bound_image_ids(&self) -> Vec<u32> {
        self.bindings_state
            .iter()
            .filter_map(|x| {
                if let DescriptorSetBinding::CombinedSampler(y) = x {
                    y.map(|x| x.0)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    }

    pub fn free(
        self,
        device: &VulkanDevice,
        descriptor_pool: &VulkanDescriptorPool,
    ) -> Result<(), VulkanDescriptorError> {
        unsafe {
            device
                .raw_device()
                .free_descriptor_sets(descriptor_pool.pool, &[self.set])
                .map_err(|err| VulkanDescriptorError::SetDeletionError(err))
        }
    }
}
