use std::sync::Arc;

use crate::graphics::vulkan::device::VulkanDevice;

pub struct Allocator {
    pub device: Arc<VulkanDevice>,
}
