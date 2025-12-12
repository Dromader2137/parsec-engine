#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Renderpass {
    id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderpassError {}

impl Renderpass {
    pub fn new(id: u32) -> Renderpass { Renderpass { id } }

    pub fn id(&self) -> u32 { self.id }
}
