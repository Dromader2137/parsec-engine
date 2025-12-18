#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Sampler {
    id: u32,
}

#[derive(Debug)]
pub enum SamplerError {
    SamplerCreationError(anyhow::Error),
    SamplerDeletionError(anyhow::Error),
    SamplerBindError(anyhow::Error),
    PipelineBindingNotFound,
    PipelineBindingLayoutNotFound,
    SamplerNotFound,
    ImageViewNowFound,
}

impl Sampler {
    pub fn new(id: u32) -> Sampler { Sampler { id } }

    pub fn id(&self) -> u32 { self.id }
}
