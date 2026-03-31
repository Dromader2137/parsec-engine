use crate::graphics::{
    ActiveGraphicsBackend, gpu_cpu_fence::GpuToCpuFence,
    gpu_gpu_fence::GpuToGpuFence,
};

pub struct RendererFrameSync {
    pub command_buffer_fence: GpuToCpuFence,
    pub image_available_semaphore: GpuToGpuFence,
}

pub struct RendererImageSync {
    pub rendering_complete_semaphore: GpuToGpuFence,
}

impl RendererFrameSync {
    pub fn new(backend: &mut ActiveGraphicsBackend) -> RendererFrameSync {
        RendererFrameSync {
            command_buffer_fence: backend
                .create_gpu_to_cpu_fence(true)
                .unwrap(),
            image_available_semaphore: backend
                .create_gpu_to_gpu_fence()
                .unwrap(),
        }
    }
}

impl RendererImageSync {
    pub fn new(backend: &mut ActiveGraphicsBackend) -> RendererImageSync {
        RendererImageSync {
            rendering_complete_semaphore: backend
                .create_gpu_to_gpu_fence()
                .unwrap(),
        }
    }
}
