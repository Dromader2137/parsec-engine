use parsec_engine_error::ParsecError;
use parsec_engine_graphics::{
    ActiveGraphicsBackend,
    image::{
        Image, ImageAspect, ImageBuilder, ImageFormat, ImageHandle, ImageSize,
        ImageUsage, ImageView, ImageViewBuilder, ImageViewHandle,
    },
};

#[derive(Debug)]
pub struct DepthImage {
    image: Option<Image>,
    image_view: Option<ImageView>,
}

impl DepthImage {
    pub fn new(
        backend: &mut ActiveGraphicsBackend,
        size: ImageSize,
    ) -> Result<DepthImage, ParsecError> {
        let image = ImageBuilder::new()
            .size(size)
            .format(ImageFormat::D32)
            .aspect(ImageAspect::Depth)
            .usage(&[ImageUsage::DepthAttachment])
            .build(backend)?;
        let image_view = ImageViewBuilder::new()
            .image(image.handle())
            .build(backend)?;
        Ok(DepthImage {
            image: Some(image),
            image_view: Some(image_view),
        })
    }

    pub fn recreate(
        &mut self,
        backend: &mut ActiveGraphicsBackend,
        size: ImageSize,
    ) -> Result<(), ParsecError> {
        self.image.take().unwrap().destroy(backend)?;
        self.image_view.take().unwrap().destroy(backend)?;
        let image = ImageBuilder::new()
            .size(size)
            .format(ImageFormat::D32)
            .aspect(ImageAspect::Depth)
            .usage(&[ImageUsage::DepthAttachment])
            .build(backend)?;
        let image_view = ImageViewBuilder::new()
            .image(image.handle())
            .build(backend)?;
        self.image = Some(image);
        self.image_view = Some(image_view);
        Ok(())
    }

    pub fn image_handle(&self) -> ImageHandle {
        self.image.as_ref().unwrap().handle()
    }

    pub fn image_view_handle(&self) -> ImageViewHandle {
        self.image_view.as_ref().unwrap().handle()
    }
}
