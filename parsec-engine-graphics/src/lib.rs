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

use parsec_engine_error::{ParsecError, StrError};

use crate::{backend::GraphicsBackend, window::Window};

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
pub mod shader;
pub mod window;

pub struct ActiveGraphicsBackend(Box<dyn GraphicsBackend>);

impl Deref for ActiveGraphicsBackend {
    type Target = Box<dyn GraphicsBackend>;
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for ActiveGraphicsBackend {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl ActiveGraphicsBackend {
    pub fn with_backend<T: GraphicsBackend>(
        window: &Window,
    ) -> Result<Self, ParsecError> {
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

impl ActiveEventLoop {
    pub fn new(
        raw_active_event_loop: &winit::event_loop::ActiveEventLoop,
    ) -> Self {
        ActiveEventLoop {
            raw_active_event_loop: NonNull::from_ref(raw_active_event_loop),
            thread_id: thread::current().id(),
        }
    }

    pub fn raw_active_event_loop(
        &self,
    ) -> Result<&winit::event_loop::ActiveEventLoop, ParsecError> {
        if self.thread_id != thread::current().id() {
            return Err(StrError(
                "Active event loop can only be accessed from the creating \
                 thread",
            )
            .into());
        }
        Ok(unsafe { self.raw_active_event_loop.as_ref() })
    }
}
