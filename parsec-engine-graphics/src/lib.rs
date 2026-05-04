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

use std::ops::{Deref, DerefMut};

use parsec_engine_error::ParsecError;

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
