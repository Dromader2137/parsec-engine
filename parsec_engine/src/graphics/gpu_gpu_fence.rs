use crate::graphics::backend::BackendError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GpuToGpuFence {
    id: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum GpuToGpuFenceError {
    #[error(transparent)]
    Backend(#[from] BackendError),
}

impl GpuToGpuFence {
    pub fn new(id: u32) -> GpuToGpuFence { GpuToGpuFence { id } }

    pub fn id(&self) -> u32 { self.id }
}
