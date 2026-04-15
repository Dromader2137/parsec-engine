use crate::{
    error::ParsecError, graphics::{
        buffer::BufferHandle,
        framebuffer::{Framebuffer, FramebufferHandle},
        image::Image,
        pipeline::{Pipeline, PipelineResource},
        renderpass::Renderpass,
    }, math::{ivec::Vec2i, uvec::Vec2u}
};

#[derive(Debug)]
pub struct CommandList {
    id: u32,
    commands: Vec<Command>,
}

#[derive(Debug)]
pub enum CommandListError {
    CommandListCreationError(ParsecError),
    CommandListBeginError(ParsecError),
    CommandListEndError(ParsecError),
    CommandListRenderpassBeginError(ParsecError),
    CommandListRenderpassEndError(ParsecError),
    CommandListDrawError(ParsecError),
    CommandListResetError(ParsecError),
    CommandListBindError(ParsecError),
    CommandListSubmitError(ParsecError),
    CommandListCopyToImageError(ParsecError),
    CommandListCopyToBufferError(ParsecError),
    CommandListBarrier(ParsecError),
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

#[derive(Debug)]
pub enum Command {
    Begin,
    End,
    BeginRenderpass(Renderpass, FramebufferHandle),
    SetViewport(Vec2u),
    SetScissor(Vec2u, Vec2i),
    EndRenderpass,
    BindGraphicsPipeline(Pipeline),
    BindPipelineBinding(PipelineResource, u32),
    BindVertexBuffer(BufferHandle),
    BindIndexBuffer(BufferHandle),
    Draw(u32, u32, u32, u32),
    DrawIndexed(u32, u32, u32, i32, u32),
    CopyBufferToImage(BufferHandle, Image, Vec2u, Vec2u),
    CopyBufferToBuffer(BufferHandle, BufferHandle),
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
