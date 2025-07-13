pub fn format_size(format: ash::vk::Format) -> Option<u32> {
    match format {
        ash::vk::Format::R8_UNORM
        | ash::vk::Format::R8_SNORM
        | ash::vk::Format::R8_UINT
        | ash::vk::Format::R8_SINT
        | ash::vk::Format::R8_SRGB => Some(1),
        ash::vk::Format::R8G8_UNORM
        | ash::vk::Format::R8G8_SNORM
        | ash::vk::Format::R8G8_UINT
        | ash::vk::Format::R8G8_SINT
        | ash::vk::Format::R8G8_SRGB => Some(2),
        ash::vk::Format::R8G8B8_UNORM
        | ash::vk::Format::R8G8B8_SNORM
        | ash::vk::Format::R8G8B8_UINT
        | ash::vk::Format::R8G8B8_SINT
        | ash::vk::Format::R8G8B8_SRGB
        | ash::vk::Format::B8G8R8_UNORM
        | ash::vk::Format::B8G8R8_SNORM
        | ash::vk::Format::B8G8R8_UINT
        | ash::vk::Format::B8G8R8_SINT
        | ash::vk::Format::B8G8R8_SRGB => Some(3),
        ash::vk::Format::R8G8B8A8_UNORM
        | ash::vk::Format::R8G8B8A8_SNORM
        | ash::vk::Format::R8G8B8A8_UINT
        | ash::vk::Format::R8G8B8A8_SINT
        | ash::vk::Format::R8G8B8A8_SRGB
        | ash::vk::Format::B8G8R8A8_UNORM
        | ash::vk::Format::B8G8R8A8_SNORM
        | ash::vk::Format::B8G8R8A8_UINT
        | ash::vk::Format::B8G8R8A8_SINT
        | ash::vk::Format::B8G8R8A8_SRGB
        | ash::vk::Format::A8B8G8R8_UNORM_PACK32
        | ash::vk::Format::A2R10G10B10_UNORM_PACK32
        | ash::vk::Format::A2B10G10R10_UNORM_PACK32
        | ash::vk::Format::B10G11R11_UFLOAT_PACK32
        | ash::vk::Format::E5B9G9R9_UFLOAT_PACK32 => Some(4),
        ash::vk::Format::R16_UNORM
        | ash::vk::Format::R16_SNORM
        | ash::vk::Format::R16_UINT
        | ash::vk::Format::R16_SINT
        | ash::vk::Format::R16_SFLOAT => Some(2),
        ash::vk::Format::R16G16_UNORM
        | ash::vk::Format::R16G16_SNORM
        | ash::vk::Format::R16G16_UINT
        | ash::vk::Format::R16G16_SINT
        | ash::vk::Format::R16G16_SFLOAT => Some(4),
        ash::vk::Format::R16G16B16_UNORM
        | ash::vk::Format::R16G16B16_SNORM
        | ash::vk::Format::R16G16B16_UINT
        | ash::vk::Format::R16G16B16_SINT
        | ash::vk::Format::R16G16B16_SFLOAT => Some(6),
        ash::vk::Format::R16G16B16A16_UNORM
        | ash::vk::Format::R16G16B16A16_SNORM
        | ash::vk::Format::R16G16B16A16_UINT
        | ash::vk::Format::R16G16B16A16_SINT
        | ash::vk::Format::R16G16B16A16_SFLOAT => Some(8),
        ash::vk::Format::R32_UINT | ash::vk::Format::R32_SINT | ash::vk::Format::R32_SFLOAT => Some(4),
        ash::vk::Format::R32G32_UINT | ash::vk::Format::R32G32_SINT | ash::vk::Format::R32G32_SFLOAT => Some(8),
        ash::vk::Format::R32G32B32_UINT | ash::vk::Format::R32G32B32_SINT | ash::vk::Format::R32G32B32_SFLOAT => {
            Some(12)
        }
        ash::vk::Format::R32G32B32A32_UINT
        | ash::vk::Format::R32G32B32A32_SINT
        | ash::vk::Format::R32G32B32A32_SFLOAT => Some(16),
        ash::vk::Format::D16_UNORM => Some(2),
        ash::vk::Format::X8_D24_UNORM_PACK32 | ash::vk::Format::D24_UNORM_S8_UINT | ash::vk::Format::D32_SFLOAT => {
            Some(4)
        }
        ash::vk::Format::S8_UINT => Some(1),
        ash::vk::Format::D32_SFLOAT_S8_UINT => Some(5),
        _ => None,
    }
}
