use crate::{
    arena::handle::{Handle, WeakHandle},
    graphics::vulkan::{
        VulkanBackend, buffer::VulkanBuffer, command_buffer::VulkanCommandPool, physical_device::VulkanPhysicalDevice, surface::VulkanSurface
    },
};

pub struct VulkanDevice {
    physical_device_handle: Handle<VulkanPhysicalDevice>,
    surface_handle: Handle<VulkanSurface>,
    buffer_handles: Vec<Handle<VulkanBuffer>>,
    command_pool_handles: Vec<Handle<VulkanCommandPool>>,
    device: ash::Device,
    memory_properties: ash::vk::PhysicalDeviceMemoryProperties,
}

#[derive(Debug, thiserror::Error)]
pub enum VulkanDeviceError {
    #[error("Failed to create device: {0}")]
    DeviceCreationError(ash::vk::Result),
    #[error("Failed to wait for device to be idle: {0}")]
    WaitIdleError(ash::vk::Result),
    #[error("Device created on different physical device")]
    PhysicalDeviceMismatch,
}

impl VulkanDevice {
    pub fn new(
        arenas: &mut VulkanBackend,
        physical_device_handle: Handle<VulkanPhysicalDevice>,
        surface_handle: Handle<VulkanSurface>,
    ) -> Result<WeakHandle<VulkanDevice>, VulkanDeviceError> {
        let physical_device = arenas
            .physical_devices
            .get_mut(physical_device_handle.clone());
        let surface = arenas.surfaces.get_mut(surface_handle.clone());

        if surface.instance() != physical_device.instance() {
            return Err(VulkanDeviceError::PhysicalDeviceMismatch);
        }

        let instance = arenas.instances.get(physical_device.instance());

        let device_extension_names_raw = [ash::khr::swapchain::NAME.as_ptr()];

        let features = ash::vk::PhysicalDeviceFeatures {
            shader_clip_distance: 1,
            ..Default::default()
        };

        let priorities = [1.0];

        let queue_info = ash::vk::DeviceQueueCreateInfo::default()
            .queue_family_index(physical_device.queue_family_index())
            .queue_priorities(&priorities);

        let device_create_info = ash::vk::DeviceCreateInfo::default()
            .queue_create_infos(std::slice::from_ref(&queue_info))
            .enabled_extension_names(&device_extension_names_raw)
            .enabled_features(&features);

        let device = unsafe {
            instance
                .raw_instance()
                .create_device(
                    *physical_device.raw_physical_device(),
                    &device_create_info,
                    None,
                )
                .map_err(|err| VulkanDeviceError::DeviceCreationError(err))?
        };

        let device = VulkanDevice {
            physical_device_handle,
            surface_handle,
            buffer_handles: Vec::new(),
            command_pool_handles: Vec::new(),
            device,
            memory_properties: physical_device.raw_physical_memory_properties(),
        };

        let handle = arenas.devices.add(device);
        physical_device.add_device(handle.clone());
        surface.add_device(handle.clone());
        Ok(handle.downgrade())
    }

    pub fn wait_idle(&self) -> Result<(), VulkanDeviceError> {
        if let Err(err) = unsafe { self.device.device_wait_idle() } {
            return Err(VulkanDeviceError::WaitIdleError(err));
        }
        Ok(())
    }

    pub fn raw_device(&self) -> &ash::Device { &self.device }

    pub fn raw_memory_properties(
        &self,
    ) -> ash::vk::PhysicalDeviceMemoryProperties {
        self.memory_properties
    }

    pub fn physical_device_handle(&self) -> Handle<VulkanPhysicalDevice> {
        self.physical_device_handle.clone()
    }

    pub fn surface_handle(&self) -> Handle<VulkanSurface> {
        self.surface_handle.clone()
    }

    pub fn add_buffer(&mut self, buffer: Handle<VulkanBuffer>) {
        self.buffer_handles.push(buffer);
    }

    pub fn buffer_handles(&self) -> &[Handle<VulkanBuffer>] {
        &self.buffer_handles
    }

    pub fn buffer_handles_mut(&mut self) -> &mut [Handle<VulkanBuffer>] {
        &mut self.buffer_handles
    }
    
    pub fn add_command_pool(&mut self, command_pool: Handle<VulkanCommandPool>) {
        self.command_pool_handles.push(command_pool);
    }

    pub fn command_pool_handles(&self) -> &[Handle<VulkanCommandPool>] {
        &self.command_pool_handles
    }

    pub fn command_pool_handles_mut(&mut self) -> &mut [Handle<VulkanCommandPool>] {
        &mut self.command_pool_handles
    }
}
