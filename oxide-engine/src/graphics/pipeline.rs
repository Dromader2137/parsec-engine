#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pipeline {
    id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PipelineBindingLayout {
    id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PipelineBinding {
    id: u32,
}

pub struct PipelineSubbindingLayout {
    pub binding_type: PipelineBindingType,
    pub shader_stage: PipelineShaderStage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineBindingType {
    UniformBuffer,
    TextureSampler,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineShaderStage {
    Vertex,
    Fragment,
}

#[derive(Debug)]
pub enum PipelineError {
    LayoutCreationError(anyhow::Error),
    PipelineCreationError(anyhow::Error),
    BindingCreationError(anyhow::Error),
    LayoutNotFound,
    ShaderNotFound,
    RenderpassNotFound,
    FramebufferNotFound,
    BindingLayoutNotFound,
    BindingNotFound,
}

impl Pipeline {
    pub fn new(id: u32) -> Pipeline { Pipeline { id } }

    pub fn id(&self) -> u32 { self.id }
}

impl PipelineBindingLayout {
    pub fn new(id: u32) -> PipelineBindingLayout {
        PipelineBindingLayout { id }
    }

    pub fn id(&self) -> u32 { self.id }
}

impl PipelineSubbindingLayout {
    pub fn new(
        binding_type: PipelineBindingType,
        shader_stage: PipelineShaderStage,
    ) -> PipelineSubbindingLayout {
        PipelineSubbindingLayout {
            binding_type,
            shader_stage,
        }
    }
}

impl PipelineBinding {
    pub fn new(id: u32) -> PipelineBinding { PipelineBinding { id } }

    pub fn id(&self) -> u32 { self.id }
}
