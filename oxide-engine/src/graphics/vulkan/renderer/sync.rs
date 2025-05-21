use crate::graphics::vulkan::{
    VulkanError, context::VulkanContext, device::Device, fence::Fence, semaphore::Semaphore,
};

pub struct VulkanRendererSync {
    pub command_buffer_fences: Vec<Fence>,
    pub images_in_flight_fences: Vec<Option<Fence>>,
    pub rendering_semaphores: Vec<Semaphore>,
    pub present_semaphores: Vec<Semaphore>,
}

impl VulkanRendererSync {
    pub fn new(
        context: &VulkanContext,
        frames_in_flight: u32,
        swapchain_image_count: u32
    ) -> Result<VulkanRendererSync, VulkanError> {
        let command_buffer_fences = {
            let mut out = Vec::new();
            for _ in 0..frames_in_flight {
                out.push(Fence::new(&context.device, true)?);
            }
            out
        };
        let images_in_flight_fences = {
            let mut out = Vec::new();
            for _ in 0..swapchain_image_count {
                out.push(None);
            }
            out
        };
        let rendering_semaphores = {
            let mut out = Vec::new();
            for _ in 0..frames_in_flight {
                out.push(Semaphore::new(&context.device)?);
            }
            out
        };
        let present_semaphores = {
            let mut out = Vec::new();
            for _ in 0..frames_in_flight {
                out.push(Semaphore::new(&context.device)?);
            }
            out
        };

        Ok(VulkanRendererSync {
            command_buffer_fences,
            images_in_flight_fences,
            rendering_semaphores,
            present_semaphores,
        })
    }

    pub fn cleanup(&self, device: &Device) {
        self.present_semaphores
            .iter()
            .for_each(|x| x.cleanup(device));
        self.rendering_semaphores
            .iter()
            .for_each(|x| x.cleanup(device));
        self.command_buffer_fences
            .iter()
            .for_each(|x| x.cleanup(device));
        self.images_in_flight_fences
            .iter()
            .for_each(|x| if let Some(y) = x { y.cleanup(device); });
    }
}
