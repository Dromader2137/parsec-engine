use parsec_engine_error::ParsecError;
use parsec_engine_math::uvec::Vec2u;

use crate::{ActiveGraphicsBackend, buffer::BufferHandle};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageHandle {
    id: u32,
}

impl ImageHandle {
    pub fn new(id: u32) -> Self { Self { id } }
    pub fn id(&self) -> u32 { self.id }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageViewHandle {
    id: u32,
}

impl ImageViewHandle {
    pub fn new(id: u32) -> Self { Self { id } }
    pub fn id(&self) -> u32 { self.id }
}

#[derive(Debug)]
pub struct Image {
    handle: ImageHandle,
    size: Vec2u,
    format: ImageFormat,
    aspect: ImageAspect,
    usage: Vec<ImageUsage>,
}

impl Image {
    fn new(
        handle: ImageHandle,
        size: Vec2u,
        format: ImageFormat,
        aspect: ImageAspect,
        usage: Vec<ImageUsage>,
    ) -> Image {
        Image {
            handle,
            size,
            format,
            aspect,
            usage,
        }
    }

    pub fn handle(&self) -> ImageHandle { self.handle }
    pub fn id(&self) -> u32 { self.handle.id }
    pub fn size(&self) -> Vec2u { self.size }
    pub fn format(&self) -> ImageFormat { self.format }
    pub fn aspect(&self) -> ImageAspect { self.aspect }
    pub fn usage(&self) -> &[ImageUsage] { &self.usage }

    pub fn create_view(
        &self,
        backend: &mut ActiveGraphicsBackend,
    ) -> Result<ImageView, ImageError> {
        let handle = backend.create_image_view(self.handle)?;
        Ok(ImageView::new(handle))
    }

    pub fn load_from_buffer(
        &self,
        backend: &mut ActiveGraphicsBackend,
        buffer: BufferHandle,
        image_size: Vec2u,
        image_offset: Vec2u,
    ) -> Result<(), ImageError> {
        backend.load_image_from_buffer(
            buffer,
            self.handle,
            image_size,
            image_offset,
        )
    }

    pub fn destroy(
        self,
        backend: &mut ActiveGraphicsBackend,
    ) -> Result<(), ImageError> {
        backend.delete_image(self)
    }
}

#[derive(Debug)]
pub struct ImageView {
    handle: ImageViewHandle,
}

impl ImageView {
    fn new(handle: ImageViewHandle) -> ImageView { ImageView { handle } }

    pub fn handle(&self) -> ImageViewHandle { self.handle }
    pub fn id(&self) -> u32 { self.handle.id }

    pub fn destroy(
        self,
        backend: &mut ActiveGraphicsBackend,
    ) -> Result<(), ImageError> {
        backend.delete_image_view(self)
    }
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

pub struct ImageBuilder<'a> {
    size: Option<ImageSize>,
    format: ImageFormat,
    aspect: ImageAspect,
    usage: &'a [ImageUsage],
}

impl<'a> Default for ImageBuilder<'a> {
    fn default() -> Self { Self::new() }
}

impl<'a> ImageBuilder<'a> {
    pub fn new() -> Self {
        Self {
            size: None,
            format: ImageFormat::default(),
            aspect: ImageAspect::default(),
            usage: &[],
        }
    }

    pub fn size(mut self, size: ImageSize) -> Self {
        self.size = Some(size);
        self
    }

    pub fn format(mut self, format: ImageFormat) -> Self {
        self.format = format;
        self
    }

    pub fn aspect(mut self, aspect: ImageAspect) -> Self {
        self.aspect = aspect;
        self
    }

    pub fn usage(mut self, usage: &'a [ImageUsage]) -> Self {
        self.usage = usage;
        self
    }

    pub fn build(
        self,
        backend: &mut ActiveGraphicsBackend,
    ) -> Result<Image, ImageError> {
        let size = self.size.ok_or(ImageError::InvalidImageSize)?;
        let handle = backend.create_image(
            size.get_size(),
            self.format,
            self.aspect,
            self.usage,
        )?;
        Ok(Image::new(
            handle,
            size.get_size(),
            self.format,
            self.aspect,
            self.usage.to_vec(),
        ))
    }
}

pub struct ImageViewBuilder {
    image: Option<ImageHandle>,
}

impl Default for ImageViewBuilder {
    fn default() -> Self { Self::new() }
}

impl ImageViewBuilder {
    pub fn new() -> Self { Self { image: None } }

    pub fn image(mut self, image: ImageHandle) -> Self {
        self.image = Some(image);
        self
    }

    pub fn build(
        self,
        backend: &mut ActiveGraphicsBackend,
    ) -> Result<ImageView, ImageError> {
        let image = self.image.ok_or(ImageError::ImageNotFound)?;
        let handle = backend.create_image_view(image)?;
        Ok(ImageView::new(handle))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ImageError {
    #[error("failed to create image: {0}")]
    ImageCreationError(ParsecError),
    #[error("failed to load image: {0}")]
    ImageLoadError(ParsecError),
    #[error("failed to delete image: {0}")]
    ImageDeletionError(ParsecError),
    #[error("failed to create image view: {0}")]
    ImageViewCreationError(ParsecError),
    #[error("failed to delete image view: {0}")]
    ImageViewDeletionError(ParsecError),
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

#[derive(Debug, Clone, Copy)]
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
