#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Swapchain {
    id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SwapchainError {
    SwapchainOutOfDate,
    Undefined
}

impl Swapchain {
    pub fn new(id: u32) -> Swapchain { Swapchain { id } }

    pub fn id(&self) -> u32 { self.id }
}
