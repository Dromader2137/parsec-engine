#[derive(Debug)]
pub struct Semaphore {
    id: u32,
}

#[derive(Debug)]
pub enum SemphoreError {}

impl Semaphore {
    pub fn new(id: u32) -> Semaphore { Semaphore { id } }

    pub fn id(&self) -> u32 { self.id }
}
