use parsec_engine_error::ParsecError;

use crate::{ActiveGraphicsBackend, image::ImageFormat};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RenderpassHandle {
    id: u32,
}

impl RenderpassHandle {
    pub fn new(id: u32) -> Self { Self { id } }
    pub fn id(&self) -> u32 { self.id }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Renderpass {
    handle: RenderpassHandle,
    attachments: Vec<RenderpassAttachment>,
}

impl Renderpass {
    fn new(
        handle: RenderpassHandle,
        attachments: Vec<RenderpassAttachment>,
    ) -> Self {
        Self {
            handle,
            attachments,
        }
    }

    pub fn handle(&self) -> RenderpassHandle { self.handle }
    pub fn id(&self) -> u32 { self.handle.id }
    pub fn attachments(&self) -> &[RenderpassAttachment] { &self.attachments }

    pub fn destroy(
        self,
        backend: &mut ActiveGraphicsBackend,
    ) -> Result<(), RenderpassError> {
        backend.delete_renderpass(self)
    }
}

pub struct RenderpassBuilder {
    attachments: Vec<RenderpassAttachment>,
}

impl Default for RenderpassBuilder {
    fn default() -> Self { Self::new() }
}

impl RenderpassBuilder {
    pub fn new() -> Self {
        Self {
            attachments: Vec::new(),
        }
    }

    pub fn attachment(mut self, attachment: RenderpassAttachment) -> Self {
        self.attachments.push(attachment);
        self
    }

    pub fn build(
        self,
        backend: &mut ActiveGraphicsBackend,
    ) -> Result<Renderpass, RenderpassError> {
        let handle = backend.create_renderpass(&self.attachments)?;
        Ok(Renderpass::new(handle, self.attachments))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderpassAttachmentType {
    Color,
    Depth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderpassAttachmentLoadOp {
    Load,
    Clear,
    DontCare,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderpassAttachmentStoreOp {
    Store,
    DontCare,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenderpassClearValue {
    Color(f32, f32, f32, f32),
    Depth(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderpassAttachment {
    pub attachment_type: RenderpassAttachmentType,
    pub load_op: RenderpassAttachmentLoadOp,
    pub store_op: RenderpassAttachmentStoreOp,
    pub image_format: ImageFormat,
    pub clear_value: RenderpassClearValue,
}

#[derive(Debug)]
pub enum RenderpassError {
    RenderpassCreationError(ParsecError),
    RenderpassDeletionError(ParsecError),
    RenderpassNotFound,
}
