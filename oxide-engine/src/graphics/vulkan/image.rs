use std::sync::atomic::{AtomicU32, Ordering};

use crate::graphics::{
    image::{ImageFormat, ImageUsage},
    vulkan::{
        VulkanError, buffer::find_memorytype_index, device::VulkanDevice,
        format_size::format_size,
    },
};

pub trait VulkanImage: Send + Sync + 'static {
    fn get_image_raw(&self) -> &ash::vk::Image;
    fn id(&self) -> u32;
    fn device_id(&self) -> u32;
    fn format(&self) -> ash::vk::Format;
    fn usage(&self) -> ash::vk::ImageUsageFlags;
    fn aspect_flags(&self) -> ash::vk::ImageAspectFlags;
}

#[derive(Debug, Clone)]
pub struct VulkanSwapchainImage {
    id: u32,
    device_id: u32,
    format: ash::vk::Format,
    image: ash::vk::Image,
}

#[allow(unused)]
pub struct VulkanOwnedImage {
    id: u32,
    device_id: u32,
    format: ash::vk::Format,
    usage: ash::vk::ImageUsageFlags,
    image: ash::vk::Image,
    memory: ash::vk::DeviceMemory,
    size: u64,
}

pub struct VulkanImageView {
    id: u32,
    image_id: u32,
    view: ash::vk::ImageView,
}

impl VulkanImage for VulkanSwapchainImage {
    fn get_image_raw(&self) -> &ash::vk::Image { &self.image }
    fn id(&self) -> u32 { self.id }
    fn device_id(&self) -> u32 { self.device_id }
    fn format(&self) -> ash::vk::Format { self.format }
    fn usage(&self) -> ash::vk::ImageUsageFlags { ash::vk::ImageUsageFlags::COLOR_ATTACHMENT }
    fn aspect_flags(&self) -> ash::vk::ImageAspectFlags {
        ash::vk::ImageAspectFlags::COLOR
    }
}

impl VulkanImage for VulkanOwnedImage {
    fn get_image_raw(&self) -> &ash::vk::Image { &self.image }
    fn id(&self) -> u32 { self.id }
    fn device_id(&self) -> u32 { self.device_id }
    fn format(&self) -> ash::vk::Format { self.format }
    fn usage(&self) -> ash::vk::ImageUsageFlags {
        self.usage
    }
    fn aspect_flags(&self) -> ash::vk::ImageAspectFlags {
        match self.usage {
            ash::vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT => ash::vk::ImageAspectFlags::DEPTH,
            _ => ash::vk::ImageAspectFlags::NONE
        }
    }
}

#[derive(Debug)]
pub enum VulkanImageError {
    CreationError(ash::vk::Result),
    ViewCreationError(ash::vk::Result),
    UnableToFindSuitableMemory,
    AllocationError(ash::vk::Result),
    BindError(ash::vk::Result),
    FormatNotSupported,
    PhysicalDeviceMismatch,
    DeviceMismatch,
}

impl From<VulkanImageError> for VulkanError {
    fn from(value: VulkanImageError) -> Self {
        VulkanError::VulkanImageError(value)
    }
}

pub type VulkanImageFormat = ash::vk::Format;

impl From<ImageFormat> for VulkanImageFormat {
    fn from(value: ImageFormat) -> Self {
        match value {
            ImageFormat::D32 => VulkanImageFormat::D32_SFLOAT,
            ImageFormat::RGB8SRGB => VulkanImageFormat::R8G8B8_SRGB,
            ImageFormat::RGBA8SRGB => VulkanImageFormat::R8G8B8A8_SRGB,
        }
    }
}

pub type VulkanImageUsage = ash::vk::ImageUsageFlags;

impl From<ImageUsage> for VulkanImageUsage {
    fn from(value: ImageUsage) -> Self {
        match value {
            ImageUsage::DepthBuffer => {
                VulkanImageUsage::DEPTH_STENCIL_ATTACHMENT
            },
        }
    }
}

pub type VulkanImageAspectFlags = ash::vk::ImageAspectFlags;

impl From<ImageUsage> for VulkanImageAspectFlags {
    fn from(value: ImageUsage) -> Self {
        match value {
            ImageUsage::DepthBuffer => VulkanImageAspectFlags::DEPTH,
        }
    }
}

pub struct VulkanImageViewInfo<'a> {
    image: &'a dyn VulkanImage,
    format: VulkanImageFormat,
    aspect_flags: VulkanImageAspectFlags,
}

#[derive(Debug, Clone, Copy)]
pub struct VulkanImageInfo {
    pub format: VulkanImageFormat,
    pub size: (u32, u32),
    pub usage: VulkanImageUsage,
}

