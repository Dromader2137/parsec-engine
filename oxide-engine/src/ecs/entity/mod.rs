#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity {
    id: u32
}

impl Entity {
    pub fn new(id: u32) -> Entity {
        Entity { id }
    }
}
