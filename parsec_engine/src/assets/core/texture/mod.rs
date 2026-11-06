pub mod png;

use parsec_engine_math::uvec::Vec2u;

use crate::{
    assets::Asset, ecs::resources::Resources, graphics::{
        ActiveGraphicsBackend, buffer::{BufferBuilder, BufferContent, BufferUsage}, image::{ImageAspect, ImageFormat, ImageHandle, ImageSize, ImageUsage, ImageViewHandle}, sampler::SamplerHandle,
    }, renderer::integrated_image::{IntegratedImage, IntegratedImageBuilder},
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
    image: IntegratedImage,
}

impl Texture {
    pub fn image_handle(&self) -> ImageHandle { self.image.image().handle() }
    pub fn view_handle(&self) -> ImageViewHandle { self.image.view().handle() }
    pub fn sampler_handle(&self) -> SamplerHandle { self.image.sampler().handle() }
}

impl Asset for Texture {
    type Input = TextureInput;
    type Cooked = TextureCooked;

    const ASSET_TYPE: &'static str = "texture";

    fn cook(input_data: Self::Input) -> Self::Cooked {
        let TextureInput { width, height, pixel_data } = input_data;
        TextureCooked { width, height, pixel_data }
    }

    fn load(cooked: Self::Cooked, resources: &mut Resources) -> Self {
        let mut backend = resources
            .get_mut::<ActiveGraphicsBackend>()
            .expect("loading a texture requires a graphics backend to be active");
        let buffer = BufferBuilder::new()
            .usage(&[BufferUsage::TransferSrc])
            .data(BufferContent::from_slice(&cooked.pixel_data))
            .build(&mut backend)
            .unwrap();
        let image = IntegratedImageBuilder::default()
            .size(ImageSize::new(Vec2u::new(cooked.width, cooked.height)).unwrap())
            .format(ImageFormat::RGBA8SRGB)
            .aspect(ImageAspect::Color)
            .usage(&[ImageUsage::Sampled, ImageUsage::TransferDst])
            .build(&mut backend)
            .unwrap();
        backend
            .load_image_from_buffer(
                buffer.handle(),
                image.image().handle(),
                Vec2u::new(cooked.height, cooked.height),
                Vec2u::ZERO,
            )
            .unwrap();
        Self { image }
    }
}
