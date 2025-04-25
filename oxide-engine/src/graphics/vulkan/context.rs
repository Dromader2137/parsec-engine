use crate::graphics::window::WindowWrapper;

use super::{device::{Device, DeviceError}, image::{Image, ImageError, ImageView}, instance::{Instance, InstanceError}, physical_device::{PhysicalDevice, PhysicalDeviceError}, queue::{Queue, QueueError}, surface::{InitialSurface, Surface, SurfaceError}, swapchain::{Swapchain, SwapchainError}};

pub struct VulkanContext {
    instance: Instance,
    surface: Surface,
    physical_device: PhysicalDevice,
    device: Device,
    graphics_queue: Queue,
    swapchain: Swapchain,
    swapchain_images: Vec<Image>,
    swapchain_image_views: Vec<ImageView>
}

#[derive(Debug)]
pub enum VulkanError {
    InstanceError(InstanceError),
    PhysicalDeviceError(PhysicalDeviceError),
    SurfaceError(SurfaceError),
    DeviceError(DeviceError),
    QueueError(QueueError),
    SwapchainError(SwapchainError),
    ImageError(ImageError)
}

impl VulkanContext {
    pub fn new(
        window: &WindowWrapper,
    ) -> Result<VulkanContext, VulkanError> {
        let instance = Instance::new(window)?;
        let initial_surface = InitialSurface::new(&instance, window)?;
        let physical_device = PhysicalDevice::new(&instance, &initial_surface)?;
        let surface = initial_surface.into_surface(&physical_device)?;
        let device = Device::new(&instance, &physical_device)?;
        let graphics_queue = device.get_present_queue(physical_device.get_queue_family_index());
        let swapchain = Swapchain::new(&instance, &surface, &physical_device, &device, window)?;
        let swapchain_images = swapchain.get_images()?; 
        let swapchain_format = surface.format().into();
        let swapchain_image_views = {
            let mut out = Vec::new();
            for img in swapchain_images.iter() {
                let view = ImageView::from_image(&device, img, swapchain_format)?;
                out.push(view);
            }
            out
        };

        Ok(VulkanContext { instance, surface, physical_device, device, graphics_queue, swapchain, swapchain_images, swapchain_image_views })
    }
}
