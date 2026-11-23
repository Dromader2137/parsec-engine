//! Module responsible for graphics.
//!
//! # Vulkan API abstraction layers:
//!
//! ## Bindings layer
//!
//! This layer is entirely provided by the [`ash`] crate.
//!
//! ## Wrapper layer
//!
//! Wrapper around the raw vulkan bindings provided by [`ash`] that provides error handling and
//! custom enums.
//!
//! ## Backend layer
//!
//! Graphics back-end trait that lets higher level abstractions use different graphics APIs.
//!
//! ## Renderer layer
//!
//! Renderer that automatically manages images, buffers, etc.
//!
//! Layers can only use types and functions provided by the layer directly below it

use std::{
    ops::{Deref, DerefMut},
    ptr::NonNull,
    thread::{self, ThreadId},
};

use crate::graphics::{
    backend::{BackendError, GraphicsBackend},
    buffer::BufferError,
    command_list::CommandListError,
    framebuffer::FramebufferError,
    gpu_cpu_fence::GpuToCpuFenceError,
    gpu_gpu_fence::GpuToGpuFenceError,
    image::ImageError,
    pipeline::PipelineError,
    renderpass::RenderpassError,
    sampler::SamplerError,
    shader_module::ShaderError,
    window::{Window, WindowError},
};

pub mod backend;
pub mod buffer;
pub mod command_list;
pub mod framebuffer;
pub mod gpu_cpu_fence;
pub mod gpu_gpu_fence;
pub mod image;
pub mod pipeline;
pub mod renderpass;
pub mod sampler;
pub mod shader_module;
pub mod window;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum GraphicsError {
    #[error("backend error: {0}")]
    Backend(#[from] BackendError),
    #[error("buffer error: {0}")]
    Buffer(#[from] BufferError),
    #[error("command list error: {0}")]
    CommandList(#[from] CommandListError),
    #[error("framebuffer error: {0}")]
    Framebuffer(#[from] FramebufferError),
    #[error("gpu to cpu fence error: {0}")]
    GpuToCpuFence(#[from] GpuToCpuFenceError),
    #[error("gpu to gpu fence error: {0}")]
    GpuToGpuFence(#[from] GpuToGpuFenceError),
    #[error("image error: {0}")]
    Image(#[from] ImageError),
    #[error("pipeline error: {0}")]
    Pipeline(#[from] PipelineError),
    #[error("renderpass error: {0}")]
    Renderpass(#[from] RenderpassError),
    #[error("sampler error: {0}")]
    Sampler(#[from] SamplerError),
    #[error("shader error: {0}")]
    Shader(#[from] ShaderError),
    #[error("window error: {0}")]
    Window(#[from] WindowError),
    #[error(transparent)]
    EventLoopAccess(#[from] EventLoopAccessError),
}

pub struct ActiveGraphicsBackend(Box<dyn GraphicsBackend>);

impl Deref for ActiveGraphicsBackend {
    type Target = Box<dyn GraphicsBackend>;
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for ActiveGraphicsBackend {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl ActiveGraphicsBackend {
    pub fn with_backend<T: GraphicsBackend>(window: &Window) -> Result<Self, BackendError> {
        let backend = T::init(window)?;
        Ok(Self(Box::new(backend)))
    }
}

pub struct ActiveEventLoop {
    raw_active_event_loop: NonNull<winit::event_loop::ActiveEventLoop>,
    thread_id: ThreadId,
}

unsafe impl Send for ActiveEventLoop {}
unsafe impl Sync for ActiveEventLoop {}

#[derive(Debug, thiserror::Error)]
#[error("Event loop can only be accessed from the main thread")]
pub struct EventLoopAccessError;

impl ActiveEventLoop {
    pub fn new(raw_active_event_loop: &winit::event_loop::ActiveEventLoop) -> Self {
        ActiveEventLoop {
            raw_active_event_loop: NonNull::from_ref(raw_active_event_loop),
            thread_id: thread::current().id(),
        }
    }

    pub fn raw_active_event_loop(
        &self,
    ) -> Result<&winit::event_loop::ActiveEventLoop, EventLoopAccessError> {
        if self.thread_id != thread::current().id() {
            return Err(EventLoopAccessError);
        }
        Ok(unsafe { self.raw_active_event_loop.as_ref() })
    }
}
