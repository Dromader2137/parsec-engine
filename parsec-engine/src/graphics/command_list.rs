use crate::{
    graphics::{
        buffer::Buffer,
        framebuffer::Framebuffer,
        image::Image,
        pipeline::{Pipeline, PipelineBinding},
        renderpass::Renderpass,
    },
    math::{ivec::Vec2i, uvec::Vec2u},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandList {
    id: u32,
    commands: Vec<Command>,
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
    CommandListCopyToBufferError(anyhow::Error),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Begin,
    End,
    BeginRenderpass(Renderpass, Framebuffer),
    SetViewport(Vec2u),
    SetScissor(Vec2u, Vec2i),
    EndRenderpass,
    BindGraphicsPipeline(Pipeline),
    BindPipelineBinding(PipelineBinding, u32),
    BindVertexBuffer(Buffer),
    BindIndexBuffer(Buffer),
    Draw(u32, u32, u32, u32),
    DrawIndexed(u32, u32, u32, i32, u32),
    CopyBufferToImage(Buffer, Image),
    CopyBufferToBuffer(Buffer, Buffer),
}

pub struct ImageBarrier {}

impl CommandList {
    pub fn new(id: u32) -> CommandList {
        CommandList {
            id,
            commands: Vec::new(),
        }
    }

    pub fn reset(&mut self) { self.commands.clear(); }

    pub fn cmd(&mut self, command: Command) { self.commands.push(command); }

    pub fn id(&self) -> u32 { self.id }

    pub fn commands(&self) -> &[Command] { &self.commands }
}
