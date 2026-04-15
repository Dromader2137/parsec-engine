use crate::{
    error::ParsecError,
    graphics::{
        ActiveGraphicsBackend,
        image::{ImageHandle, ImageView, ImageViewBuilder, ImageViewHandle},
    },
};

#[derive(Debug)]
pub struct PresentImage {
    image_handle: ImageHandle,
    image_view: Option<ImageView>,
}

impl PresentImage {
    pub fn new(
        backend: &mut ActiveGraphicsBackend,
        image_handle: ImageHandle,
    ) -> Result<PresentImage, ParsecError> {
        let image_view =
            ImageViewBuilder::new().image(image_handle).build(backend)?;
        Ok(PresentImage {
            image_handle,
            image_view: Some(image_view),
        })
    }

    pub fn recreate(
        &mut self,
        backend: &mut ActiveGraphicsBackend,
        image_handle: ImageHandle,
    ) -> Result<(), ParsecError> {
        self.image_view.take().unwrap().destroy(backend);
        let image_view =
            ImageViewBuilder::new().image(image_handle).build(backend)?;
        self.image_handle = image_handle;
        self.image_view = Some(image_view);
        Ok(())
    }

    pub fn image_handle(&self) -> ImageHandle { self.image_handle }

    pub fn image_view_handle(&self) -> ImageViewHandle {
        self.image_view.as_ref().unwrap().handle()
    }
}
