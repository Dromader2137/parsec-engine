use crate::graphics::vulkan::{
    VulkanError, command_buffer::CommandBuffer, device::Device, fence::Fence,
    semaphore::Semaphore,
};

pub struct Queue {
    device_id: u32,
    queue: ash::vk::Queue,
}

#[derive(Debug)]
pub enum QueueError {
    SubmitError(ash::vk::Result),
    DeviceMismatch,
}

impl From<QueueError> for VulkanError {
    fn from(value: QueueError) -> Self { VulkanError::QueueError(value) }
}

impl Queue {
    pub fn present(device: &Device, family_index: u32) -> Queue {
        let raw_queue = unsafe {
            device.get_device_raw().get_device_queue(family_index, 0)
        };
        Queue {
            device_id: device.id(),
            queue: raw_queue,
        }
    }

    pub fn submit(
        &self,
        device: &Device,
        wait_semaphores: &[&Semaphore],
        signal_semaphores: &[&Semaphore],
        command_buffers: &[&CommandBuffer],
        submit_fence: &Fence,
    ) -> Result<(), QueueError> {
        if device.id() != self.device_id {
            return Err(QueueError::DeviceMismatch);
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
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&[
                ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            ])
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores);

        unsafe {
            device
                .get_device_raw()
                .queue_submit(
                    self.queue,
                    &[submit_info],
                    *submit_fence.get_fence_raw(),
                )
                .map_err(|err| QueueError::SubmitError(err))
        }
    }

    pub fn get_queue_raw(&self) -> &ash::vk::Queue { &self.queue }

    pub fn device_id(&self) -> u32 { self.device_id }
}
