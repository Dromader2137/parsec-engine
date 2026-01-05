#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommandList {
    id: u32,
}

#[derive(Debug)]
pub enum CommandListError {
    CommandListCreationError(anyhow::Error),
    CommandListBeginError(anyhow::Error),
    CommandListEndError(anyhow::Error),
    CommandListRenderpassBeginError(anyhow::Error),
    CommandListRenderpassEndError(anyhow::Error),
    CommandListDrawError(anyhow::Error),
    CommandListResetError(anyhow::Error),
    CommandListBindError(anyhow::Error),
    CommandListSubmitError(anyhow::Error),
    CommandListCopyToImageError(anyhow::Error),
    CommandListBarrier(anyhow::Error),
    CommandListNotFound,
    FramebufferNotFound,
    RenderpassNotFound,
    PipelineNotFound,
    PipelineLayoutNotFound,
    BufferNotFound,
    SemaphoreNotFound,
    FenceNotFound,
    ImageNotFound,
}

pub struct ImageBarrier {}

impl CommandList {
    pub fn new(id: u32) -> CommandList { CommandList { id } }

    pub fn id(&self) -> u32 { self.id }
}
