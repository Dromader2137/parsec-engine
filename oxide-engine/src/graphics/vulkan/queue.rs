use super::{command_buffer::CommandBuffer, context::VulkanError, device::Device, fence::Fence, semaphore::Semaphore};

pub struct Queue {
    queue: ash::vk::Queue
}

#[derive(Debug)]
pub enum QueueError {
    SubmitError(ash::vk::Result)
}

impl From<QueueError> for VulkanError {
    fn from(value: QueueError) -> Self {
        VulkanError::QueueError(value)
    }
}

impl Queue {
    pub fn new(raw_queue: ash::vk::Queue) -> Queue {
        Queue { queue: raw_queue }
    }

    pub fn submit(&self, device: &Device, wait_semaphores: &[&Semaphore], signal_semaphores: &[&Semaphore], command_buffers: &[&CommandBuffer], submit_fence: &Fence) -> Result<(), QueueError> {
        let command_buffers = command_buffers.iter().map(|x| *x.get_command_buffer_raw()).collect::<Vec<_>>();
        let wait_semaphores = wait_semaphores.iter().map(|x| *x.get_semaphore_raw()).collect::<Vec<_>>();
        let signal_semaphores = signal_semaphores.iter().map(|x| *x.get_semaphore_raw()).collect::<Vec<_>>();
 
        let submit_info = ash::vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&[ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores);

        if let Err(err) = unsafe { device.get_device_raw().queue_submit(self.queue, &[submit_info], *submit_fence.get_fence_raw()) } {
            return Err(QueueError::SubmitError(err));
        }

        Ok(())
    }

    pub fn get_queue_raw(&self) -> &ash::vk::Queue {
        &self.queue
    }
}
