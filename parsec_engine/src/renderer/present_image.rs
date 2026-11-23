use crate::graphics::{
    ActiveGraphicsBackend,
    image::{ImageError, ImageHandle, ImageView, ImageViewBuilder, ImageViewHandle},
};

#[derive(Debug)]
pub struct PresentImage {
    image_handle: ImageHandle,
    image_view: ImageView,
}

impl PresentImage {
    pub fn new(
        backend: &mut ActiveGraphicsBackend,
        image_handle: ImageHandle,
    ) -> Result<PresentImage, ImageError> {
        let image_view = ImageViewBuilder::new().image(image_handle).build(backend)?;
        Ok(PresentImage { image_handle, image_view })
    }

    pub fn recreate(
        &mut self,
        backend: &mut ActiveGraphicsBackend,
        image_handle: ImageHandle,
    ) -> Result<(), ImageError> {
        let image_view = ImageViewBuilder::new().image(image_handle).build(backend)?;
        std::mem::replace(&mut self.image_view, image_view).destroy(backend)?;
        self.image_handle = image_handle;
        Ok(())
    }

    pub fn image_handle(&self) -> ImageHandle { self.image_handle }

    pub fn image_view_handle(&self) -> ImageViewHandle { self.image_view.handle() }
}
