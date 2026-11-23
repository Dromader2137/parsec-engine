use crate::graphics::{
    ActiveGraphicsBackend,
    image::{
        Image, ImageAspect, ImageBuilder, ImageError, ImageFormat, ImageHandle, ImageSize,
        ImageUsage, ImageView, ImageViewBuilder, ImageViewHandle,
    },
};

#[derive(Debug)]
pub struct DepthImage {
    image: Image,
    image_view: ImageView,
}

fn create_depth_image(
    backend: &mut ActiveGraphicsBackend,
    size: ImageSize,
) -> Result<(Image, ImageView), ImageError> {
    let image = ImageBuilder::new()
        .size(size)
        .format(ImageFormat::D32)
        .aspect(ImageAspect::Depth)
        .usage(&[ImageUsage::DepthAttachment])
        .build(backend)?;
    let image_view = ImageViewBuilder::new().image(image.handle()).build(backend)?;
    Ok((image, image_view))
}

impl DepthImage {
    pub fn new(
        backend: &mut ActiveGraphicsBackend,
        size: ImageSize,
    ) -> Result<DepthImage, ImageError> {
        let (image, image_view) = create_depth_image(backend, size)?;
        Ok(DepthImage { image, image_view })
    }

    pub fn recreate(
        &mut self,
        backend: &mut ActiveGraphicsBackend,
        size: ImageSize,
    ) -> Result<(), ImageError> {
        let (image, image_view) = create_depth_image(backend, size)?;
        std::mem::replace(&mut self.image_view, image_view).destroy(backend)?;
        std::mem::replace(&mut self.image, image).destroy(backend)?;
        Ok(())
    }

    pub fn image_handle(&self) -> ImageHandle { self.image.handle() }

    pub fn image_view_handle(&self) -> ImageViewHandle { self.image_view.handle() }
}
