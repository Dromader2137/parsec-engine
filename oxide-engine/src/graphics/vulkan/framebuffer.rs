use crate::graphics::window::WindowWrapper;

use super::{context::VulkanError, device::Device, image::ImageView, renderpass::Renderpass, surface::Surface};

pub struct Framebuffer {
    framebuffer: ash::vk::Framebuffer,
    extent: ash::vk::Extent2D
}

#[derive(Debug)]
pub enum FramebufferError {
    CreationError(ash::vk::Result)
}

impl From<FramebufferError> for VulkanError {
    fn from(value: FramebufferError) -> Self {
        VulkanError::FramebufferError(value)
    }
}

impl Framebuffer {
    pub fn new(surface: &Surface, device: &Device, image_view: &ImageView, renderpass: &Renderpass, window: &WindowWrapper) -> Result<Framebuffer, FramebufferError> {
        let extent = surface.current_extent(window);
        let framebuffer_attachments = [*image_view.get_image_view_raw()];
        let frame_buffer_create_info = ash::vk::FramebufferCreateInfo::default()
            .render_pass(*renderpass.get_renderpass_raw())
            .attachments(&framebuffer_attachments)
            .width(extent.width)
            .height(extent.height)
            .layers(1);

        let framebuffer = match unsafe { device.get_device_raw().create_framebuffer(&frame_buffer_create_info, None) } {
            Ok(val) => val,
            Err(err) => return Err(FramebufferError::CreationError(err))
        };

        Ok( Framebuffer { framebuffer, extent } )
    }

    pub fn get_framebuffer_raw(&self) -> &ash::vk::Framebuffer {
        &self.framebuffer
    }

    pub fn get_extent_raw(&self) -> ash::vk::Extent2D {
        self.extent
    }
}
