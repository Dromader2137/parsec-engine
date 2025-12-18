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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageUsage {
    DepthBuffer,
    Sampled,
    Src,
    Dst,
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
