#[derive(Debug)]
pub struct CommandList {
    id: u32,
}

#[derive(Debug)]
pub enum CommandListError {
    CommandListNotFound,
}

impl CommandList {
    pub fn new(id: u32) -> CommandList { CommandList { id } }

    pub fn id(&self) -> u32 { self.id }
}
