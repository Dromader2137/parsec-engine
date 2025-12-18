use ash::vk::Extent2D;

use crate::{
    graphics::{
        vulkan::{
            device::VulkanDevice, image::VulkanImageView,
            renderpass::VulkanRenderpass,
        },
        window::Window,
    },
    utils::id_counter::IdCounter,
};

pub struct VulkanFramebuffer {
    id: u32,
    device_id: u32,
    renderpass_id: u32,
    _image_view_ids: Vec<u32>,
    framebuffer: ash::vk::Framebuffer,
    extent: ash::vk::Extent2D,
}

#[derive(Debug, thiserror::Error)]
pub enum VulkanFramebufferError {
    #[error("Failed to create framebuffer: {0}")]
    CreationError(ash::vk::Result),
    #[error("Framebuffer created on different device")]
    DeviceMismatch,
}

static ID_COUNTER: once_cell::sync::Lazy<IdCounter> =
    once_cell::sync::Lazy::new(|| IdCounter::new(0));
impl VulkanFramebuffer {
    pub fn new(
        window: &Window,
        device: &VulkanDevice,
        image_view: &VulkanImageView,
        depth_view: &VulkanImageView,
        renderpass: &VulkanRenderpass,
    ) -> Result<VulkanFramebuffer, VulkanFramebufferError> {
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
            Err(err) => return Err(VulkanFramebufferError::CreationError(err)),
        };

        Ok(VulkanFramebuffer {
            id: ID_COUNTER.next(),
            device_id: device.id(),
            renderpass_id: renderpass.id(),
            _image_view_ids: vec![image_view.id(), depth_view.id()],
            framebuffer,
            extent,
        })
    }

    pub fn delete_framebuffer(
        self,
        device: &VulkanDevice,
    ) -> Result<(), VulkanFramebufferError> {
        if self.device_id != device.id() {
            return Err(VulkanFramebufferError::DeviceMismatch);
        }

        unsafe {
            device
                .get_device_raw()
                .destroy_framebuffer(*self.get_framebuffer_raw(), None);
        }
        Ok(())
    }

    pub fn get_framebuffer_raw(&self) -> &ash::vk::Framebuffer {
        &self.framebuffer
    }

    pub fn get_extent_raw(&self) -> ash::vk::Extent2D { self.extent }

    pub fn id(&self) -> u32 { self.id }

    pub fn renderpass_id(&self) -> u32 { self.renderpass_id }

    pub fn _image_view_ids(&self) -> &[u32] { &self._image_view_ids }
}
