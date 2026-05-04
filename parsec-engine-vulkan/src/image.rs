use parsec_engine_graphics::image::{ImageAspect, ImageFormat, ImageUsage};
use parsec_engine_math::uvec::Vec2u;
use parsec_engine_utils::create_counter;

use crate::{
    allocation::VulkanAllocationError,
    allocator::{
        VulkanAllocator, VulkanMemoryProperties, VulkanMemoryRequirements,
    },
    device::VulkanDevice,
    format_size::format_size,
    memory::VulkanMemory,
    utils::raw_extent_2d,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulkanImageAspect {
    Color,
    Depth,
}

impl VulkanImageAspect {
    pub fn new(value: ImageAspect) -> Self {
        match value {
            ImageAspect::Color => Self::Color,
            ImageAspect::Depth => Self::Depth,
        }
    }

    pub fn raw_image_aspect(&self) -> ash::vk::ImageAspectFlags {
        match self {
            VulkanImageAspect::Color => ash::vk::ImageAspectFlags::COLOR,
            VulkanImageAspect::Depth => ash::vk::ImageAspectFlags::DEPTH,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulkanImageUsage {
    TransferSrc,
    TransferDst,
    Sampled,
    ColorAttachment,
    DepthStencilAttachment,
}

impl VulkanImageUsage {
    pub fn new(value: ImageUsage) -> Self {
        match value {
            ImageUsage::TransferSrc => Self::TransferSrc,
            ImageUsage::TransferDst => Self::TransferDst,
            ImageUsage::Sampled => Self::Sampled,
            ImageUsage::ColorAttachment => Self::ColorAttachment,
            ImageUsage::DepthAttachment => Self::DepthStencilAttachment,
        }
    }

    fn raw_image_usage(&self) -> ash::vk::ImageUsageFlags {
        match self {
            VulkanImageUsage::TransferSrc => {
                ash::vk::ImageUsageFlags::TRANSFER_SRC
            },
            VulkanImageUsage::TransferDst => {
                ash::vk::ImageUsageFlags::TRANSFER_DST
            },
            VulkanImageUsage::Sampled => ash::vk::ImageUsageFlags::SAMPLED,
            VulkanImageUsage::ColorAttachment => {
                ash::vk::ImageUsageFlags::COLOR_ATTACHMENT
            },
            VulkanImageUsage::DepthStencilAttachment => {
                ash::vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT
            },
        }
    }

    fn raw_combined_image_usage(usage: &[Self]) -> ash::vk::ImageUsageFlags {
        usage
            .iter()
            .fold(ash::vk::ImageUsageFlags::empty(), |acc, v| {
                acc | v.raw_image_usage()
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulkanImageFormat {
    RGBA8SRGB,
    RGB8SRGB,
    RG8SRGB,
    R8SRGB,
    D32,
    RGBA8UNORM,
    BGRA8SRGB,
    R8UNORM,
    RG8UNORM,
    RGB8UNORM,
    BGRA8UNORM,
    R16UNORM,
    RG16UNORM,
    RGB16UNORM,
    RGBA16UNORM,
    R16SNORM,
    RG16SNORM,
    RGB16SNORM,
    RGBA16SNORM,
}

impl VulkanImageFormat {
    pub fn new(value: ImageFormat) -> Self {
        match value {
            ImageFormat::D32 => Self::D32,
            ImageFormat::R8SRGB => Self::R8SRGB,
            ImageFormat::RG8SRGB => Self::RG8SRGB,
            ImageFormat::RGB8SRGB => Self::RGB8SRGB,
            ImageFormat::RGBA8SRGB => Self::RGBA8SRGB,
            ImageFormat::BGRA8SRGB => Self::BGRA8SRGB,
            ImageFormat::R8UNORM => Self::R8UNORM,
            ImageFormat::RG8UNORM => Self::RG8UNORM,
            ImageFormat::RGB8UNORM => Self::RGB8UNORM,
            ImageFormat::RGBA8UNORM => Self::RGBA8UNORM,
            ImageFormat::BGRA8UNORM => Self::BGRA8UNORM,
            ImageFormat::R16UNORM => Self::R16UNORM,
            ImageFormat::RG16UNORM => Self::RG16UNORM,
            ImageFormat::RGB16UNORM => Self::RGB16UNORM,
            ImageFormat::RGBA16UNORM => Self::RGBA16UNORM,
            ImageFormat::R16SNORM => Self::R16SNORM,
            ImageFormat::RG16SNORM => Self::RG16SNORM,
            ImageFormat::RGB16SNORM => Self::RGB16SNORM,
            ImageFormat::RGBA16SNORM => Self::RGBA16SNORM,
        }
    }

    pub fn raw_image_format(&self) -> ash::vk::Format {
        match self {
            VulkanImageFormat::RGBA8SRGB => ash::vk::Format::R8G8B8A8_SRGB,
            VulkanImageFormat::RGB8SRGB => ash::vk::Format::R8G8B8_SRGB,
            VulkanImageFormat::RG8SRGB => ash::vk::Format::R8G8_SRGB,
            VulkanImageFormat::R8SRGB => ash::vk::Format::R8_SRGB,
            VulkanImageFormat::D32 => ash::vk::Format::D32_SFLOAT,
            VulkanImageFormat::BGRA8SRGB => ash::vk::Format::B8G8R8A8_SRGB,
            VulkanImageFormat::BGRA8UNORM => ash::vk::Format::B8G8R8A8_UNORM,
            VulkanImageFormat::R8UNORM => ash::vk::Format::R8_UNORM,
            VulkanImageFormat::RG8UNORM => ash::vk::Format::R8G8_UNORM,
            VulkanImageFormat::RGB8UNORM => ash::vk::Format::R8G8B8_UNORM,
            VulkanImageFormat::RGBA8UNORM => ash::vk::Format::R8G8B8A8_UNORM,
            VulkanImageFormat::R16UNORM => ash::vk::Format::R16_UNORM,
            VulkanImageFormat::RG16UNORM => ash::vk::Format::R16G16_UNORM,
            VulkanImageFormat::RGB16UNORM => ash::vk::Format::R16G16B16_UNORM,
            VulkanImageFormat::RGBA16UNORM => {
                ash::vk::Format::R16G16B16A16_UNORM
            },
            VulkanImageFormat::R16SNORM => ash::vk::Format::R16_SNORM,
            VulkanImageFormat::RG16SNORM => ash::vk::Format::R16G16_SNORM,
            VulkanImageFormat::RGB16SNORM => ash::vk::Format::R16G16B16_SNORM,
            VulkanImageFormat::RGBA16SNORM => {
                ash::vk::Format::R16G16B16A16_SNORM
            },
        }
    }

    pub fn general_image_format(&self) -> ImageFormat {
        match self {
            Self::D32 => ImageFormat::D32,
            Self::R8SRGB => ImageFormat::R8SRGB,
            Self::RG8SRGB => ImageFormat::RG8SRGB,
            Self::RGB8SRGB => ImageFormat::RGB8SRGB,
            Self::RGBA8SRGB => ImageFormat::RGBA8SRGB,
            Self::BGRA8SRGB => ImageFormat::BGRA8SRGB,
            Self::R8UNORM => ImageFormat::R8UNORM,
            Self::RG8UNORM => ImageFormat::RG8UNORM,
            Self::RGB8UNORM => ImageFormat::RGB8UNORM,
            Self::RGBA8UNORM => ImageFormat::RGBA8UNORM,
            Self::BGRA8UNORM => ImageFormat::BGRA8UNORM,
            Self::R16UNORM => ImageFormat::R16UNORM,
            Self::RG16UNORM => ImageFormat::RG16UNORM,
            Self::RGB16UNORM => ImageFormat::RGB16UNORM,
            Self::RGBA16UNORM => ImageFormat::RGBA16UNORM,
            Self::R16SNORM => ImageFormat::R16SNORM,
            Self::RG16SNORM => ImageFormat::RG16SNORM,
            Self::RGB16SNORM => ImageFormat::RGB16SNORM,
            Self::RGBA16SNORM => ImageFormat::RGBA16SNORM,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(unused)]
pub enum VulkanImageLayout {
    Undefined,
    General,
    ColorAttachmentOptimal,
    DepthStencilAttachmentOptimal,
    DepthStencilReadOnlyOptimal,
    ShaderReadOnlyOptimal,
    TransferSrcOptimal,
    TransferDstOptimal,
    PresentSrcKHR,
    Preinitialized,
}

impl VulkanImageLayout {
    pub fn raw_image_layout(&self) -> ash::vk::ImageLayout {
        match self {
            VulkanImageLayout::Undefined => ash::vk::ImageLayout::UNDEFINED,
            VulkanImageLayout::General => ash::vk::ImageLayout::GENERAL,
            VulkanImageLayout::ColorAttachmentOptimal => {
                ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
            },
            VulkanImageLayout::DepthStencilAttachmentOptimal => {
                ash::vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
            },
            VulkanImageLayout::DepthStencilReadOnlyOptimal => {
                ash::vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL
            },
            VulkanImageLayout::ShaderReadOnlyOptimal => {
                ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
            },
            VulkanImageLayout::TransferSrcOptimal => {
                ash::vk::ImageLayout::TRANSFER_SRC_OPTIMAL
            },
            VulkanImageLayout::TransferDstOptimal => {
                ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL
            },
            VulkanImageLayout::PresentSrcKHR => {
                ash::vk::ImageLayout::PRESENT_SRC_KHR
            },
            VulkanImageLayout::Preinitialized => {
                ash::vk::ImageLayout::PREINITIALIZED
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VulkanImageSize {
    size: Vec2u,
}

impl VulkanImageSize {
    pub fn new(size: Vec2u) -> Option<Self> {
        if size.x == 0 || size.y == 0 {
            None
        } else {
            Some(Self { size })
        }
    }

    pub fn raw_size(&self) -> Vec2u { self.size }
}

#[allow(dead_code)]
pub trait VulkanImage: Send + Sync + 'static {
    fn id(&self) -> u32;
    fn format(&self) -> VulkanImageFormat;
    fn usage(&self) -> &[VulkanImageUsage];
    fn aspect(&self) -> VulkanImageAspect;
    fn extent(&self) -> VulkanImageSize;
    fn get_layout(&self) -> VulkanImageLayout;
    fn set_layout(&mut self, layout: VulkanImageLayout);
    fn raw_image(&self) -> &ash::vk::Image;
    fn destroy(&self, device: &VulkanDevice, allocator: &mut VulkanAllocator);
}

#[derive(Debug, Clone)]
pub struct VulkanSwapchainImage {
    id: u32,
    format: VulkanImageFormat,
    extent: VulkanImageSize,
    last_known_layout: VulkanImageLayout,
    image: ash::vk::Image,
}

#[allow(unused)]
pub struct VulkanOwnedImage {
    id: u32,
    allocation_id: u32,
    extent: VulkanImageSize,
    format: VulkanImageFormat,
    usage: Vec<VulkanImageUsage>,
    aspect: VulkanImageAspect,
    last_known_layout: VulkanImageLayout,
    image: ash::vk::Image,
    memory: VulkanMemory,
    memory_offset: u64,
    memory_size: u64,
    size: u64,
}

#[derive(Debug)]
pub struct VulkanImageView {
    id: u32,
    image_id: u32,
    view: ash::vk::ImageView,
}

impl VulkanImage for VulkanSwapchainImage {
    fn id(&self) -> u32 { self.id }
    fn raw_image(&self) -> &ash::vk::Image { &self.image }
    fn format(&self) -> VulkanImageFormat { self.format }
    fn usage(&self) -> &[VulkanImageUsage] {
        &[VulkanImageUsage::ColorAttachment]
    }
    fn extent(&self) -> VulkanImageSize { self.extent }
    fn get_layout(&self) -> VulkanImageLayout { self.last_known_layout }
    fn set_layout(&mut self, layout: VulkanImageLayout) {
        self.last_known_layout = layout;
    }
    fn aspect(&self) -> VulkanImageAspect { VulkanImageAspect::Color }
    fn destroy(&self, _: &VulkanDevice, _: &mut VulkanAllocator) {}
}

impl VulkanImage for VulkanOwnedImage {
    fn id(&self) -> u32 { self.id }
    fn format(&self) -> VulkanImageFormat { self.format }
    fn usage(&self) -> &[VulkanImageUsage] { &self.usage }
    fn aspect(&self) -> VulkanImageAspect { self.aspect }
    fn extent(&self) -> VulkanImageSize { self.extent }
    fn get_layout(&self) -> VulkanImageLayout { self.last_known_layout }
    fn set_layout(&mut self, layout: VulkanImageLayout) {
        self.last_known_layout = layout;
    }
    fn raw_image(&self) -> &ash::vk::Image { &self.image }
    fn destroy(&self, device: &VulkanDevice, allocator: &mut VulkanAllocator) {
        unsafe { device.raw_device().destroy_image(*self.raw_image(), None) }
        allocator.free(
            device,
            self.allocation_id,
            self.memory_offset,
            self.memory_size,
        );
    }
}

#[derive(Debug, thiserror::Error)]
pub enum VulkanImageError {
    #[error("Failed to create image: {0}")]
    CreationError(ash::vk::Result),
    #[error("Failed to create image view: {0}")]
    ViewCreationError(ash::vk::Result),
    #[error("Failed to allocate memory: {0:?}")]
    AllocationError(VulkanAllocationError),
    #[error("Failed to bind memory to image: {0}")]
    BindError(ash::vk::Result),
    #[error("Image format not supported")]
    FormatNotSupported,
}

fn get_image_view_create_info(
    image: &Box<dyn VulkanImage>,
) -> ash::vk::ImageViewCreateInfo<'_> {
    ash::vk::ImageViewCreateInfo::default()
        .view_type(ash::vk::ImageViewType::TYPE_2D)
        .format(image.format().raw_image_format())
        .subresource_range(ash::vk::ImageSubresourceRange {
            aspect_mask: image.aspect().raw_image_aspect(),
            level_count: 1,
            layer_count: 1,
            ..Default::default()
        })
        .components(ash::vk::ComponentMapping::default())
        .image(*image.raw_image())
}

fn get_image_create_info(
    format: VulkanImageFormat,
    size: VulkanImageSize,
    usage: &[VulkanImageUsage],
) -> ash::vk::ImageCreateInfo<'_> {
    ash::vk::ImageCreateInfo::default()
        .image_type(ash::vk::ImageType::TYPE_2D)
        .format(format.raw_image_format())
        .extent(raw_extent_2d(size.raw_size()).into())
        .mip_levels(1)
        .array_layers(1)
        .samples(ash::vk::SampleCountFlags::TYPE_1)
        .tiling(ash::vk::ImageTiling::OPTIMAL)
        .usage(VulkanImageUsage::raw_combined_image_usage(usage))
        .sharing_mode(ash::vk::SharingMode::EXCLUSIVE)
        .initial_layout(ash::vk::ImageLayout::UNDEFINED)
}

create_counter! {ID_COUNTER_IMAGE}
impl VulkanSwapchainImage {
    pub fn new(
        format: VulkanImageFormat,
        extent: VulkanImageSize,
        raw_image: ash::vk::Image,
    ) -> VulkanSwapchainImage {
        VulkanSwapchainImage {
            id: ID_COUNTER_IMAGE.next(),
            format,
            extent,
            last_known_layout: VulkanImageLayout::PresentSrcKHR,
            image: raw_image,
        }
    }
}

impl VulkanOwnedImage {
    pub fn new(
        device: &VulkanDevice,
        allocator: &mut VulkanAllocator,
        size: VulkanImageSize,
        format: VulkanImageFormat,
        usage: &[VulkanImageUsage],
        aspect: VulkanImageAspect,
        memory_properties: VulkanMemoryProperties,
    ) -> Result<VulkanOwnedImage, VulkanImageError> {
        let create_info = get_image_create_info(format, size, usage);

        let image = match unsafe {
            device.raw_device().create_image(&create_info, None)
        } {
            Ok(val) => val,
            Err(err) => return Err(VulkanImageError::CreationError(err)),
        };
        let memory_requirements =
            VulkanMemoryRequirements::from_raw_requirements(unsafe {
                device.raw_device().get_image_memory_requirements(image)
            });

        let (memory, memory_offset, memory_size, allocation_id) = allocator
            .get_memory(device, memory_properties, memory_requirements)
            .map_err(|err| VulkanImageError::AllocationError(err))?;

        if let Err(err) = unsafe {
            device.raw_device().bind_image_memory(
                image,
                memory.raw_memory(),
                memory_offset,
            )
        } {
            return Err(VulkanImageError::BindError(err));
        };

        let format_size = match format_size(format.raw_image_format()) {
            Some(val) => val as u64,
            None => return Err(VulkanImageError::FormatNotSupported),
        };

        Ok(VulkanOwnedImage {
            id: ID_COUNTER_IMAGE.next(),
            allocation_id,
            extent: size,
            image,
            format,
            usage: usage.to_vec(),
            aspect,
            last_known_layout: VulkanImageLayout::Undefined,
            memory,
            memory_offset,
            memory_size,
            size: format_size
                * size.raw_size().x as u64
                * size.raw_size().y as u64,
        })
    }
}

create_counter! {ID_COUNTER_VIEW}
impl VulkanImageView {
    pub fn from_image(
        device: &VulkanDevice,
        image: &Box<dyn VulkanImage>,
    ) -> Result<VulkanImageView, VulkanImageError> {
        let create_info = get_image_view_create_info(image);

        match unsafe {
            device.raw_device().create_image_view(&create_info, None)
        } {
            Ok(val) => Ok(VulkanImageView {
                id: ID_COUNTER_VIEW.next(),
                image_id: image.id(),
                view: val,
            }),
            Err(err) => Err(VulkanImageError::ViewCreationError(err)),
        }
    }

    pub fn destroy(self, device: &VulkanDevice) {
        unsafe {
            device
                .raw_device()
                .destroy_image_view(*self.raw_image_view(), None)
        }
    }

    pub fn raw_image_view(&self) -> &ash::vk::ImageView { &self.view }

    pub fn id(&self) -> u32 { self.id }

    pub fn image_id(&self) -> u32 { self.image_id }
}
