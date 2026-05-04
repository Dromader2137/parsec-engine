use parsec_engine_error::ParsecError;
use parsec_engine_math::uvec::Vec2u;

use crate::{
    ActiveGraphicsBackend, image::ImageViewHandle, renderpass::RenderpassHandle,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FramebufferHandle {
    id: u32,
}

impl FramebufferHandle {
    pub fn new(id: u32) -> Self { Self { id } }
    pub fn id(&self) -> u32 { self.id }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Framebuffer {
    handle: FramebufferHandle,
    attachments: Vec<ImageViewHandle>,
    renderpass: RenderpassHandle,
    size: Vec2u,
}

impl Framebuffer {
    fn new(
        handle: FramebufferHandle,
        attachments: Vec<ImageViewHandle>,
        renderpass: RenderpassHandle,
        size: Vec2u,
    ) -> Self {
        Self {
            handle,
            attachments,
            renderpass,
            size,
        }
    }

    pub fn handle(&self) -> FramebufferHandle { self.handle }
    pub fn id(&self) -> u32 { self.handle.id }
    pub fn attachments(&self) -> &[ImageViewHandle] { &self.attachments }
    pub fn size(&self) -> Vec2u { self.size }
    pub fn renderpass(&self) -> RenderpassHandle { self.renderpass }

    pub fn destroy(
        self,
        backend: &mut ActiveGraphicsBackend,
    ) -> Result<(), FramebufferError> {
        backend.delete_framebuffer(self)
    }
}

pub struct FramebufferBuilder {
    attachments: Vec<ImageViewHandle>,
    size: Option<Vec2u>,
    renderpass: Option<RenderpassHandle>,
}

impl Default for FramebufferBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl FramebufferBuilder {
    pub fn new() -> Self {
        Self {
            attachments: Vec::new(),
            size: None,
            renderpass: None,
        }
    }

    pub fn attachment(mut self, view: ImageViewHandle) -> Self {
        self.attachments.push(view);
        self
    }

    pub fn size(mut self, size: Vec2u) -> Self {
        self.size = Some(size);
        self
    }

    pub fn renderpass(mut self, renderpass: RenderpassHandle) -> Self {
        self.renderpass = Some(renderpass);
        self
    }

    pub fn build(
        self,
        backend: &mut ActiveGraphicsBackend,
    ) -> Result<Framebuffer, FramebufferError> {
        let size = self.size.ok_or(FramebufferError::RenderpassSizeNotSet)?;
        let renderpass =
            self.renderpass.ok_or(FramebufferError::RenderpassNotSet)?;
        let handle =
            backend.create_framebuffer(size, &self.attachments, renderpass)?;
        Ok(Framebuffer::new(handle, self.attachments, renderpass, size))
    }
}

#[derive(Debug)]
pub enum FramebufferError {
    FramebufferCreationError(ParsecError),
    FramebufferDeletionError(ParsecError),
    ImageViewNotFound,
    RenderpassNotSet,
    FramebufferNotFound,
    ImageNotFound,
    RenderpassSizeNotSet,
}
