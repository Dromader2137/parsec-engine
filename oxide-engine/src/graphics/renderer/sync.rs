use crate::graphics::{backend::GraphicsBackend, fence::Fence, semaphore::Semaphore};

pub struct RendererFrameSync {
    pub command_buffer_fence: Fence,
    pub image_available_semaphore: Semaphore,
}

pub struct RendererImageSync {
    pub rendering_complete_semaphore: Semaphore,
}

impl RendererFrameSync {
    pub fn new(
        backend: &mut impl GraphicsBackend,
    ) -> RendererFrameSync {
        RendererFrameSync {
            command_buffer_fence: backend.create_fence(true),
            image_available_semaphore: backend.create_semaphore(),
        }
    }
}

impl RendererImageSync {
    pub fn new(
        backend: &mut impl GraphicsBackend,
    ) -> RendererImageSync {
        RendererImageSync {
            rendering_complete_semaphore: backend.create_semaphore(),
        }
    }
}
