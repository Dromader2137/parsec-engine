use parsec_engine_error::ParsecError;
use parsec_engine_math::vec::{Vec2f, Vec3f};

use crate::{
    ActiveGraphicsBackend, buffer::BufferHandle, image::ImageViewHandle,
    renderpass::RenderpassHandle, sampler::SamplerHandle, shader::ShaderHandle,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PipelineHandle {
    id: u32,
}

impl PipelineHandle {
    pub fn new(id: u32) -> Self { Self { id } }
    pub fn id(&self) -> u32 { self.id }
}

#[derive(Debug)]
pub struct Pipeline {
    handle: PipelineHandle,
    resource_layouts: Vec<PipelineResourceLayoutHandle>,
    vertex_shader: ShaderHandle,
    fragment_shader: ShaderHandle,
    renderpass: RenderpassHandle,
    options: PipelineOptions,
}

impl Pipeline {
    fn new(
        handle: PipelineHandle,
        resource_layouts: Vec<PipelineResourceLayoutHandle>,
        vertex_shader: ShaderHandle,
        fragment_shader: ShaderHandle,
        renderpass: RenderpassHandle,
        options: PipelineOptions,
    ) -> Self {
        Self {
            handle,
            resource_layouts,
            vertex_shader,
            fragment_shader,
            renderpass,
            options,
        }
    }

    pub fn handle(&self) -> PipelineHandle { self.handle }
    pub fn id(&self) -> u32 { self.handle.id }
    pub fn destroy(
        self,
        backend: &mut ActiveGraphicsBackend,
    ) -> Result<(), PipelineError> {
        backend.delete_pipeline(self)
    }

    pub fn layouts(&self) -> &[PipelineResourceLayoutHandle] {
        &self.resource_layouts
    }
    pub fn vertex_shader(&self) -> ShaderHandle { self.vertex_shader }
    pub fn fragment_shader(&self) -> ShaderHandle { self.fragment_shader }
    pub fn renderpass(&self) -> RenderpassHandle { self.renderpass }
    pub fn options(&self) -> &PipelineOptions { &self.options }
}

pub struct PipelineBuilder {
    resource_layouts: Vec<PipelineResourceLayoutHandle>,
    vertex_shader: Option<ShaderHandle>,
    fragment_shader: Option<ShaderHandle>,
    renderpass: Option<RenderpassHandle>,
    options: PipelineOptions,
}

impl Default for PipelineBuilder {
    fn default() -> Self { Self::new() }
}

impl PipelineBuilder {
    pub fn new() -> Self {
        Self {
            resource_layouts: Vec::new(),
            vertex_shader: None,
            fragment_shader: None,
            renderpass: None,
            options: PipelineOptions::default(),
        }
    }

    pub fn resource_layout(
        mut self,
        resource_layout: PipelineResourceLayoutHandle,
    ) -> Self {
        self.resource_layouts.push(resource_layout);
        self
    }

    pub fn resource_layouts(
        mut self,
        resource_layouts: &[PipelineResourceLayoutHandle],
    ) -> Self {
        self.resource_layouts.extend_from_slice(resource_layouts);
        self
    }

    pub fn vertex_shader(mut self, shader: ShaderHandle) -> Self {
        self.vertex_shader = Some(shader);
        self
    }

    pub fn fragment_shader(mut self, shader: ShaderHandle) -> Self {
        self.fragment_shader = Some(shader);
        self
    }

    pub fn renderpass(mut self, renderpass: RenderpassHandle) -> Self {
        self.renderpass = Some(renderpass);
        self
    }

    pub fn cull_mode(mut self, cull_mode: PipelineCullingMode) -> Self {
        self.options.culling_mode = cull_mode;
        self
    }

    pub fn build(
        self,
        backend: &mut ActiveGraphicsBackend,
    ) -> Result<Pipeline, PipelineError> {
        let vs = self
            .vertex_shader
            .ok_or(PipelineError::VertexShaderNotSet)?;
        let fs = self
            .fragment_shader
            .ok_or(PipelineError::FragmentShaderNotSet)?;
        let renderpass =
            self.renderpass.ok_or(PipelineError::RenderpassNotSet)?;
        let handle = backend.create_pipeline(
            vs,
            fs,
            renderpass,
            &self.resource_layouts,
            self.options.clone(),
        )?;
        Ok(Pipeline::new(
            handle,
            self.resource_layouts,
            vs,
            fs,
            renderpass,
            self.options,
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PipelineResourceLayoutHandle {
    id: u32,
}

impl PipelineResourceLayoutHandle {
    pub fn new(id: u32) -> Self { Self { id } }
    pub fn id(&self) -> u32 { self.id }

    pub fn create_resource(
        &self,
        backend: &mut ActiveGraphicsBackend,
    ) -> Result<PipelineResource, PipelineError> {
        let handle = backend.create_pipeline_resource(*self)?;
        Ok(PipelineResource::new(handle, *self))
    }
}

#[derive(Debug)]
pub struct PipelineResourceLayout {
    handle: PipelineResourceLayoutHandle,
    bindings: Vec<PipelineResourceBindingLayout>,
}

impl PipelineResourceLayout {
    fn new(
        handle: PipelineResourceLayoutHandle,
        bindings: Vec<PipelineResourceBindingLayout>,
    ) -> Self {
        Self { handle, bindings }
    }

    pub fn handle(&self) -> PipelineResourceLayoutHandle { self.handle }
    pub fn id(&self) -> u32 { self.handle.id }
    pub fn destroy(
        self,
        backend: &mut ActiveGraphicsBackend,
    ) -> Result<(), PipelineError> {
        backend.delete_pipeline_resource_layout(self)
    }

    pub fn bindings(&self) -> &[PipelineResourceBindingLayout] {
        &self.bindings
    }

    pub fn create_resource(
        &self,
        backend: &mut ActiveGraphicsBackend,
    ) -> Result<PipelineResource, PipelineError> {
        self.handle.create_resource(backend)
    }
}

pub struct PipelineResourceLayoutBuilder {
    bindings: Vec<PipelineResourceBindingLayout>,
}

impl Default for PipelineResourceLayoutBuilder {
    fn default() -> Self { Self::new() }
}

impl PipelineResourceLayoutBuilder {
    pub fn new() -> Self {
        Self {
            bindings: Vec::new(),
        }
    }

    pub fn binding(mut self, binding: PipelineResourceBindingLayout) -> Self {
        self.bindings.push(binding);
        self
    }

    pub fn bindings(
        mut self,
        bindings: &[PipelineResourceBindingLayout],
    ) -> Self {
        self.bindings.extend_from_slice(bindings);
        self
    }

    pub fn build(
        self,
        backend: &mut ActiveGraphicsBackend,
    ) -> Result<PipelineResourceLayout, PipelineError> {
        let handle = backend.create_pipeline_resource_layout(&self.bindings)?;
        Ok(PipelineResourceLayout::new(handle, self.bindings))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PipelineResourceHandle {
    id: u32,
}

impl PipelineResourceHandle {
    pub fn new(id: u32) -> Self { Self { id } }
    pub fn id(&self) -> u32 { self.id }
}

#[derive(Debug)]
pub struct PipelineResource {
    handle: PipelineResourceHandle,
    layout: PipelineResourceLayoutHandle,
}

impl PipelineResource {
    fn new(
        handle: PipelineResourceHandle,
        layout: PipelineResourceLayoutHandle,
    ) -> Self {
        Self { handle, layout }
    }
    pub fn handle(&self) -> PipelineResourceHandle { self.handle }
    pub fn layout(&self) -> PipelineResourceLayoutHandle { self.layout }
    pub fn id(&self) -> u32 { self.handle.id }
    pub fn destroy(
        self,
        backend: &mut ActiveGraphicsBackend,
    ) -> Result<(), PipelineError> {
        backend.delete_pipeline_resource(self)
    }

    pub fn bind_buffer(
        &self,
        backend: &mut ActiveGraphicsBackend,
        buffer: BufferHandle,
        binding_idx: u32,
    ) -> Result<(), PipelineError> {
        backend.bind_buffer(self.handle, buffer, binding_idx)
    }

    pub fn bind_sampler(
        &self,
        backend: &mut ActiveGraphicsBackend,
        sampler: SamplerHandle,
        image_view: ImageViewHandle,
        binding_idx: u32,
    ) -> Result<(), PipelineError> {
        backend.bind_sampler(self.handle, sampler, image_view, binding_idx)
    }
}

#[derive(Debug, Clone)]
pub struct PipelineResourceBindingLayout {
    pub binding_type: PipelineBindingType,
    pub shader_stages: Vec<PipelineShaderStage>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineBindingType {
    UniformBuffer,
    TextureSampler,
    StorageBuffer,
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

pub trait Vertex: Clone + Copy {
    fn fields() -> Vec<VertexField>;
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
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
    LayoutCreationError(ParsecError),
    PipelineCreationError(ParsecError),
    BindingCreationError(ParsecError),
    LayoutNotFound,
    ShaderNotFound,
    RenderpassNotFound,
    FramebufferNotFound,
    BindingLayoutNotFound,
    BindingNotFound,
    FragmentShaderNotSet,
    VertexShaderNotSet,
    RenderpassNotSet,
    ResourceLayoutNotFound,
    BufferNotFound,
    BufferBindError(ParsecError),
    ResourceNotFound,
    SamplerNotFound,
    ImageViewNotFound,
    SamplerBindError(ParsecError),
    PipelineNotFound,
    PipelineResourceDestructionError(ParsecError),
}
