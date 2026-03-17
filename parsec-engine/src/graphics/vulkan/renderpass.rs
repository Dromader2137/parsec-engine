use crate::{
    graphics::{
        renderpass::{
            RenderpassAttachment, RenderpassAttachmentLoadOp,
            RenderpassAttachmentStoreOp, RenderpassAttachmentType,
            RenderpassClearValue,
        },
        vulkan::{device::VulkanDevice, image::VulkanImageFormat},
    },
    math::vec::Vec4f,
};

pub struct VulkanRenderpass {
    id: u32,
    attachments: Vec<VulkanRenderpassAttachment>,
    clear_values: Vec<VulkanClearValue>,
    renderpass: ash::vk::RenderPass,
}

#[derive(Debug, thiserror::Error)]
pub enum VulkanRenderpassError {
    #[error("Failed to create a renderpass: {0}")]
    CreationError(ash::vk::Result),
    #[error("Only one depth attachment is allowed")]
    OnlyOneDepthAttachmentAllowed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VulkanClearValue {
    Color(Vec4f),
    Depth(f32),
}

impl VulkanClearValue {
    pub fn new(value: RenderpassClearValue) -> Self {
        match value {
            RenderpassClearValue::Color(r, g, b, a) => {
                Self::Color(Vec4f::new(r, g, b, a))
            },
            RenderpassClearValue::Depth(d) => Self::Depth(d),
        }
    }

