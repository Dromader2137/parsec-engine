use crate::{
    graphics::{
        image::{ImageFormat, ImageUsage},
        vulkan::{
            buffer::find_memorytype_index, device::VulkanDevice,
            format_size::format_size,
        },
    },
    utils::id_counter::IdCounter,
};

pub trait VulkanImage: Send + Sync + 'static {
    fn get_image_raw(&self) -> &ash::vk::Image;
    fn id(&self) -> u32;
    fn _device_id(&self) -> u32;
    fn format(&self) -> ash::vk::Format;
    fn _usage(&self) -> ash::vk::ImageUsageFlags;
    fn aspect_flags(&self) -> ash::vk::ImageAspectFlags;
}

#[derive(Debug, Clone)]
pub struct VulkanSwapchainImage {
    id: u32,
    _device_id: u32,
    format: ash::vk::Format,
    image: ash::vk::Image,
}

#[allow(unused)]
pub struct VulkanOwnedImage {
    id: u32,
    device_id: u32,
    extent: ash::vk::Extent3D,
    format: ash::vk::Format,
    usage: ash::vk::ImageUsageFlags,
    aspect: ash::vk::ImageAspectFlags,
    image: ash::vk::Image,
    memory: ash::vk::DeviceMemory,
    size: u64,
}

#[derive(Debug)]
pub struct VulkanImageView {
    id: u32,
    _image_id: u32,
    device_id: u32,
    _format: ash::vk::Format,
    view: ash::vk::ImageView,
}

impl VulkanImage for VulkanSwapchainImage {
    fn get_image_raw(&self) -> &ash::vk::Image { &self.image }
    fn id(&self) -> u32 { self.id }
    fn _device_id(&self) -> u32 { self._device_id }
    fn format(&self) -> ash::vk::Format { self.format }
    fn _usage(&self) -> ash::vk::ImageUsageFlags {
        ash::vk::ImageUsageFlags::COLOR_ATTACHMENT
    }
    fn aspect_flags(&self) -> ash::vk::ImageAspectFlags {
        ash::vk::ImageAspectFlags::COLOR
    }
}

impl VulkanImage for VulkanOwnedImage {
    fn get_image_raw(&self) -> &ash::vk::Image { &self.image }
    fn id(&self) -> u32 { self.id }
    fn _device_id(&self) -> u32 { self.device_id }
    fn format(&self) -> ash::vk::Format { self.format }
    fn _usage(&self) -> ash::vk::ImageUsageFlags { self.usage }
    fn aspect_flags(&self) -> ash::vk::ImageAspectFlags { self.aspect }
}

#[derive(Debug, thiserror::Error)]
pub enum VulkanImageError {
    #[error("Failed to create image: {0}")]
    CreationError(ash::vk::Result),
    #[error("Failed to create image view: {0}")]
    ViewCreationError(ash::vk::Result),
    #[error("Unable to find suitable memory for image allocation")]
    UnableToFindSuitableMemory,
    #[error("Failed to allocate memory: {0}")]
    AllocationError(ash::vk::Result),
    #[error("Failed to bind memory to image: {0}")]
    BindError(ash::vk::Result),
    #[error("Image format not supported")]
    FormatNotSupported,
    #[error("Image created on different device")]
    DeviceMismatch,
}

pub type VulkanImageFormat = ash::vk::Format;
pub type VulkanImageLayout = ash::vk::ImageLayout;
pub type VulkanImageUsage = ash::vk::ImageUsageFlags;
pub type VulkanImageAspectFlags = ash::vk::ImageAspectFlags;

impl From<ImageFormat> for VulkanImageFormat {
    fn from(value: ImageFormat) -> Self {
        match value {
            ImageFormat::D32 => VulkanImageFormat::D32_SFLOAT,
            ImageFormat::R8SRGB => VulkanImageFormat::R8_SRGB,
            ImageFormat::RG8SRGB => VulkanImageFormat::R8G8_SRGB,
            ImageFormat::RGB8SRGB => VulkanImageFormat::R8G8B8_SRGB,
            ImageFormat::RGBA8SRGB => VulkanImageFormat::R8G8B8A8_SRGB,
            ImageFormat::RGBA8UNORM => VulkanImageFormat::R8G8B8A8_UNORM,
        }
    }
}

