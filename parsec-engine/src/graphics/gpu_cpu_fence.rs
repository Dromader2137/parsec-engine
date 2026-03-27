#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GpuToCpuFence {
    id: u32,
}

#[derive(Debug)]
pub enum GpuToCpuFenceError {
    GpuToCpuFenneCreationError(anyhow::Error),
    GpuToCpuFenceWaitError(anyhow::Error),
    GpuToCpuFenceResetError(anyhow::Error),
    GpuToCpuFenceDeletionError(anyhow::Error),
    GpuToCpuFenceNotFound,
}

impl GpuToCpuFence {
    pub fn new(id: u32) -> GpuToCpuFence { GpuToCpuFence { id } }

    pub fn id(&self) -> u32 { self.id }
}
