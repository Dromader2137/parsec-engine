use super::{
    VulkanError, buffer::find_memorytype_index, device::Device, format_size::format_size,
    instance::Instance, physical_device::PhysicalDevice,
};

pub trait GetImageRaw {
    fn get_image_raw(&self) -> &ash::vk::Image;
}

pub struct Image {
    image: ash::vk::Image,
}

#[allow(unused)]
pub struct OwnedImage {
    image: ash::vk::Image,
    memory: ash::vk::DeviceMemory,
    size: u64,
}

pub struct ImageView {
    view: ash::vk::ImageView,
}

impl GetImageRaw for Image {
    fn get_image_raw(&self) -> &ash::vk::Image {
        &self.image
    }
}

impl GetImageRaw for OwnedImage {
    fn get_image_raw(&self) -> &ash::vk::Image {
        &self.image
    }
}

#[derive(Debug)]
pub enum ImageError {
    CreationError(ash::vk::Result),
    ViewCreationError(ash::vk::Result),
    UnableToFindSuitableMemory,
    AllocationError(ash::vk::Result),
    BindError(ash::vk::Result),
    FormatNotSupported,
}

impl From<ImageError> for VulkanError {
    fn from(value: ImageError) -> Self {
        VulkanError::ImageError(value)
    }
}

pub type ImageFormat = ash::vk::Format;
pub type ImageUsage = ash::vk::ImageUsageFlags;
pub type ImageAspectFlags = ash::vk::ImageAspectFlags;

pub struct ImageViewInfo<'a> {
    image: &'a dyn GetImageRaw,
    format: ImageFormat,
    aspect_flags: ImageAspectFlags,
}

pub struct ImageInfo {
    pub format: ImageFormat,
    pub size: (u32, u32),
    pub usage: ImageUsage,
}

impl<'a> From<ImageViewInfo<'a>> for ash::vk::ImageViewCreateInfo<'_> {
    fn from(value: ImageViewInfo) -> Self {
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

impl From<ImageInfo> for ash::vk::ImageCreateInfo<'_> {
    fn from(value: ImageInfo) -> Self {
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

impl Image {
    pub fn from_raw_image(raw_image: ash::vk::Image) -> Image {
        Image { image: raw_image }
    }

    pub fn cleanup(&self, device: &Device) {
        unsafe { device.get_device_raw().destroy_image(self.image, None) };
    }
}

impl OwnedImage {
    pub fn new(
        instance: &Instance,
        physical_device: &PhysicalDevice,
        device: &Device,
        create_info: ImageInfo,
    ) -> Result<OwnedImage, ImageError> {
        let size = create_info.size;
        let format = create_info.format;

        let image = match unsafe {
            device
                .get_device_raw()
                .create_image(&create_info.into(), None)
        } {
            Ok(val) => val,
            Err(err) => return Err(ImageError::CreationError(err)),
        };
        let memory_req = unsafe { device.get_device_raw().get_image_memory_requirements(image) };
        let memory_index = match find_memorytype_index(
            &memory_req,
            ash::vk::MemoryPropertyFlags::DEVICE_LOCAL,
            instance,
            physical_device,
        ) {
            Some(val) => val,
            None => return Err(ImageError::UnableToFindSuitableMemory),
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
            Err(err) => return Err(ImageError::AllocationError(err)),
        };

        if let Err(err) = unsafe {
            device
                .get_device_raw()
                .bind_image_memory(image, image_memory, 0)
        } {
            return Err(ImageError::BindError(err));
        };

        let format_size = match format_size(format) {
            Some(val) => val as u64,
            None => return Err(ImageError::FormatNotSupported),
        };

        Ok(OwnedImage {
            image,
            memory: image_memory,
            size: format_size * size.0 as u64 * size.1 as u64,
        })
    }

    pub fn get_memory_raw(&self) -> &ash::vk::DeviceMemory {
        &self.memory
    }

    pub fn cleanup(&self, device: &Device) {
        unsafe { device.get_device_raw().free_memory(self.memory, None) };
        unsafe { device.get_device_raw().destroy_image(self.image, None) };
    }
}

impl ImageView {
    pub fn from_image(
        device: &Device,
        image: &impl GetImageRaw,
        image_format: ImageFormat,
        aspect_flags: ImageAspectFlags,
    ) -> Result<ImageView, ImageError> {
        let view_info = ImageViewInfo {
            image,
            format: image_format,
            aspect_flags,
        };

        match unsafe {
            device
                .get_device_raw()
                .create_image_view(&view_info.into(), None)
        } {
            Ok(val) => Ok(ImageView { view: val }),
            Err(err) => Err(ImageError::ViewCreationError(err)),
        }
    }

    pub fn get_image_view_raw(&self) -> &ash::vk::ImageView {
        &self.view
    }

    pub fn cleanup(&self, device: &Device) {
        unsafe { device.get_device_raw().destroy_image_view(self.view, None) };
    }
}
