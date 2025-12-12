#[derive(Debug)]
pub struct Shader {
    id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderError {
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
