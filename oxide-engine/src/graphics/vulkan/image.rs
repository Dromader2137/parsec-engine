use super::{context::VulkanError, device::Device};

pub struct Image {
    image: ash::vk::Image,
    view: ash::vk::ImageView
}

#[derive(Debug)]
pub enum ImageError {
    CreationError(ash::vk::Result),
    ViewCreationError(ash::vk::Result),
}

impl From<ImageError> for VulkanError {
    fn from(value: ImageError) -> Self {
        VulkanError::ImageError(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageFormat {
    R8G8B8,
    R8G8B8A8
}

impl From<ImageFormat> for ash::vk::Format {
    fn from(value: ImageFormat) -> Self {
        match value {
            ImageFormat::R8G8B8 => ash::vk::Format::R8G8B8_UNORM,
            ImageFormat::R8G8B8A8 => ash::vk::Format::R8G8B8A8_UNORM,
        }
    }
}

pub struct ImageInfo {
    format: ImageFormat
}

impl From<ImageInfo> for ash::vk::ImageCreateInfo<'_> {
    fn from(value: ImageInfo) -> Self {
        ash::vk::ImageCreateInfo::default()
            .format(ash::vk::Format::from(value.format))
    }
}

pub struct ImageViewInfo {
    format: ImageFormat
}

impl From<ImageViewInfo> for ash::vk::ImageViewCreateInfo<'_> {
    fn from(value: ImageViewInfo) -> Self {
        ash::vk::ImageViewCreateInfo::default()
            .format(ash::vk::Format::from(value.format))
    }
}

impl Image {
    pub fn new(device: &Device, image_info: ImageInfo) -> Result<Image, ImageError> {
        let view_info = ImageViewInfo {
            format: image_info.format
        };

        let image = match device.create_image_raw(ash::vk::ImageCreateInfo::from(image_info)) {
            Ok(val) => val,
            Err(err) => return Err(ImageError::CreationError(err))
        };

        let view = match device.create_image_view_raw(ash::vk::ImageViewCreateInfo::from(view_info)) {
            Ok(val) => val,
            Err(err) => return Err(ImageError::ViewCreationError(err))
        };

        Ok(Image { image, view } )
    }
}
