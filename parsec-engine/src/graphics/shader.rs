#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Shader {
    id: u32,
}

#[derive(Debug)]
pub enum ShaderError {
    ShaderCreationError(anyhow::Error),
    ShaderDeletionError(anyhow::Error),
    ShaderNotFound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderType {
    Vertex,
    Fragment,
}

impl Shader {
    pub fn new(id: u32) -> Shader { Shader { id } }

    pub fn id(&self) -> u32 { self.id }
}
