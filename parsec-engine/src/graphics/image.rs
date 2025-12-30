#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Image {
    id: u32,
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
pub enum ImageFlag {
    DepthAttachment,
    ColorAttachment,
    ColorBuffer,
    Sampled,
    TransferSrc,
    TransferDst,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageView {
    id: u32,
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
}

impl Image {
    pub fn new(id: u32) -> Image { Image { id } }

    pub fn id(&self) -> u32 { self.id }
}

impl ImageView {
    pub fn new(id: u32) -> ImageView { ImageView { id } }

    pub fn id(&self) -> u32 { self.id }
}
