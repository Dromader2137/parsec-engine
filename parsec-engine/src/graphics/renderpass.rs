use crate::graphics::image::ImageFormat;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Renderpass {
    id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderpassAttachmentType {
    PresentColor,
    PresentDepth,
    StoreDepth,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenderpassClearValue {
    Color(f32, f32, f32, f32),
    Depth(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderpassAttachment {
    pub attachment_type: RenderpassAttachmentType,
    pub image_format: ImageFormat,
    pub clear_value: RenderpassClearValue,
}

#[derive(Debug)]
pub enum RenderpassError {
    RenderpassCreationError(anyhow::Error),
    RenderpassDeletionError(anyhow::Error),
    RenderpassNotFound,
}

impl Renderpass {
    pub fn new(id: u32) -> Renderpass { Renderpass { id } }

    pub fn id(&self) -> u32 { self.id }
}
