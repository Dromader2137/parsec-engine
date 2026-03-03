use crate::math::uvec::Vec2u;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Image {
    id: u32,
}

impl Image {
    pub fn new(id: u32) -> Image { Image { id } }

    pub fn id(&self) -> u32 { self.id }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageView {
    id: u32,
}

impl ImageView {
    pub fn new(id: u32) -> ImageView { ImageView { id } }

    pub fn id(&self) -> u32 { self.id }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageFormat {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageUsage {
    DepthAttachment,
    ColorAttachment,
    Sampled,
    TransferSrc,
    TransferDst,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageAspect {
    Color,
    Depth
}

#[derive(Debug)]
pub enum ImageError {
    ImageCreationError(anyhow::Error),
    ImageLoadError(anyhow::Error),
    ImageDeletionError(anyhow::Error),
    ImageViewCreationError(anyhow::Error),
    ImageViewDeletionError(anyhow::Error),
    ImageNotFound,
    SwapchainImageNotFound,
    ImageViewNotFound,
    BufferNotFound,
    InvalidImageSize
}

pub struct ImageSize {
    size: Vec2u
}

impl ImageSize {
    pub fn new(size: Vec2u) -> Result<ImageSize, ImageError> {
        if size.x == 0 || size.y == 0 {
            return Err(ImageError::InvalidImageSize)
        }

        Ok(ImageSize { size })
    }

    pub fn get_size(&self) -> Vec2u {
        self.size
    }
}
