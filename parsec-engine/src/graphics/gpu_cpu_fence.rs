use crate::graphics::backend::BackendError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GpuToCpuFence {
    id: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum GpuToCpuFenceError {
    #[error(transparent)]
    Backend(#[from] BackendError),
}

impl GpuToCpuFence {
    pub fn new(id: u32) -> GpuToCpuFence { GpuToCpuFence { id } }

    pub fn id(&self) -> u32 { self.id }
}
