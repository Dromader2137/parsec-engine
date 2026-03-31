#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GpuToGpuFence {
    id: u32,
}

#[derive(Debug)]
pub enum GpuToGpuFenceError {
    GpuToGpuFenceCreationError(anyhow::Error),
    GpuToGpuFenceDeletionError(anyhow::Error),
    GpuToGpuFenceNotFound,
}

impl GpuToGpuFence {
    pub fn new(id: u32) -> GpuToGpuFence { GpuToGpuFence { id } }

    pub fn id(&self) -> u32 { self.id }
}
