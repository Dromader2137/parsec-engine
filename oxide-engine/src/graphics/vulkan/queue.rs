use crate::graphics::vulkan::{
    command_buffer::VulkanCommandBuffer, device::VulkanDevice,
    fence::VulkanFence, semaphore::VulkanSemaphore,
};

pub struct VulkanQueue {
    device_id: u32,
    queue: ash::vk::Queue,
}

#[derive(Debug, thiserror::Error)]
pub enum VulkanQueueError {
    #[error("Failed to submit Queue: {0}")]
    SubmitError(ash::vk::Result),
    #[error("Queue created on a different Device")]
    DeviceMismatch,
}

impl VulkanQueue {
    pub fn present(device: &VulkanDevice, family_index: u32) -> VulkanQueue {
        let raw_queue = unsafe {
            device.get_device_raw().get_device_queue(family_index, 0)
        };
        VulkanQueue {
            device_id: device.id(),
            queue: raw_queue,
        }
    }

    pub fn submit(
        &self,
        device: &VulkanDevice,
        wait_semaphores: &[&VulkanSemaphore],
        signal_semaphores: &[&VulkanSemaphore],
        command_buffers: &[&VulkanCommandBuffer],
        submit_fence: &VulkanFence,
    ) -> Result<(), VulkanQueueError> {
        if device.id() != self.device_id {
            return Err(VulkanQueueError::DeviceMismatch);
        }

        let command_buffers = command_buffers
            .iter()
            .map(|x| *x.get_command_buffer_raw())
            .collect::<Vec<_>>();
        let wait_semaphores = wait_semaphores
            .iter()
            .map(|x| *x.get_semaphore_raw())
            .collect::<Vec<_>>();
        let signal_semaphores = signal_semaphores
            .iter()
            .map(|x| *x.get_semaphore_raw())
            .collect::<Vec<_>>();

        let submit_info = ash::vk::SubmitInfo::default()
            .wait_dst_stage_mask(&[
                ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            ])
            .command_buffers(&command_buffers)
            .wait_semaphores(&wait_semaphores)
            .signal_semaphores(&signal_semaphores);

        unsafe {
            device
                .get_device_raw()
                .queue_submit(
                    self.queue,
                    &[submit_info],
                    *submit_fence.get_fence_raw(),
                )
                .map_err(|err| VulkanQueueError::SubmitError(err))
        }
    }

    pub fn get_queue_raw(&self) -> &ash::vk::Queue { &self.queue }

    pub fn device_id(&self) -> u32 { self.device_id }
}
