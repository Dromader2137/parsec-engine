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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ImageFormat {
    #[default]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ImageUsage {
    DepthAttachment,
    ColorAttachment,
    #[default]
    Sampled,
    TransferSrc,
    TransferDst,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ImageAspect {
    #[default]
    Color,
    Depth,
}

#[derive(Debug, thiserror::Error)]
pub enum ImageError {
    #[error("failed to create image: {0}")]
    ImageCreationError(anyhow::Error),
    #[error("failed to load image: {0}")]
    ImageLoadError(anyhow::Error),
    #[error("failed to delete image: {0}")]
    ImageDeletionError(anyhow::Error),
    #[error("failed to create image view: {0}")]
    ImageViewCreationError(anyhow::Error),
    #[error("failed to delete image view: {0}")]
    ImageViewDeletionError(anyhow::Error),
    #[error("image does not exist")]
    ImageNotFound,
    #[error("present image does not exist")]
    SwapchainImageNotFound,
    #[error("image view does not exist")]
    ImageViewNotFound,
    #[error("buffer does not exist")]
    BufferNotFound,
    #[error("invalid image size (expected width > 0 and height > 0)")]
    InvalidImageSize,
}

#[derive(Debug)]
pub struct ImageSize {
    size: Vec2u,
}

impl Default for ImageSize {
    fn default() -> Self { Self { size: Vec2u::ONE } }
}

impl ImageSize {
    pub fn new(size: Vec2u) -> Result<ImageSize, ImageError> {
        if size.x == 0 || size.y == 0 {
            return Err(ImageError::InvalidImageSize);
        }

        Ok(ImageSize { size })
    }

    pub fn get_size(&self) -> Vec2u { self.size }
}
