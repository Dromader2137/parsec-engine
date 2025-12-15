#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Buffer {
    id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BufferUsage {
    Uniform,
    Vertex,
    Index,
    TransferSrc,
    TransferDst
}

#[derive(Debug)]
pub enum BufferError {
    BufferCreationError(anyhow::Error),
    BufferUpdateError(anyhow::Error),
    BufferDeletionError(anyhow::Error),
    BufferNotFound,
}

impl Buffer {
    pub fn new(id: u32) -> Buffer { Buffer { id } }

    pub fn id(&self) -> u32 { self.id }
}
