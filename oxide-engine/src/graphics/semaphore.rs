#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Semaphore {
    id: u32,
}

#[derive(Debug)]
pub enum SemaphoreError {
    SemaphoreCreationError(anyhow::Error),
    SemaphoreDeletionError(anyhow::Error),
    SemaphoreNotFound,
}

impl Semaphore {
    pub fn new(id: u32) -> Semaphore { Semaphore { id } }

    pub fn id(&self) -> u32 { self.id }
}
