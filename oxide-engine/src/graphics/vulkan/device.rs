use super::{context::VulkanError, instance::Instance, physical_device::PhysicalDevice, queue::Queue};

pub struct Device {
    device: ash::Device,
}

#[derive(Debug)]
pub enum DeviceError {
    DeviceCreationError(ash::vk::Result),
}

impl From<DeviceError> for VulkanError {
    fn from(value: DeviceError) -> Self {
        VulkanError::DeviceError(value)
    }
}

impl Device {
    pub fn new(instance: &Instance, physical_device: &PhysicalDevice) -> Result<Device, DeviceError> {
        let device_extension_names_raw = [
            ash::khr::swapchain::NAME.as_ptr(),
        ];

        let features = ash::vk::PhysicalDeviceFeatures {
            shader_clip_distance: 1,
            ..Default::default()
        };
        
        let priorities = [1.0];

        let queue_info = ash::vk::DeviceQueueCreateInfo::default()
            .queue_family_index(physical_device.get_queue_family_index())
            .queue_priorities(&priorities);

        let device_create_info = ash::vk::DeviceCreateInfo::default()
            .queue_create_infos(std::slice::from_ref(&queue_info))
            .enabled_extension_names(&device_extension_names_raw)
            .enabled_features(&features);

        let device = match instance.create_device(physical_device, &device_create_info) {
            Ok(val) => val,
            Err(err) => return Err(DeviceError::DeviceCreationError(err))
        };

        Ok(Device { device })
    }

    pub fn get_present_queue(&self, family_index: u32) -> Queue {
        let raw_queue = unsafe { self.device.get_device_queue(family_index, 0) };
        Queue::new(raw_queue)
    }

    pub fn create_image_raw(&self, create_info: ash::vk::ImageCreateInfo) -> Result<ash::vk::Image, ash::vk::Result> {
        unsafe { self.device.create_image(&create_info, None) }    
    }

    pub fn create_image_view_raw(&self, create_info: ash::vk::ImageViewCreateInfo) -> Result<ash::vk::ImageView, ash::vk::Result> {
        unsafe { self.device.create_image_view(&create_info, None) }
    }

    pub fn create_render_pass(&self, create_info: ash::vk::RenderPassCreateInfo) -> Result<ash::vk::RenderPass, ash::vk::Result> {
        unsafe { self.device.create_render_pass(&create_info, None) }
    }
    
    pub fn create_framebuffer(&self, create_info: ash::vk::FramebufferCreateInfo) -> Result<ash::vk::Framebuffer, ash::vk::Result> {
        unsafe { self.device.create_framebuffer(&create_info, None) }
    }

    pub fn create_command_pool(&self, create_info: ash::vk::CommandPoolCreateInfo) -> Result<ash::vk::CommandPool, ash::vk::Result> {
        unsafe { self.device.create_command_pool(&create_info, None) }
    }

    pub fn create_command_buffers(&self, create_info: ash::vk::CommandBufferAllocateInfo) -> Result<Vec<ash::vk::CommandBuffer>, ash::vk::Result> {
        unsafe { self.device.allocate_command_buffers(&create_info) }
    }
    
    pub fn begin_command_buffer(&self, command_buffer: ash::vk::CommandBuffer, begin_info: ash::vk::CommandBufferBeginInfo) -> Result<(), ash::vk::Result> {
        unsafe { self.device.begin_command_buffer(command_buffer, &begin_info) }
    }
    
    pub fn end_command_buffer(&self, command_buffer: ash::vk::CommandBuffer) -> Result<(), ash::vk::Result> {
        unsafe { self.device.end_command_buffer(command_buffer) }
    }
    
    pub fn cmd_begin_render_pass(&self, command_buffer: ash::vk::CommandBuffer, begin_info: ash::vk::RenderPassBeginInfo, subpass_contents: ash::vk::SubpassContents) {
        unsafe { self.device.cmd_begin_render_pass(command_buffer, &begin_info, subpass_contents) }
    }

    pub fn cmd_end_render_pass(&self, command_buffer: ash::vk::CommandBuffer) {
        unsafe { self.device.cmd_end_render_pass(command_buffer) }
    }

    pub fn create_fence(&self, create_info: ash::vk::FenceCreateInfo) -> Result<ash::vk::Fence, ash::vk::Result> {
        unsafe { self.device.create_fence(&create_info, None) } 

    }

    pub fn create_semaphore(&self, create_info: ash::vk::SemaphoreCreateInfo) -> Result<ash::vk::Semaphore, ash::vk::Result> {
        unsafe { self.device.create_semaphore(&create_info, None) } 

    }

    pub fn get_device_raw(&self) -> &ash::Device {
        &self.device
    }
}