    pub fn raw_clear_value(&self) -> ash::vk::ClearValue {
        match self {
            VulkanClearValue::Color(col) => ash::vk::ClearValue {
                color: ash::vk::ClearColorValue {
                    float32: [col.x, col.y, col.z, col.w],
                },
            },
            VulkanClearValue::Depth(d) => ash::vk::ClearValue {
                depth_stencil: ash::vk::ClearDepthStencilValue {
                    depth: *d,
                    stencil: 0,
                },
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulkanStoreOp {
    Store,
    DontCare,
}

impl VulkanStoreOp {
    pub fn new(value: RenderpassAttachmentStoreOp) -> Self {
        match value {
            RenderpassAttachmentStoreOp::Store => Self::Store,
            RenderpassAttachmentStoreOp::DontCare => Self::DontCare,
        }
    }

    pub fn raw_store_op(&self) -> ash::vk::AttachmentStoreOp {
        match self {
            VulkanStoreOp::Store => ash::vk::AttachmentStoreOp::STORE,
            VulkanStoreOp::DontCare => ash::vk::AttachmentStoreOp::DONT_CARE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulkanLoadOp {
    Load,
    Clear,
    DontCare,
}

impl VulkanLoadOp {
    pub fn new(value: RenderpassAttachmentLoadOp) -> Self {
        match value {
            RenderpassAttachmentLoadOp::Load => Self::Load,
            RenderpassAttachmentLoadOp::Clear => Self::Clear,
            RenderpassAttachmentLoadOp::DontCare => Self::DontCare,
        }
    }

    pub fn raw_load_op(&self) -> ash::vk::AttachmentLoadOp {
        match self {
            VulkanLoadOp::Load => ash::vk::AttachmentLoadOp::LOAD,
            VulkanLoadOp::Clear => ash::vk::AttachmentLoadOp::CLEAR,
            VulkanLoadOp::DontCare => ash::vk::AttachmentLoadOp::DONT_CARE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulkanRenderpassAttachmentType {
    Color,
    Depth,
}

impl VulkanRenderpassAttachmentType {
    fn new(value: RenderpassAttachmentType) -> Self {
        match value {
            RenderpassAttachmentType::Color => Self::Color,
            RenderpassAttachmentType::Depth => Self::Depth,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VulkanRenderpassAttachment {
    pub attachment_type: VulkanRenderpassAttachmentType,
    pub load_op: VulkanLoadOp,
    pub store_op: VulkanStoreOp,
    pub image_format: VulkanImageFormat,
    // pub final_layout: VulkanImageLayout,
}

impl VulkanRenderpassAttachment {
    pub fn new(value: RenderpassAttachment) -> Self {
        VulkanRenderpassAttachment {
            attachment_type: VulkanRenderpassAttachmentType::new(
                value.attachment_type,
            ),
            load_op: VulkanLoadOp::new(value.load_op),
            store_op: VulkanStoreOp::new(value.store_op),
            image_format: VulkanImageFormat::new(value.image_format),
        }
    }

    fn raw_renderpass_attachment(&self) -> ash::vk::AttachmentDescription {
        ash::vk::AttachmentDescription {
            format: self.image_format.raw_image_format(),
            samples: ash::vk::SampleCountFlags::TYPE_1,
            load_op: self.load_op.raw_load_op(),
            store_op: self.store_op.raw_store_op(),
            final_layout: ash::vk::ImageLayout::PRESENT_SRC_KHR,
            ..Default::default()
        }
    }
}

crate::create_counter! {ID_COUNTER}
impl VulkanRenderpass {
    pub fn new(
        device: &VulkanDevice,
        attachments: &[(VulkanRenderpassAttachment, VulkanClearValue)],
    ) -> Result<VulkanRenderpass, VulkanRenderpassError> {
        let mut color_attachment_refs = Vec::new();
        let mut depth_attachment_refs = Vec::new();
        let mut renderpass_attachments = Vec::new();
        let mut clear_values = Vec::new();
        for (id, (attachment, clear_value)) in attachments.iter().enumerate() {
            match attachment.attachment_type {
                VulkanRenderpassAttachmentType::Color => {
                    color_attachment_refs.push(ash::vk::AttachmentReference {
                        attachment: id as u32,
                        layout: ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                    });
                },
                VulkanRenderpassAttachmentType::Depth => {
                    depth_attachment_refs.push(
                        ash::vk::AttachmentReference {
                            attachment: id as u32,
                            layout: ash::vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                        }
                    );
                },
            };
            renderpass_attachments.push(attachment.raw_renderpass_attachment());
            clear_values.push(*clear_value);
        }

        if depth_attachment_refs.len() > 1 {
            return Err(VulkanRenderpassError::OnlyOneDepthAttachmentAllowed);
        }
        let depth_attachment_ref = depth_attachment_refs.get(0);

        let mut subpass = ash::vk::SubpassDescription::default()
            .color_attachments(&color_attachment_refs)
            .pipeline_bind_point(ash::vk::PipelineBindPoint::GRAPHICS);

        if let Some(depth) = depth_attachment_ref {
            subpass = subpass.depth_stencil_attachment(depth);
        }

        let renderpass_create_info = ash::vk::RenderPassCreateInfo::default()
            .attachments(&renderpass_attachments)
            .subpasses(std::slice::from_ref(&subpass));

        let renderpass = match unsafe {
            device
                .raw_handle()
                .create_render_pass(&renderpass_create_info, None)
        } {
            Ok(val) => val,
            Err(err) => return Err(VulkanRenderpassError::CreationError(err)),
        };

        Ok(VulkanRenderpass {
            id: ID_COUNTER.next(),
            attachments: attachments.iter().map(|x| x.0.clone()).collect(),
            clear_values,
            renderpass,
        })
    }

    pub fn destroy(self, device: &VulkanDevice) {
        unsafe {
            device
                .raw_handle()
                .destroy_render_pass(self.renderpass, None)
        }
    }

    pub fn raw_handle(&self) -> &ash::vk::RenderPass { &self.renderpass }

    pub fn id(&self) -> u32 { self.id }

    pub fn color_attachment_count(&self) -> u32 {
        self.attachments
            .iter()
            .filter(|x| {
                x.attachment_type == VulkanRenderpassAttachmentType::Color
            })
            .count() as u32
    }

    pub fn has_depth_attachment(&self) -> bool {
        self.attachments
            .iter()
            .find(|x| {
                x.attachment_type == VulkanRenderpassAttachmentType::Depth
            })
            .is_some()
    }

    pub fn clear_values(&self) -> &[VulkanClearValue] { &self.clear_values }

    pub fn depth_attachment_id(&self) -> Option<u32> {
        self.attachments
            .iter()
            .enumerate()
            .find(|(_, x)| {
                x.attachment_type == VulkanRenderpassAttachmentType::Depth
            })
            .map(|x| x.0 as u32)
    }

    pub fn attachments(&self) -> &[VulkanRenderpassAttachment] {
        &self.attachments
    }
}
