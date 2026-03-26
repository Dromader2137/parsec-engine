#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GpuToCpuFence {
    id: u32,
}

#[derive(Debug)]
pub enum FenceError {
    FenceCreationError(anyhow::Error),
    FenceWaitError(anyhow::Error),
    FenceResetError(anyhow::Error),
    FenceDeletionError(anyhow::Error),
    FenceNotFound,
}

impl GpuToCpuFence {
    pub fn new(id: u32) -> GpuToCpuFence { GpuToCpuFence { id } }

    pub fn id(&self) -> u32 { self.id }
}
