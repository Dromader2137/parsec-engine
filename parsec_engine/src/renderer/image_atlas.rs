use parsec_engine_math::uvec::Vec2u;

use crate::{
    error::ParsecError,
    graphics::{
        ActiveGraphicsBackend,
        buffer::BufferHandle,
        command_list::{Command, CommandList},
        image::{
            ImageAspect, ImageBuilder, ImageFormat, ImageSize, ImageUsage,
            ImageViewBuilder,
        },
        sampler::SamplerBuilder,
    },
    renderer::integrated_image::IntegratedImage,
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
    ) -> Result<ImageAtlas, ParsecError> {
        let image = ImageBuilder::new()
            .size(self.size)
            .format(self.format)
            .aspect(self.aspect)
            .usage(self.usage)
            .build(backend)?;
        let view = ImageViewBuilder::new()
            .image(image.handle())
            .build(backend)?;
        let sampler = SamplerBuilder::new().build(backend)?;
        let image = IntegratedImage::new(image, view, sampler);

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
    ) -> Result<(), ParsecError> {
        backend.load_image_from_buffer(
            buffer,
            self.image.image().handle(),
            region.size,
            region.offset,
        )?;
        Ok(())
    }

    pub fn set_rendering_region(
        mut command_list: CommandList,
        region: ImageAtlasRegion,
    ) -> Result<(), ParsecError> {
        command_list
            .cmd(Command::SetScissor(region.offset, region.size.signed()));
        command_list.cmd(Command::SetViewport(region.size));
        Ok(())
    }

    pub fn destroy(
        self,
        backend: &mut ActiveGraphicsBackend,
    ) -> Result<(), ParsecError> {
        self.image.destroy(backend)
    }

    pub fn size(&self) -> Vec2u { self.size }

    pub fn image(&self) -> &IntegratedImage { &self.image }

    pub fn elements(&self) -> &[ImageAtlasRegion] { &self.elements }
}
