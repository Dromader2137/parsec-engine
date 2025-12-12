#[derive(Debug)]
pub struct Fence {
    id: u32,
}

#[derive(Debug)]
pub enum FenceError {}

impl Fence {
    pub fn new(id: u32) -> Fence { Fence { id } }

    pub fn id(&self) -> u32 { self.id }
}
