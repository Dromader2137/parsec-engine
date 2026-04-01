use crate::{
    graphics::{
        ActiveGraphicsBackend,
        buffer::Buffer,
        image::{ImageAspect, ImageFormat, ImageSize, ImageUsage},
        renderer::texture::Texture,
    },
    math::uvec::Vec2u, error::ParsecError,
};

struct TextureAtlasElement {
    offset: Vec2u,
    size: Vec2u,
}

pub struct TextureAtlas {
    size: Vec2u,
    texture: Texture,
    elements: Vec<TextureAtlasElement>,
}

#[derive(Debug, Default)]
pub struct TextureAtlasBuilder<'a> {
    size: ImageSize,
    format: ImageFormat,
    aspect: ImageAspect,
    usage: &'a [ImageUsage],
}

impl<'a> TextureAtlasBuilder<'a> {
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
    ) -> Result<TextureAtlas, ParsecError> {
        let image = backend.create_image(
            self.size.get_size(),
            self.format,
            self.aspect,
            self.usage,
        )?;
        let view = backend.create_image_view(image)?;
        let sampler = backend.create_image_sampler()?;
        let texture = Texture::new(image, view, sampler);

        Ok(TextureAtlas {
            size: self.size.get_size(),
            texture,
            elements: Vec::new(),
        })
    }
}

impl TextureAtlas {
    pub fn copy_to_region(
        &self,
        backend: &mut ActiveGraphicsBackend,
        buffer: Buffer,
        size: Vec2u,
        offset: Vec2u,
    ) -> Result<(), ParsecError> {
        backend.load_image_from_buffer(
            buffer,
            self.texture.image(),
            size,
            offset,
        )?;
        Ok(())
    }

    pub fn delete(
        self,
        backend: &mut ActiveGraphicsBackend,
    ) -> Result<(), ParsecError> {
        self.texture.delete(backend)
    }

    pub fn size(&self) -> Vec2u { self.size }

    pub fn texture(&self) -> &Texture { &self.texture }
}
