use crate::graphics::backend::BackendError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GpuToCpuFence {
    id: u32,
}

impl GpuToCpuFence {
    pub fn new(id: u32) -> GpuToCpuFence { GpuToCpuFence { id } }

    pub fn id(&self) -> u32 { self.id }
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum GpuToCpuFenceError {
    #[error(transparent)]
    Backend(#[from] BackendError),
}
