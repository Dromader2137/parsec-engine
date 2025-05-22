use std::sync::Arc;

use super::{
    VulkanError, command_buffer::CommandBuffer, device::Device, fence::Fence, semaphore::Semaphore,
};

pub struct Queue {
    pub device: Arc<Device>,
    queue: ash::vk::Queue,
}

#[derive(Debug)]
pub enum QueueError {
    SubmitError(ash::vk::Result),
}

impl From<QueueError> for VulkanError {
    fn from(value: QueueError) -> Self {
        VulkanError::QueueError(value)
    }
}

impl Queue {
    pub fn present(device: Arc<Device>, family_index: u32) -> Arc<Queue> {
        let raw_queue = unsafe { device.get_device_raw().get_device_queue(family_index, 0) };
        Arc::new(Queue { device, queue: raw_queue })
    }

    pub fn submit(
        &self,
        wait_semaphores: &[Arc<Semaphore>],
        signal_semaphores: &[Arc<Semaphore>],
        command_buffers: &[Arc<CommandBuffer>],
        submit_fence: Arc<Fence>,
    ) -> Result<(), QueueError> {
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
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&[ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores);

        if let Err(err) = unsafe {
            self.device.get_device_raw().queue_submit(
                self.queue,
                &[submit_info],
                *submit_fence.get_fence_raw(),
            )
        } {
            return Err(QueueError::SubmitError(err));
        }

        Ok(())
    }

    pub fn get_queue_raw(&self) -> &ash::vk::Queue {
        &self.queue
    }
}