impl From<ImageUsage> for VulkanImageUsage {
    fn from(value: ImageUsage) -> Self {
        match value {
            ImageUsage::DepthAttachment => {
                VulkanImageUsage::DEPTH_STENCIL_ATTACHMENT
            },
            ImageUsage::Sampled => VulkanImageUsage::SAMPLED,
            ImageUsage::Src => VulkanImageUsage::TRANSFER_SRC,
            ImageUsage::Dst => VulkanImageUsage::TRANSFER_DST,
            ImageUsage::ColorAttachment => VulkanImageUsage::COLOR_ATTACHMENT,
            _ => VulkanImageUsage::empty(),
        }
    }
}

impl From<ImageUsage> for VulkanImageAspectFlags {
    fn from(value: ImageUsage) -> Self {
        match value {
            ImageUsage::DepthAttachment => VulkanImageAspectFlags::DEPTH,
            ImageUsage::ColorAttachment => VulkanImageAspectFlags::COLOR,
            ImageUsage::ColorBuffer => VulkanImageAspectFlags::COLOR,
            _ => VulkanImageAspectFlags::empty(),
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
    pub aspect: VulkanImageAspectFlags,
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

static ID_COUNTER_IMAGE: once_cell::sync::Lazy<IdCounter> =
    once_cell::sync::Lazy::new(|| IdCounter::new(0));
impl VulkanSwapchainImage {
    pub fn from_raw_image(
        device: &VulkanDevice,
        format: ash::vk::Format,
        raw_image: ash::vk::Image,
    ) -> VulkanSwapchainImage {
        VulkanSwapchainImage {
            id: ID_COUNTER_IMAGE.next(),
            _device_id: device.id(),
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

        Ok(VulkanOwnedImage {
            id: ID_COUNTER_IMAGE.next(),
            device_id: device.id(),
            extent: ash::vk::Extent3D {
                width: create_info.size.0,
                height: create_info.size.1,
                depth: 1,
            },
            image,
            format,
            usage: create_info.usage,
            aspect: create_info.aspect,
            memory: image_memory,
            size: format_size * size.0 as u64 * size.1 as u64,
        })
    }

    pub fn delete_image(
        self,
        device: &VulkanDevice,
    ) -> Result<(), VulkanImageError> {
        if self.device_id != device.id() {
            return Err(VulkanImageError::DeviceMismatch);
        }

        unsafe {
            device
                .get_device_raw()
                .destroy_image(*self.get_image_raw(), None);
            device
                .get_device_raw()
                .free_memory(*self.get_memory_raw(), None);
        }
        Ok(())
    }

    pub fn get_memory_raw(&self) -> &ash::vk::DeviceMemory { &self.memory }

    pub fn device_id(&self) -> u32 { self.device_id }

    pub fn extent(&self) -> ash::vk::Extent3D { self.extent }
}

static ID_COUNTER_VIEW: once_cell::sync::Lazy<IdCounter> =
    once_cell::sync::Lazy::new(|| IdCounter::new(0));
impl VulkanImageView {
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

        match unsafe {
            device
                .get_device_raw()
                .create_image_view(&view_info.into(), None)
        } {
            Ok(val) => Ok(VulkanImageView {
                id: ID_COUNTER_VIEW.next(),
                _image_id: image_id,
                _format: image.format(),
                device_id: device.id(),
                view: val,
            }),
            Err(err) => Err(VulkanImageError::ViewCreationError(err)),
        }
    }

    pub fn delete_image_view(
        self,
        device: &VulkanDevice,
    ) -> Result<(), VulkanImageError> {
        if self.device_id != device.id() {
            return Err(VulkanImageError::DeviceMismatch);
        }

        unsafe {
            device
                .get_device_raw()
                .destroy_image_view(*self.get_image_view_raw(), None);
        }
        Ok(())
    }

    pub fn get_image_view_raw(&self) -> &ash::vk::ImageView { &self.view }

    pub fn id(&self) -> u32 { self.id }

    pub fn _image_id(&self) -> u32 { self._image_id }

    pub fn _format(&self) -> ash::vk::Format { self._format }
}
