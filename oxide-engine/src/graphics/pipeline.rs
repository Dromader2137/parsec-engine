#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pipeline {
    id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PipelineLayout {
    ids: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PipelineBinding {
    id: u32
}

pub struct PipelineLayoutBinding {
    pub binding_type: PipelineBindingType,
    pub shader_stage: PipelineShaderStage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineBindingType {
    UniformBuffer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineShaderStage {
    Vertex,
    Fragment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineError {
    ShaderNotFound,
    RenderpassNotFound,
    FramebufferNotFound,
}

impl Pipeline {
    pub fn new(id: u32) -> Pipeline { Pipeline { id } }

    pub fn id(&self) -> u32 { self.id }
}

impl PipelineLayout {
    pub fn new(ids: Vec<u32>) -> PipelineLayout { PipelineLayout { ids } }

    pub fn ids(&self) -> &[u32] { &self.ids }
}

impl PipelineBinding {
    pub fn new(id: u32) -> PipelineBinding { PipelineBinding { id } }

    pub fn id(&self) -> u32 { self.id }
}
