#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Fence {
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

impl Fence {
    pub fn new(id: u32) -> Fence { Fence { id } }

    pub fn id(&self) -> u32 { self.id }
}
