use std::sync::Arc;

use crate::graphics::vulkan::device::Device;

pub struct Allocator {
    pub device: Arc<Device>,
}   
