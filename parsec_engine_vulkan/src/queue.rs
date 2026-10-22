use std::collections::HashMap;

use crate::{
    command_buffer::VulkanCommandBuffer, device::VulkanDevice,
    fence::VulkanFence, image::VulkanImage,
    pipeline_stage::VulkanPipelineStage, semaphore::VulkanSemaphore,
};

pub struct VulkanQueue {
    queue: ash::vk::Queue,
    internal_semaphore: Option<VulkanSemaphore>,
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
        VulkanQueue {
            queue: raw_queue,
            internal_semaphore: None,
        }
    }

    pub fn submit(
        &mut self,
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
        let mut wait_semaphores = wait_semaphores
            .iter()
            .map(|x| *x.get_semaphore_raw())
            .collect::<Vec<_>>();
        let mut signal_semaphores = signal_semaphores
            .iter()
            .map(|x| *x.get_semaphore_raw())
            .collect::<Vec<_>>();
        match self.internal_semaphore.clone() {
            Some(sem) => wait_semaphores.push(*sem.get_semaphore_raw()),
            None => {
                self.internal_semaphore =
                    Some(VulkanSemaphore::new(device).unwrap())
            },
        }
        signal_semaphores.push(
            *self
                .internal_semaphore
                .as_ref()
                .unwrap()
                .get_semaphore_raw(),
        );

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

        for image_state in command_buffers.iter().flat_map(|x| x.image_state())
        {
            if let Some(image) = image_map.get_mut(&image_state.0) {
                image.set_layout(image_state.1.last_layout);
            }
        }

        Ok(())
    }

    pub fn destroy(&mut self, device: &VulkanDevice) {
        if let Some(sem) = self.internal_semaphore.take() {
            sem.destroy(device);
        }
    }

    pub fn get_queue_raw(&self) -> &ash::vk::Queue { &self.queue }
}
