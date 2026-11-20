pub mod png;

use parsec_engine_math::uvec::Vec2u;

use crate::{
    assets::Asset, ecs::resources::Resources, graphics::{
        ActiveGraphicsBackend, buffer::{BufferBuilder, BufferContent, BufferError, BufferUsage}, image::{ImageAspect, ImageError, ImageFormat, ImageHandle, ImageSize, ImageUsage, ImageViewHandle}, sampler::SamplerHandle,
    }, renderer::integrated_image::{IntegratedImage, IntegratedImageBuilder, IntegratedImageError},
};

pub struct TextureInput {
    width: u32,
    height: u32,
    pixel_data: Vec<u8>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TextureCooked {
    width: u32,
    height: u32,
    pixel_data: Vec<u8>,
}

pub struct Texture {
    integrated_image: IntegratedImage,
}

impl Texture {
    pub fn image_handle(&self) -> ImageHandle { self.integrated_image.image().handle() }
    pub fn view_handle(&self) -> ImageViewHandle { self.integrated_image.view().handle() }
    pub fn sampler_handle(&self) -> SamplerHandle { self.integrated_image.sampler().handle() }
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
enum TextureCookError {}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
enum TextureLoadError {
    #[error("graphics backend not found in resources")]
    GraphicsBackendNotFound,
    #[error(transparent)]
    Buffer(#[from] BufferError),
    #[error(transparent)]
    IntegratedImage(#[from] IntegratedImageError),
    #[error(transparent)]
    Image(#[from] ImageError),
}

impl Asset for Texture {
    type Input = TextureInput;
    type Cooked = TextureCooked;
    type CookError = TextureCookError;
    type LoadError = TextureLoadError;

    const ASSET_TYPE: &'static str = "texture";

    fn cook(input_data: Self::Input) -> Result<Self::Cooked, Self::CookError> {
        let TextureInput { width, height, pixel_data } = input_data;
        Ok(TextureCooked { width, height, pixel_data })
    }

    fn load(cooked: Self::Cooked, resources: &mut Resources) -> Result<Self, Self::LoadError> {
        let mut backend = resources
            .get_mut::<ActiveGraphicsBackend>()
                .ok_or(TextureLoadError::GraphicsBackendNotFound)?;
        let buffer = BufferBuilder::new()
            .usage(&[BufferUsage::TransferSrc])
            .data(BufferContent::from_slice(&cooked.pixel_data))
            .build(&mut backend)?;
        let integrated_image = IntegratedImageBuilder::default()
            .size(ImageSize::new(Vec2u::new(cooked.width, cooked.height))?)
            .format(ImageFormat::RGBA8SRGB)
            .aspect(ImageAspect::Color)
            .usage(&[ImageUsage::Sampled, ImageUsage::TransferDst])
            .build(&mut backend)?;
        integrated_image.image()
            .load_from_buffer(
                &mut backend,
                buffer.handle(),
                Vec2u::new(cooked.height, cooked.height),
                Vec2u::ZERO,
            )?;
        Ok(Self { integrated_image })
    }
}
