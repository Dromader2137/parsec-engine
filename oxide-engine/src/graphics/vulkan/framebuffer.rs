use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};

use ash::vk::Extent2D;

use crate::graphics::{
    vulkan::{
        VulkanError, device::Device, image::ImageView, renderpass::Renderpass,
        surface::Surface,
    },
    window::WindowWrapper,
};

pub struct Framebuffer {
    id: u32,
    renderpass_id: u32,
    image_view_ids: Vec<u32>,
    framebuffer: ash::vk::Framebuffer,
    extent: ash::vk::Extent2D,
}

#[derive(Debug)]
pub enum FramebufferError {
    CreationError(ash::vk::Result),
    NotFound(u32),
}

impl From<FramebufferError> for VulkanError {
    fn from(value: FramebufferError) -> Self {
        VulkanError::FramebufferError(value)
    }
}

impl Framebuffer {
    const ID_COUNTER: AtomicU32 = AtomicU32::new(0);

    pub fn new(
        window: &WindowWrapper,
        device: &Device,
        image_view: &ImageView,
        depth_view: &ImageView,
        renderpass: &Renderpass,
    ) -> Result<Framebuffer, FramebufferError> {
        let extent = window.size();
        let extent = Extent2D {
            width: extent.0,
            height: extent.1,
        };

        let framebuffer_attachments = [
            *image_view.get_image_view_raw(),
            *depth_view.get_image_view_raw(),
        ];
        let frame_buffer_create_info =
            ash::vk::FramebufferCreateInfo::default()
                .render_pass(*renderpass.get_renderpass_raw())
                .attachments(&framebuffer_attachments)
                .width(extent.width)
                .height(extent.height)
                .layers(1);

        let framebuffer = match unsafe {
            device
                .get_device_raw()
                .create_framebuffer(&frame_buffer_create_info, None)
        } {
            Ok(val) => val,
            Err(err) => return Err(FramebufferError::CreationError(err)),
        };

        let id = Self::ID_COUNTER.load(Ordering::Acquire);
        Self::ID_COUNTER.store(id + 1, Ordering::Release);

        Ok(Framebuffer {
            id,
            renderpass_id: renderpass.id(),
            image_view_ids: vec![image_view.id(), depth_view.id()],
            framebuffer,
            extent,
        })
    }

    pub fn get_framebuffer_raw(&self) -> &ash::vk::Framebuffer {
        &self.framebuffer
    }

    pub fn get_extent_raw(&self) -> ash::vk::Extent2D { self.extent }

    pub fn id(&self) -> u32 { self.id }

    pub fn renderpass_id(&self) -> u32 { self.renderpass_id }
}
