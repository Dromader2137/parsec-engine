use crate::error::ParsecError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GpuToGpuFence {
    id: u32,
}

#[derive(Debug)]
pub enum GpuToGpuFenceError {
    GpuToGpuFenceCreationError(ParsecError),
    GpuToGpuFenceDeletionError(ParsecError),
    GpuToGpuFenceNotFound,
}

impl GpuToGpuFence {
    pub fn new(id: u32) -> GpuToGpuFence { GpuToGpuFence { id } }

    pub fn id(&self) -> u32 { self.id }
}
