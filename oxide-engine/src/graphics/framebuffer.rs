#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Framebuffer {
    id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FramebufferError {}

impl Framebuffer {
    pub fn new(id: u32) -> Framebuffer { Framebuffer { id } }

    pub fn id(&self) -> u32 { self.id }
}
