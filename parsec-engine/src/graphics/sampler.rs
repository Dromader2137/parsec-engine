use crate::error::ParsecError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Sampler {
    id: u32,
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
    ImageViewNowFound,
}

impl Sampler {
    pub fn new(id: u32) -> Sampler { Sampler { id } }

    pub fn id(&self) -> u32 { self.id }
}
