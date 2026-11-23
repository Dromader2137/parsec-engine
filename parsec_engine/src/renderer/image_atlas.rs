use parsec_engine_math::uvec::Vec2u;

use crate::{
    graphics::{
        ActiveGraphicsBackend,
        backend::BackendError,
        buffer::BufferHandle,
        command_list::{Command, CommandList},
        image::{ImageAspect, ImageFormat, ImageSize, ImageUsage},
    },
    renderer::integrated_image::{IntegratedImage, IntegratedImageBuilder, IntegratedImageError},
};

pub struct ImageAtlasRegion {
    offset: Vec2u,
    size: Vec2u,
}

pub struct ImageAtlas {
    size: Vec2u,
    image: IntegratedImage,
    elements: Vec<ImageAtlasRegion>,
}

#[derive(Debug, Default)]
pub struct ImageAtlasBuilder<'a> {
    size: ImageSize,
    format: ImageFormat,
    aspect: ImageAspect,
    usage: &'a [ImageUsage],
}

impl<'a> ImageAtlasBuilder<'a> {
    pub fn size(mut self, size: ImageSize) -> Self {
        self.size = size;
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
    ) -> Result<ImageAtlas, IntegratedImageError> {
        let image = IntegratedImageBuilder::default()
            .size(self.size)
            .format(self.format)
            .aspect(self.aspect)
            .usage(self.usage)
            .build(backend)?;

        Ok(ImageAtlas {
            size: self.size.get_size(),
            image,
            elements: Vec::new(),
        })
    }
}

impl ImageAtlas {
    pub fn copy_to_region(
        &self,
        backend: &mut ActiveGraphicsBackend,
        buffer: BufferHandle,
        region: ImageAtlasRegion,
    ) -> Result<(), BackendError> {
        backend.load_image_from_buffer(
            buffer,
            self.image.image().handle(),
            region.size,
            region.offset,
        )
    }

    pub fn set_rendering_region(mut command_list: CommandList, region: ImageAtlasRegion) {
        command_list
            .cmd(Command::SetScissor(region.offset, region.size.signed()));
        command_list.cmd(Command::SetViewport(region.size));
    }

    pub fn destroy(self, backend: &mut ActiveGraphicsBackend) -> Result<(), IntegratedImageError> {
        self.image.destroy(backend)
    }

    pub fn size(&self) -> Vec2u { self.size }

    pub fn image(&self) -> &IntegratedImage { &self.image }

    pub fn elements(&self) -> &[ImageAtlasRegion] { &self.elements }
}
