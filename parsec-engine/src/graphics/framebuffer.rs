#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Framebuffer {
    id: u32,
}

#[derive(Debug)]
pub enum FramebufferError {
    FramebufferCreationError(anyhow::Error),
    FramebufferDeletionError(anyhow::Error),
    ImageViewNotFound,
    RenderpassNotFound,
    FramebufferNotFound,
}

impl Framebuffer {
    pub fn new(id: u32) -> Framebuffer { Framebuffer { id } }

    pub fn id(&self) -> u32 { self.id }
}
