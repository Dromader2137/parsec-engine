use crate::{
    graphics::vulkan::{
        device::VulkanDevice,
        image::{VulkanImage, VulkanImageView},
        renderpass::VulkanRenderpass,
    },
    math::uvec::Vec2u,
};

pub struct VulkanFramebuffer {
    id: u32,
    device_id: u32,
    renderpass_id: u32,
    attached_images_ids: Vec<u32>,
    dimensions: Vec2u,
    raw_framebuffer: ash::vk::Framebuffer,
}

#[derive(Debug, thiserror::Error)]
pub enum VulkanFramebufferError {
    #[error("Failed to create framebuffer: {0}")]
    CreationError(ash::vk::Result),
    #[error("Framebuffer created on different device")]
    DeviceMismatch,
}

crate::create_counter! {FRAMEBUFFER_ID_COUNTER}
impl VulkanFramebuffer {
    pub fn new(
        device: &VulkanDevice,
        attachments: &[(&VulkanImageView, &dyn VulkanImage)],
        renderpass: &VulkanRenderpass,
        dimensions: Vec2u,
    ) -> Result<VulkanFramebuffer, VulkanFramebufferError> {
        let mut attached_images_ids = Vec::new();
        let mut raw_attachments = Vec::new();
        for (attachment_view, attachment_image) in attachments.iter() {
            attached_images_ids.push(attachment_image.id());
            raw_attachments.push(*attachment_view.raw_image_view());
        }

        let frame_buffer_create_info =
            ash::vk::FramebufferCreateInfo::default()
                .render_pass(*renderpass.get_renderpass_raw())
                .attachments(&raw_attachments)
                .width(dimensions.x)
                .height(dimensions.y)
                .layers(1);

        let raw_framebuffer = match unsafe {
            device
                .raw_device()
                .create_framebuffer(&frame_buffer_create_info, None)
        } {
            Ok(val) => val,
            Err(err) => return Err(VulkanFramebufferError::CreationError(err)),
        };

        Ok(VulkanFramebuffer {
            id: FRAMEBUFFER_ID_COUNTER.next(),
            device_id: device.id(),
            renderpass_id: renderpass.id(),
            attached_images_ids,
            dimensions,
            raw_framebuffer,
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
                .raw_device()
                .destroy_framebuffer(*self.raw_framebuffer(), None);
        }
        Ok(())
    }

    pub fn raw_framebuffer(&self) -> &ash::vk::Framebuffer {
        &self.raw_framebuffer
    }

    pub fn dimensions(&self) -> Vec2u { self.dimensions }

    pub fn id(&self) -> u32 { self.id }

    pub fn renderpass_id(&self) -> u32 { self.renderpass_id }

    pub fn attached_images_id(&self) -> &[u32] { &self.attached_images_ids }
}
