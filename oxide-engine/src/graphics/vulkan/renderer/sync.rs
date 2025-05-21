use crate::graphics::vulkan::{
    VulkanError, context::VulkanContext, device::Device, fence::Fence, semaphore::Semaphore,
};

pub struct VulkanRendererSync {
<<<<<<< HEAD
    command_buffer_fences: Vec<Fence>,
    rendering_complete_semaphores: Vec<Semaphore>,
    image_available_semaphores: Vec<Semaphore>,
    pub frame_to_image_mapping: Vec<usize>
}

pub struct VulkanRendererFrameSyncBundle {
    pub command_buffer_fence: Fence,
    pub rendering_complete_semaphore: Semaphore,
    pub image_available_semaphore: Semaphore
=======
    pub command_buffer_fences: Vec<Fence>,
    pub images_in_flight_fences: Vec<Option<Fence>>,
    pub rendering_semaphores: Vec<Semaphore>,
    pub present_semaphores: Vec<Semaphore>,
>>>>>>> d9cb9d02e29a71a53cd606c09e2ed026f7170dc7
}

impl VulkanRendererSync {
    pub fn new(
        context: &VulkanContext,
<<<<<<< HEAD
        frames_in_flight: usize,
        swapchain_image_count: usize
=======
        frames_in_flight: u32,
        swapchain_image_count: u32
>>>>>>> d9cb9d02e29a71a53cd606c09e2ed026f7170dc7
    ) -> Result<VulkanRendererSync, VulkanError> {
        let command_buffer_fences = {
            let mut out = Vec::new();
            for _ in 0..swapchain_image_count {
                out.push(Fence::new(&context.device, true)?);
            }
            out
        };
<<<<<<< HEAD
        let rendering_complete_semaphores = {
=======
        let images_in_flight_fences = {
            let mut out = Vec::new();
            for _ in 0..swapchain_image_count {
                out.push(None);
            }
            out
        };
        let rendering_semaphores = {
>>>>>>> d9cb9d02e29a71a53cd606c09e2ed026f7170dc7
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
<<<<<<< HEAD
            rendering_complete_semaphores,
            image_available_semaphores,
            frame_to_image_mapping
        })
    }

    pub fn get_frame_sync_bundle(&self, frame_id: usize) -> VulkanRendererFrameSyncBundle {
        let image_id = self.frame_to_image_mapping[frame_id];
        VulkanRendererFrameSyncBundle { command_buffer_fence: self.command_buffer_fences[image_id].clone(), rendering_complete_semaphore: self.rendering_complete_semaphores[image_id].clone(), image_available_semaphore: self.image_available_semaphores[image_id].clone()  }
    }

=======
            images_in_flight_fences,
            rendering_semaphores,
            present_semaphores,
        })
    }

>>>>>>> d9cb9d02e29a71a53cd606c09e2ed026f7170dc7
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
        self.images_in_flight_fences
            .iter()
            .for_each(|x| if let Some(y) = x { y.cleanup(device); });
    }
}
