use super::{VulkanError, device::Device};

pub struct Image {
    image: ash::vk::Image,
}

pub struct ImageView {
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

pub type ImageFormat = ash::vk::Format;

pub struct ImageViewInfo<'a> {
    image: &'a Image,
    format: ImageFormat,
}

impl<'a> From<ImageViewInfo<'a>> for ash::vk::ImageViewCreateInfo<'_> {
    fn from(value: ImageViewInfo) -> Self {
        ash::vk::ImageViewCreateInfo::default()
            .view_type(ash::vk::ImageViewType::TYPE_2D)
            .format(ash::vk::Format::from(value.format))
            .subresource_range(ash::vk::ImageSubresourceRange {
                aspect_mask: ash::vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1
            })
            .components(ash::vk::ComponentMapping {
                r: ash::vk::ComponentSwizzle::R,
                g: ash::vk::ComponentSwizzle::G,
                b: ash::vk::ComponentSwizzle::B,
                a: ash::vk::ComponentSwizzle::A,
            })
            .image(value.image.image)
    }
}

impl Image {
    pub fn from_raw_image(raw_image: ash::vk::Image) -> Image {
        Image { image: raw_image }
    }

    pub fn get_image_raw(&self) -> &ash::vk::Image {
        &self.image
    }

    pub fn cleanup(&self, device: &Device) {
        unsafe { device.get_device_raw().destroy_image(self.image, None) };
    }
}

impl ImageView {
    pub fn from_image(device: &Device, image: &Image, image_format: ImageFormat) -> Result<ImageView, ImageError> {
        let view_info = ImageViewInfo {
            image,
            format: image_format
        };

        match unsafe { device.get_device_raw().create_image_view(&view_info.into(), None) } {
            Ok(val) => Ok(ImageView { view: val }),
            Err(err) => Err(ImageError::ViewCreationError(err))
        }
    }

    pub fn get_image_view_raw(&self) -> &ash::vk::ImageView {
        &self.view
    }

    pub fn cleanup(&self, device: &Device) {
        unsafe { device.get_device_raw().destroy_image_view(self.view, None) };
    }
}
