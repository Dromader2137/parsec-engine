use std::fmt::Debug;

use crate::{
    graphics::{
        ActiveGraphicsBackend, image::{
            Image, ImageAspect, ImageBuilder, ImageError, ImageFormat, ImageSize, ImageUsage, ImageView, ImageViewBuilder,
        }, sampler::{Sampler, SamplerBuilder, SamplerError},
    },
};

pub struct IntegratedImage {
    image: Image,
    view: ImageView,
    sampler: Sampler,
}

#[derive(Debug, Default)]
pub struct IntegratedImageBuilder<'a> {
    size: ImageSize,
    format: ImageFormat,
    aspect: ImageAspect,
    usage: &'a [ImageUsage],
}

impl<'a> IntegratedImageBuilder<'a> {
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
    ) -> Result<IntegratedImage, IntegratedImageError> {
        let image = ImageBuilder::new()
            .size(self.size)
            .format(self.format)
            .aspect(self.aspect)
            .usage(self.usage)
            .build(backend)?;
        let view = ImageViewBuilder::new().image(image.handle()).build(backend)?;
        let sampler = SamplerBuilder::new().build(backend)?;
        Ok(IntegratedImage { image, view, sampler })
    }
}

impl IntegratedImage {
    pub fn new(image: Image, view: ImageView, sampler: Sampler) -> Self {
        Self { image, view, sampler }
    }

    pub fn destroy(self, backend: &mut ActiveGraphicsBackend) -> Result<(), IntegratedImageError> {
        self.image.destroy(backend)?;
        self.view.destroy(backend)?;
        self.sampler.destroy(backend)?;
        Ok(())
    }

    pub fn image(&self) -> &Image { &self.image }

    pub fn view(&self) -> &ImageView { &self.view }

    pub fn sampler(&self) -> &Sampler { &self.sampler }
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum IntegratedImageError {
    #[error(transparent)]
    Image(#[from] ImageError),
    #[error(transparent)]
    Sampler(#[from] SamplerError)
}
