use std::sync::Arc;

use crate::graphics::window::WindowWrapper;

#[derive(Debug)]
pub struct VulkanContext {
    instance: Arc<vulkano::instance::Instance>,
    physical_device: Arc<vulkano::device::physical::PhysicalDevice>,
    device: Arc<vulkano::device::Device>,
    queue: Arc<vulkano::device::Queue>,
    surface: Arc<vulkano::swapchain::Surface>,
    allocator: Arc<vulkano::memory::allocator::StandardMemoryAllocator>,
    command_buffer_allocator: Arc<vulkano::command_buffer::allocator::StandardCommandBufferAllocator>,
    descriptor_set_allocator: Arc<vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator>,
}

#[derive(Debug, Clone)]
pub enum VulkanContextError {
    LibraryLoadingError,
    InstanceError(String),
    PhysicalDeviceError(String),
    DeviceError(String),
    SurfaceError(String),
}

impl VulkanContext {
    pub fn new(event_loop: &winit::event_loop::ActiveEventLoop, window: &WindowWrapper) -> Result<VulkanContext, VulkanContextError> {
        let library = match vulkano::library::VulkanLibrary::new() {
            Ok(val) => val,
            Err(_) => {
                return Err(VulkanContextError::LibraryLoadingError)
            }
        };

        let requiered_extensions = vulkano::swapchain::Surface::required_extensions(event_loop);
        let extensions = vulkano::instance::InstanceExtensions {
            ext_debug_utils: true,
            ..requiered_extensions
        };

        let instance = match vulkano::instance::Instance::new(
            library,
            vulkano::instance::InstanceCreateInfo {
                flags: vulkano::instance::InstanceCreateFlags::ENUMERATE_PORTABILITY,
                enabled_extensions: extensions,
                ..Default::default()
            }
        ) {
            Ok(val) => val,
            Err(err) => {
                return Err(VulkanContextError::InstanceError(format!("{:?}", err)));
            }

        };

        let device_extensions = vulkano::device::DeviceExtensions {
            khr_swapchain: true,
            ..vulkano::device::DeviceExtensions::empty()
        };

        let surface = match window.get_surface(instance.clone()) {
            Ok(val) => val,
            Err(err) => {
                return Err(VulkanContextError::SurfaceError(format!("{:?}", err)));
            }
        };

        let (physical_device, queue_family_index) = match instance.enumerate_physical_devices() {
            Ok(val) => {
                match val
                    .filter(|p| {
                        p.supported_extensions().contains(&device_extensions)
                    })
                    .filter_map(|p| {
                        p.queue_family_properties()
                            .iter()
                            .enumerate()
                            .position(|(i, q)| {
                                q.queue_flags.intersects(vulkano::device::QueueFlags::GRAPHICS)
                                    && p.surface_support(i as u32, &surface).unwrap_or(false)
                            })
                            .map(|i| (p, i as u32))
                    })
                    .min_by_key(|(p, _)| {
                        match p.properties().device_type {
                            vulkano::device::physical::PhysicalDeviceType::DiscreteGpu => 0,
                            vulkano::device::physical::PhysicalDeviceType::IntegratedGpu => 1,
                            vulkano::device::physical::PhysicalDeviceType::VirtualGpu => 2,
                            vulkano::device::physical::PhysicalDeviceType::Cpu => 3,
                            vulkano::device::physical::PhysicalDeviceType::Other => 4,
                            _ => 5,
                        }
                    }) {
                    Some(val) => val,
                    None => return Err(VulkanContextError::PhysicalDeviceError("Physical device not found".to_string()))
                }
            },
            Err(err) => {
                return Err(VulkanContextError::PhysicalDeviceError(format!("{:?}", err)));
            }
        };

        let (device, mut queues) = match vulkano::device::Device::new(
            physical_device.clone(),
            vulkano::device::DeviceCreateInfo {
                enabled_extensions: device_extensions,
                queue_create_infos: vec![
                    vulkano::device::QueueCreateInfo {
                        queue_family_index,
                        ..Default::default()
                    }
                ],
                ..Default::default()
            }
        ) {
            Ok(val) => val,
            Err(err) => {
                return Err(VulkanContextError::DeviceError(format!("{:?}", err)));
            }
        };

        let queue = match queues.next() {
            Some(val) => val,
            None => {
                return Err(VulkanContextError::DeviceError("Queue not found".to_string()));
            }
        };

        let allocator = Arc::new(vulkano::memory::allocator::StandardMemoryAllocator::new_default(device.clone()));
        let command_buffer_allocator = Arc::new(
            vulkano::command_buffer::allocator::StandardCommandBufferAllocator::new(
                device.clone(), 
                vulkano::command_buffer::allocator::StandardCommandBufferAllocatorCreateInfo::default()
            )
        );
        let descriptor_set_allocator = Arc::new(
            vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator::new(
                device.clone(), 
                Default::default()
            )
        );

        let vulkan_context = VulkanContext {
            instance,
            physical_device,
            device,
            queue,
            allocator,
            surface,
            command_buffer_allocator,
            descriptor_set_allocator,
        };

        Ok(vulkan_context)
    }

    pub fn get_device(&self) -> Arc<vulkano::device::Device> {
        self.device.clone()
    }
    
    pub fn get_physical_device(&self) -> Arc<vulkano::device::physical::PhysicalDevice> {
        self.physical_device.clone()
    }
    
    pub fn get_surface(&self) -> Arc<vulkano::swapchain::Surface> {
        self.surface.clone()
    }

    pub fn get_allocator(&self) -> Arc<vulkano::memory::allocator::StandardMemoryAllocator> {
        self.allocator.clone()
    }

    pub fn get_command_buffer_allocator(&self) -> Arc<vulkano::command_buffer::allocator::StandardCommandBufferAllocator> {
        self.command_buffer_allocator.clone()
    }
    
    pub fn get_descriptor_set_allocator(&self) -> Arc<vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator> {
        self.descriptor_set_allocator.clone()
    }
    
    pub fn get_queue(&self) -> Arc<vulkano::device::Queue> {
        self.queue.clone()
    }

    pub fn get_instance(&self) -> Arc<vulkano::instance::Instance> {
        self.instance.clone()
    }
}
