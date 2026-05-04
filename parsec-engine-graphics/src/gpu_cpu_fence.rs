use parsec_engine_error::ParsecError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GpuToCpuFence {
    id: u32,
}

#[derive(Debug)]
pub enum GpuToCpuFenceError {
    GpuToCpuFenneCreationError(ParsecError),
    GpuToCpuFenceWaitError(ParsecError),
    GpuToCpuFenceResetError(ParsecError),
    GpuToCpuFenceDeletionError(ParsecError),
    GpuToCpuFenceNotFound,
}

impl GpuToCpuFence {
    pub fn new(id: u32) -> GpuToCpuFence { GpuToCpuFence { id } }

    pub fn id(&self) -> u32 { self.id }
}
