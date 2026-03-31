use crate::math::vec::{Vec2f, Vec3f};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pipeline {
    id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PipelineResourceLayout {
    id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PipelineResource {
    id: u32,
}

pub struct PipelineResourceBindingLayout {
    pub binding_type: PipelineBindingType,
    pub shader_stages: Vec<PipelineShaderStage>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineCullingMode {
    None,
    CullBack,
    CullFront,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VertexFieldFormat {
    Float,
    Vec2,
    Vec3,
    Vec4,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VertexField {
    pub format: VertexFieldFormat,
}

pub trait Vertex: Clone + Copy + bytemuck::NoUninit {
    fn fields() -> Vec<VertexField>;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::NoUninit)]
pub struct DefaultVertex {
    position: [f32; 3],
    normal: [f32; 3],
    tangent: [f32; 3],
    uv: [f32; 2],
}

impl Vertex for DefaultVertex {
    fn fields() -> Vec<VertexField> {
        vec![
            VertexField {
                format: VertexFieldFormat::Vec3,
            },
            VertexField {
                format: VertexFieldFormat::Vec3,
            },
            VertexField {
                format: VertexFieldFormat::Vec3,
            },
            VertexField {
                format: VertexFieldFormat::Vec2,
            },
        ]
    }
}

impl DefaultVertex {
    pub fn new(pos: Vec3f, nor: Vec3f, uv: Vec2f) -> DefaultVertex {
        DefaultVertex {
            position: [pos.x, pos.y, pos.z],
            normal: [nor.x, nor.y, nor.z],
            tangent: [0.0, 1.0, 0.0],
            uv: [uv.x, uv.y],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineVertexLayout {
    pub fields: Vec<VertexField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineOptions {
    pub vertex_layout: PipelineVertexLayout,
    pub culling_mode: PipelineCullingMode,
}

impl Default for PipelineOptions {
    fn default() -> Self {
        PipelineOptions {
            vertex_layout: PipelineVertexLayout {
                fields: DefaultVertex::fields(),
            },
            culling_mode: PipelineCullingMode::None,
        }
    }
}

impl PipelineOptions {
    pub fn new<V: Vertex>(culling_mode: PipelineCullingMode) -> Self {
        Self {
            vertex_layout: PipelineVertexLayout {
                fields: V::fields(),
            },
            culling_mode,
        }
    }
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

impl PipelineResourceLayout {
    pub fn new(id: u32) -> PipelineResourceLayout {
        PipelineResourceLayout { id }
    }

    pub fn id(&self) -> u32 { self.id }
}

impl PipelineResourceBindingLayout {
    pub fn new(
        binding_type: PipelineBindingType,
        shader_stages: &[PipelineShaderStage],
    ) -> PipelineResourceBindingLayout {
        PipelineResourceBindingLayout {
            binding_type,
            shader_stages: shader_stages.to_vec(),
        }
    }
}

impl PipelineResource {
    pub fn new(id: u32) -> PipelineResource { PipelineResource { id } }

    pub fn id(&self) -> u32 { self.id }
}
