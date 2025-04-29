use super::{context::VulkanError, device::Device, surface::Surface};

pub struct Renderpass {
    renderpass: ash::vk::RenderPass
}

#[derive(Debug)]
pub enum RenderpassError {
    CreationError(ash::vk::Result)
}

impl From<RenderpassError> for VulkanError {
    fn from(value: RenderpassError) -> Self {
        VulkanError::RenderpassError(value)
    }
}

impl Renderpass {
    pub fn new(surface: &Surface, device: &Device) -> Result<Renderpass, RenderpassError> {
        let renderpass_attachments = [
            ash::vk::AttachmentDescription {
                format: surface.format(),
                samples: ash::vk::SampleCountFlags::TYPE_1,
                load_op: ash::vk::AttachmentLoadOp::CLEAR,
                store_op: ash::vk::AttachmentStoreOp::STORE,
                final_layout: ash::vk::ImageLayout::PRESENT_SRC_KHR,
                ..Default::default()
            },
        ];
        let color_attachment_refs = [ash::vk::AttachmentReference {
            attachment: 0,
            layout: ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }];
        let dependencies = [ash::vk::SubpassDependency {
            src_subpass: ash::vk::SUBPASS_EXTERNAL,
            src_stage_mask: ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_access_mask: ash::vk::AccessFlags::COLOR_ATTACHMENT_READ | ash::vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dst_stage_mask: ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            ..Default::default()
        }];

        let subpass = ash::vk::SubpassDescription::default()
            .color_attachments(&color_attachment_refs)
            .pipeline_bind_point(ash::vk::PipelineBindPoint::GRAPHICS);

        let renderpass_create_info = ash::vk::RenderPassCreateInfo::default()
            .attachments(&renderpass_attachments)
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(&dependencies);

        let renderpass = match unsafe { device.get_device_raw().create_render_pass(&renderpass_create_info, None) } {
            Ok(val) => val,
            Err(err) => return Err(RenderpassError::CreationError(err))
        };

        Ok( Renderpass { renderpass } )
    }

    pub fn get_renderpass_raw(&self) -> &ash::vk::RenderPass {
        &self.renderpass
    }
}
