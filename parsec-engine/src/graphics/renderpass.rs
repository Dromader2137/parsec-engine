use crate::{graphics::image::ImageFormat, error::ParsecError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Renderpass {
    id: u32,
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

impl Renderpass {
    pub fn new(id: u32) -> Renderpass { Renderpass { id } }

    pub fn id(&self) -> u32 { self.id }
}
