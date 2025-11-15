use std::sync::Arc;

use crate::graphics::vulkan::{VulkanError, image::ImageView, renderpass::Renderpass};

pub struct Framebuffer {
    pub renderpass: Arc<Renderpass>,
    pub image_views: Vec<Arc<ImageView>>,
    framebuffer: ash::vk::Framebuffer,
    extent: ash::vk::Extent2D,
}

#[derive(Debug)]
pub enum FramebufferError {
    CreationError(ash::vk::Result),
    NotFound(u32),
}

impl From<FramebufferError> for VulkanError {
    fn from(value: FramebufferError) -> Self { VulkanError::FramebufferError(value) }
}

impl Framebuffer {
    pub fn new(
        image_view: Arc<ImageView>,
        depth_view: Arc<ImageView>,
        renderpass: Arc<Renderpass>,
    ) -> Result<Arc<Framebuffer>, FramebufferError> {
        let extent = renderpass.surface.current_extent();
        let framebuffer_attachments = [
            *image_view.get_image_view_raw(),
            *depth_view.get_image_view_raw(),
        ];
        let frame_buffer_create_info = ash::vk::FramebufferCreateInfo::default()
            .render_pass(*renderpass.get_renderpass_raw())
            .attachments(&framebuffer_attachments)
            .width(extent.width)
            .height(extent.height)
            .layers(1);

        let framebuffer = match unsafe {
            renderpass
                .device
                .get_device_raw()
                .create_framebuffer(&frame_buffer_create_info, None)
        } {
            Ok(val) => val,
            Err(err) => return Err(FramebufferError::CreationError(err)),
        };

        Ok(Arc::new(Framebuffer {
            renderpass,
            image_views: vec![image_view, depth_view],
            framebuffer,
            extent,
        }))
    }

    pub fn get_framebuffer_raw(&self) -> &ash::vk::Framebuffer { &self.framebuffer }

    pub fn get_extent_raw(&self) -> ash::vk::Extent2D { self.extent }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            self.renderpass
                .device
                .get_device_raw()
                .destroy_framebuffer(self.framebuffer, None)
        };
    }
}
