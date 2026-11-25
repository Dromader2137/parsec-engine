use crate::graphics::backend::BackendError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GpuToGpuFence {
    id: u32,
}

impl GpuToGpuFence {
    pub fn new(id: u32) -> GpuToGpuFence { GpuToGpuFence { id } }

    pub fn id(&self) -> u32 { self.id }
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum GpuToGpuFenceError {
    #[error(transparent)]
    Backend(#[from] BackendError),
}
