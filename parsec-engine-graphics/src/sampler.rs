use parsec_engine_error::ParsecError;

use crate::ActiveGraphicsBackend;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SamplerHandle {
    id: u32,
}

impl SamplerHandle {
    pub fn new(id: u32) -> Self { Self { id } }
    pub fn id(&self) -> u32 { self.id }
}

#[derive(Debug)]
pub struct Sampler {
    handle: SamplerHandle,
}

impl Sampler {
    fn new(handle: SamplerHandle) -> Self { Self { handle } }

    pub fn handle(&self) -> SamplerHandle { self.handle }
    pub fn id(&self) -> u32 { self.handle.id }

    pub fn destroy(
        self,
        backend: &mut ActiveGraphicsBackend,
    ) -> Result<(), SamplerError> {
        backend.delete_image_sampler(self)
    }
}

pub struct SamplerBuilder;

impl Default for SamplerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SamplerBuilder {
    pub fn new() -> Self { Self }

    pub fn build(
        self,
        backend: &mut ActiveGraphicsBackend,
    ) -> Result<Sampler, SamplerError> {
        let handle = backend.create_image_sampler()?;
        Ok(Sampler::new(handle))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SamplerError {
    #[error("failed to create sampler: {0}")]
    SamplerCreationError(ParsecError),
    #[error("failed to delete sampler: {0}")]
    SamplerDeletionError(ParsecError),
    #[error("failed to bind sampler: {0}")]
    SamplerBindError(ParsecError),
    #[error("pipeline resource does not exist")]
    PipelineResourceNotFound,
    #[error("pipeline resource layout does not exist")]
    PipelineResourceLayoutNotFound,
    #[error("sampler does not exist")]
    SamplerNotFound,
    #[error("image view does not exist")]
    ImageViewNotFound,
}
