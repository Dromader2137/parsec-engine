use crate::{
    graphics::vulkan::{device::VulkanDevice, surface::VulkanSurface},
    utils::id_counter::IdCounter,
};

#[derive(Debug)]
pub struct VulkanRenderpass {
    id: u32,
    device_id: u32,
    renderpass: ash::vk::RenderPass,
}

#[derive(Debug, thiserror::Error)]
pub enum VulkanRenderpassError {
    #[error("Failed to create a renderpass: {0}")]
    CreationError(ash::vk::Result),
    #[error(
        "Device used for creating a renderpass was created for a different \
         surface"
    )]
    SurfaceMismatch,
    #[error("Renderpass created on a different device")]
    DeviceMismatch,
}

static ID_COUNTER: once_cell::sync::Lazy<IdCounter> =
    once_cell::sync::Lazy::new(|| IdCounter::new(0));
impl VulkanRenderpass {
    pub fn new(
        surface: &VulkanSurface,
        device: &VulkanDevice,
    ) -> Result<VulkanRenderpass, VulkanRenderpassError> {
        if device.surface_id() != surface.id() {
            return Err(VulkanRenderpassError::SurfaceMismatch);
        }

        let renderpass_attachments = [
            ash::vk::AttachmentDescription {
                format: surface.format(),
                samples: ash::vk::SampleCountFlags::TYPE_1,
                load_op: ash::vk::AttachmentLoadOp::CLEAR,
                store_op: ash::vk::AttachmentStoreOp::STORE,
                final_layout: ash::vk::ImageLayout::PRESENT_SRC_KHR,
                ..Default::default()
            },
            ash::vk::AttachmentDescription {
                format: ash::vk::Format::D32_SFLOAT,
                samples: ash::vk::SampleCountFlags::TYPE_1,
                load_op: ash::vk::AttachmentLoadOp::CLEAR,
                store_op: ash::vk::AttachmentStoreOp::DONT_CARE,
                final_layout:
                    ash::vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                ..Default::default()
            },
        ];
        let color_attachment_refs = [ash::vk::AttachmentReference {
            attachment: 0,
            layout: ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }];
        let depth_attachment_refs = ash::vk::AttachmentReference {
            attachment: 1,
            layout: ash::vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };
        let dependencies = [ash::vk::SubpassDependency {
            src_subpass: ash::vk::SUBPASS_EXTERNAL,
            src_stage_mask:
                ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_access_mask: ash::vk::AccessFlags::COLOR_ATTACHMENT_READ
                | ash::vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dst_stage_mask:
                ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            ..Default::default()
        }];

        let subpass = ash::vk::SubpassDescription::default()
            .color_attachments(&color_attachment_refs)
            .depth_stencil_attachment(&depth_attachment_refs)
            .pipeline_bind_point(ash::vk::PipelineBindPoint::GRAPHICS);

        let renderpass_create_info = ash::vk::RenderPassCreateInfo::default()
            .attachments(&renderpass_attachments)
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(&dependencies);

        let renderpass = match unsafe {
            device
                .get_device_raw()
                .create_render_pass(&renderpass_create_info, None)
        } {
            Ok(val) => val,
            Err(err) => return Err(VulkanRenderpassError::CreationError(err)),
        };

        Ok(VulkanRenderpass {
            id: ID_COUNTER.next(),
            device_id: device.id(),
            renderpass,
        })
    }

    pub fn delete_renderpass(
        self,
        device: &VulkanDevice,
    ) -> Result<(), VulkanRenderpassError> {
        if self.device_id != device.id() {
            return Err(VulkanRenderpassError::DeviceMismatch);
        }

        unsafe {
            device
                .get_device_raw()
                .destroy_render_pass(*self.get_renderpass_raw(), None);
        }

        Ok(())
    }

    pub fn get_renderpass_raw(&self) -> &ash::vk::RenderPass {
        &self.renderpass
    }

    pub fn device_id(&self) -> u32 { self.device_id }

    pub fn id(&self) -> u32 { self.id }
}