impl<'a> From<VulkanImageViewInfo<'a>> for ash::vk::ImageViewCreateInfo<'_> {
    fn from(value: VulkanImageViewInfo) -> Self {
        ash::vk::ImageViewCreateInfo::default()
            .view_type(ash::vk::ImageViewType::TYPE_2D)
            .format(ash::vk::Format::from(value.format))
            .subresource_range(ash::vk::ImageSubresourceRange {
                aspect_mask: value.aspect_flags,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            })
            .components(ash::vk::ComponentMapping {
                r: ash::vk::ComponentSwizzle::R,
                g: ash::vk::ComponentSwizzle::G,
                b: ash::vk::ComponentSwizzle::B,
                a: ash::vk::ComponentSwizzle::A,
            })
            .image(*value.image.get_image_raw())
    }
}

impl From<VulkanImageInfo> for ash::vk::ImageCreateInfo<'_> {
    fn from(value: VulkanImageInfo) -> Self {
        ash::vk::ImageCreateInfo::default()
            .image_type(ash::vk::ImageType::TYPE_2D)
            .format(value.format)
            .extent(ash::vk::Extent3D {
                width: value.size.0,
                height: value.size.1,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .samples(ash::vk::SampleCountFlags::TYPE_1)
            .tiling(ash::vk::ImageTiling::OPTIMAL)
            .usage(value.usage)
            .sharing_mode(ash::vk::SharingMode::EXCLUSIVE)
    }
}

const ID_COUNTER: AtomicU32 = AtomicU32::new(0);

impl VulkanSwapchainImage {
    pub fn from_raw_image(
        device: &VulkanDevice,
        format: ash::vk::Format,
        raw_image: ash::vk::Image,
    ) -> VulkanSwapchainImage {
        let id = ID_COUNTER.load(Ordering::Acquire);
        ID_COUNTER.store(id + 1, Ordering::Release);

        VulkanSwapchainImage {
            id,
            device_id: device.id(),
            format,
            image: raw_image,
        }
    }
}

impl VulkanOwnedImage {
    pub fn new(
        device: &VulkanDevice,
        create_info: VulkanImageInfo,
    ) -> Result<VulkanOwnedImage, VulkanImageError> {
        let size = create_info.size;
        let format = create_info.format;

        let image = match unsafe {
            device
                .get_device_raw()
                .create_image(&create_info.into(), None)
        } {
            Ok(val) => val,
            Err(err) => return Err(VulkanImageError::CreationError(err)),
        };
        let memory_req = unsafe {
            device.get_device_raw().get_image_memory_requirements(image)
        };
        let memory_index = match find_memorytype_index(
            &memory_req,
            ash::vk::MemoryPropertyFlags::DEVICE_LOCAL,
            device,
        ) {
            Some(val) => val,
            None => return Err(VulkanImageError::UnableToFindSuitableMemory),
        };

        let image_allocate_info = ash::vk::MemoryAllocateInfo::default()
            .allocation_size(memory_req.size)
            .memory_type_index(memory_index);

        let image_memory = match unsafe {
            device
                .get_device_raw()
                .allocate_memory(&image_allocate_info, None)
        } {
            Ok(val) => val,
            Err(err) => return Err(VulkanImageError::AllocationError(err)),
        };

        if let Err(err) = unsafe {
            device
                .get_device_raw()
                .bind_image_memory(image, image_memory, 0)
        } {
            return Err(VulkanImageError::BindError(err));
        };

        let format_size = match format_size(format) {
            Some(val) => val as u64,
            None => return Err(VulkanImageError::FormatNotSupported),
        };

        let id = ID_COUNTER.load(Ordering::Acquire);
        ID_COUNTER.store(id + 1, Ordering::Release);

        Ok(VulkanOwnedImage {
            id,
            device_id: device.id(),
            image,
            format,
            usage: create_info.usage,
            memory: image_memory,
            size: format_size * size.0 as u64 * size.1 as u64,
        })
    }

    pub fn get_memory_raw(&self) -> &ash::vk::DeviceMemory { &self.memory }
}

impl VulkanImageView {
    const ID_COUNTER: AtomicU32 = AtomicU32::new(0);

    pub fn from_image(
        device: &VulkanDevice,
        image: &impl VulkanImage,
    ) -> Result<VulkanImageView, VulkanImageError> {
        let image_id = image.id();
        let view_info = VulkanImageViewInfo {
            image,
            format: image.format(),
            aspect_flags: image.aspect_flags(),
        };

        let id = Self::ID_COUNTER.load(Ordering::Acquire);
        Self::ID_COUNTER.store(id + 1, Ordering::Release);

        match unsafe {
            device
                .get_device_raw()
                .create_image_view(&view_info.into(), None)
        } {
            Ok(val) => Ok(VulkanImageView {
                id,
                image_id,
                view: val,
            }),
            Err(err) => Err(VulkanImageError::ViewCreationError(err)),
        }
    }

    pub fn get_image_view_raw(&self) -> &ash::vk::ImageView { &self.view }

    pub fn id(&self) -> u32 { self.id }

    pub fn image_id(&self) -> u32 { self.image_id }
}
