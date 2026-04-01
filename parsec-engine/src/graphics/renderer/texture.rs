use std::fmt::Debug;

use crate::{graphics::{
    ActiveGraphicsBackend,
    image::{
        Image, ImageAspect, ImageFormat, ImageSize, ImageUsage, ImageView,
    },
    sampler::Sampler,
}, error::ParsecError};

pub struct Texture {
    image: Image,
    view: ImageView,
    sampler: Sampler,
}

#[derive(Debug, Default)]
pub struct TextureBuilder<'a> {
    size: ImageSize,
    format: ImageFormat,
    aspect: ImageAspect,
    usage: &'a [ImageUsage],
}

impl<'a> TextureBuilder<'a> {
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
    ) -> Result<Texture, ParsecError> {
        let image = backend.create_image(
            self.size.get_size(),
            self.format,
            self.aspect,
            self.usage,
        )?;
        let view = backend.create_image_view(image)?;
        let sampler = backend.create_image_sampler()?;
        Ok(Texture {
            image,
            view,
            sampler,
        })
    }
}

impl Texture {
    pub fn new(image: Image, view: ImageView, sampler: Sampler) -> Self {
        Self {
            image,
            view,
            sampler,
        }
    }

    pub fn delete(
        self,
        backend: &mut ActiveGraphicsBackend,
    ) -> Result<(), ParsecError> {
        backend.delete_image(self.image)?;
        backend.delete_image_view(self.view)?;
        backend.delete_image_sampler(self.sampler)?;
        Ok(())
    }

    pub fn image(&self) -> Image { self.image }

    pub fn view(&self) -> ImageView { self.view }

    pub fn sampler(&self) -> Sampler { self.sampler }
}
