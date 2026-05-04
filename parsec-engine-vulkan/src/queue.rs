use std::collections::HashMap;

use crate::{
    command_buffer::VulkanCommandBuffer, device::VulkanDevice,
    fence::VulkanFence, image::VulkanImage,
    pipeline_stage::VulkanPipelineStage, semaphore::VulkanSemaphore,
};

pub struct VulkanQueue {
    queue: ash::vk::Queue,
}

#[derive(Debug, thiserror::Error)]
pub enum VulkanQueueError {
    #[error("Failed to submit Queue: {0}")]
    SubmitError(ash::vk::Result),
}

impl VulkanQueue {
    pub fn present(device: &VulkanDevice, family_index: u32) -> VulkanQueue {
        let raw_queue =
            unsafe { device.raw_device().get_device_queue(family_index, 0) };
        VulkanQueue { queue: raw_queue }
    }

    pub fn submit(
        &self,
        device: &VulkanDevice,
        wait_semaphores: &[&VulkanSemaphore],
        signal_semaphores: &[&VulkanSemaphore],
        command_buffers: &[&VulkanCommandBuffer],
        submit_fence: &VulkanFence,
        wait_dst_stage: VulkanPipelineStage,
        image_map: &mut HashMap<u32, Box<dyn VulkanImage>>,
    ) -> Result<(), VulkanQueueError> {
        let raw_command_buffers = command_buffers
            .iter()
            .map(|x| *x.raw_command_buffer())
            .collect::<Vec<_>>();
        let wait_semaphores = wait_semaphores
            .iter()
            .map(|x| *x.get_semaphore_raw())
            .collect::<Vec<_>>();
        let signal_semaphores = signal_semaphores
            .iter()
            .map(|x| *x.get_semaphore_raw())
            .collect::<Vec<_>>();

        let pipeline_stage = &[wait_dst_stage.raw_pipeline_stage()];
        let submit_info = ash::vk::SubmitInfo::default()
            .wait_dst_stage_mask(pipeline_stage)
            .command_buffers(&raw_command_buffers)
            .wait_semaphores(&wait_semaphores)
            .signal_semaphores(&signal_semaphores);

        unsafe {
            device
                .raw_device()
                .queue_submit(
                    self.queue,
                    &[submit_info],
                    *submit_fence.get_fence_raw(),
                )
                .map_err(VulkanQueueError::SubmitError)?;
        }

        for image_state in
            command_buffers.iter().flat_map(|x| x.image_state())
        {
            if let Some(image) = image_map.get_mut(&image_state.0) {
                image.set_layout(image_state.1.last_layout);
            }
        }

        Ok(())
    }

    pub fn get_queue_raw(&self) -> &ash::vk::Queue { &self.queue }
}
