use super::context::VulkanError;

pub struct Queue {
    queue: ash::vk::Queue
}

#[derive(Debug)]
pub enum QueueError {}

impl From<QueueError> for VulkanError {
    fn from(value: QueueError) -> Self {
        VulkanError::QueueError(value)
    }
}

impl Queue {
    pub fn new(raw_queue: ash::vk::Queue) -> Queue {
        Queue { queue: raw_queue }
    }

    pub fn get_queue_raw(&self) -> &ash::vk::Queue {
        &self.queue
    }
}
