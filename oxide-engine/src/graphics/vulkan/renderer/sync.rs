use crate::graphics::vulkan::{
    VulkanError, context::VulkanContext, device::Device, fence::Fence, semaphore::Semaphore,
};

pub struct VulkanRendererSync {
    command_buffer_fences: Vec<Fence>,
    rendering_complete_semaphores: Vec<Semaphore>,
    image_available_semaphores: Vec<Semaphore>,
    pub frame_to_image_mapping: Vec<usize>
}

pub struct VulkanRendererFrameSyncBundle {
    pub command_buffer_fence: Fence,
    pub rendering_complete_semaphore: Semaphore,
    pub image_available_semaphore: Semaphore
}

impl VulkanRendererSync {
    pub fn new(
        context: &VulkanContext,
        frames_in_flight: usize,
        swapchain_image_count: usize
    ) -> Result<VulkanRendererSync, VulkanError> {
        let command_buffer_fences = {
            let mut out = Vec::new();
            for _ in 0..swapchain_image_count {
                out.push(Fence::new(&context.device, true)?);
            }
            out
        };
        let rendering_complete_semaphores = {
            let mut out = Vec::new();
            for _ in 0..swapchain_image_count {
                out.push(Semaphore::new(&context.device)?);
            }
            out
        };
        let image_available_semaphores = {
            let mut out = Vec::new();
            for _ in 0..swapchain_image_count {
                out.push(Semaphore::new(&context.device)?);
            }
            out
        };
        let frame_to_image_mapping = (0..frames_in_flight).collect();

        Ok(VulkanRendererSync {
            command_buffer_fences,
            rendering_complete_semaphores,
            image_available_semaphores,
            frame_to_image_mapping
        })
    }

    pub fn get_frame_sync_bundle(&self, frame_id: usize) -> VulkanRendererFrameSyncBundle {
        let image_id = self.frame_to_image_mapping[frame_id];
        VulkanRendererFrameSyncBundle { command_buffer_fence: self.command_buffer_fences[image_id].clone(), rendering_complete_semaphore: self.rendering_complete_semaphores[image_id].clone(), image_available_semaphore: self.image_available_semaphores[image_id].clone()  }
    }

    pub fn cleanup(&self, device: &Device) {
        self.image_available_semaphores
            .iter()
            .for_each(|x| x.cleanup(device));
        self.rendering_complete_semaphores
            .iter()
            .for_each(|x| x.cleanup(device));
        self.command_buffer_fences
            .iter()
            .for_each(|x| x.cleanup(device));
    }
}
