use crate::graphics::{
    renderpass::{
        RenderpassAttachment, RenderpassAttachmentType, RenderpassClearValue,
    },
    vulkan::{
        device::VulkanDevice,
        image::{VulkanImageFormat, VulkanImageLayout},
        surface::VulkanSurface,
    },
};

pub struct VulkanRenderpass {
    id: u32,
    device_id: u32,
    color_attachment_count: u32,
    has_depth_attachment: bool,
    depth_attachment_id: u32,
    attachments: Vec<VulkanRenderpassAttachment>, 
    clear_values: Vec<VulkanClearValue>,
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
    #[error("Only one depth attachment is allowed")]
    OnlyOneDepthAttachmentAllowed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulkanRenderpassAttachmentType {
    Color,
    Depth,
}

pub type VulkanLoadOperation = ash::vk::AttachmentLoadOp;
pub type VulkanStoreOperation = ash::vk::AttachmentStoreOp;

#[derive(Debug, Clone)]
pub struct VulkanRenderpassAttachment {
    pub attachment_type: VulkanRenderpassAttachmentType,
    pub load_op: VulkanLoadOperation,
    pub store_op: VulkanStoreOperation,
    pub image_initial_layout: VulkanImageLayout,
    pub image_layout: VulkanImageLayout,
    pub image_format: VulkanImageFormat,
}

impl From<VulkanRenderpassAttachment> for ash::vk::AttachmentDescription {
    fn from(value: VulkanRenderpassAttachment) -> Self {
        ash::vk::AttachmentDescription {
            format: value.image_format,
            samples: ash::vk::SampleCountFlags::TYPE_1,
            load_op: value.load_op,
            store_op: value.store_op,
            // initial_layout: value.image_initial_layout,
            final_layout: value.image_layout,
            ..Default::default()
        }
    }
}

impl From<RenderpassAttachmentType> for VulkanRenderpassAttachmentType {
    fn from(value: RenderpassAttachmentType) -> Self {
        match value {
            RenderpassAttachmentType::PresentColor => {
                VulkanRenderpassAttachmentType::Color
            },
            RenderpassAttachmentType::PresentDepth => {
                VulkanRenderpassAttachmentType::Depth
            },
            RenderpassAttachmentType::StoreDepth => {
                VulkanRenderpassAttachmentType::Depth
            },
        }
    }
}

impl From<RenderpassAttachmentType> for VulkanLoadOperation {
    fn from(value: RenderpassAttachmentType) -> Self {
        match value {
            RenderpassAttachmentType::PresentColor => {
                VulkanLoadOperation::CLEAR
            },
            RenderpassAttachmentType::PresentDepth => {
                VulkanLoadOperation::CLEAR
            },
            RenderpassAttachmentType::StoreDepth => VulkanLoadOperation::CLEAR,
        }
    }
}

impl From<RenderpassAttachmentType> for VulkanStoreOperation {
    fn from(value: RenderpassAttachmentType) -> Self {
        match value {
            RenderpassAttachmentType::PresentColor => {
                VulkanStoreOperation::STORE
            },
            RenderpassAttachmentType::PresentDepth => {
                VulkanStoreOperation::DONT_CARE
            },
            RenderpassAttachmentType::StoreDepth => VulkanStoreOperation::STORE,
        }
    }
}

impl From<RenderpassClearValue> for VulkanClearValue {
    fn from(value: RenderpassClearValue) -> Self {
        match value {
            RenderpassClearValue::Color(r, g, b, a) => VulkanClearValue {
                color: VulkanClearColorValue {
                    float32: [r, g, b, a],
                },
            },
            RenderpassClearValue::Depth(d) => VulkanClearValue {
                depth_stencil: VulkanClearDepthValue {
                    depth: d,
                    stencil: 0,
                },
            },
        }
    }
}

fn get_image_layout(value: &RenderpassAttachmentType) -> VulkanImageLayout {
    match value {
        RenderpassAttachmentType::PresentColor => {
            VulkanImageLayout::PRESENT_SRC_KHR
        },
        RenderpassAttachmentType::PresentDepth => {
            VulkanImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
        },
        RenderpassAttachmentType::StoreDepth => {
            VulkanImageLayout::SHADER_READ_ONLY_OPTIMAL
        },
    }
}

fn get_image_initial_layout(value: &RenderpassAttachmentType) -> VulkanImageLayout {
    match value {
        RenderpassAttachmentType::PresentColor => {
            VulkanImageLayout::PRESENT_SRC_KHR
        },
        RenderpassAttachmentType::PresentDepth => {
            VulkanImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
        },
        RenderpassAttachmentType::StoreDepth => {
            VulkanImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
        },
    }
}

impl From<RenderpassAttachment>
    for (VulkanRenderpassAttachment, VulkanClearValue)
{
    fn from(value: RenderpassAttachment) -> Self {
        (
            VulkanRenderpassAttachment {
                attachment_type: value.attachment_type.into(),
                load_op: value.attachment_type.into(),
                store_op: value.attachment_type.into(),
                image_layout: get_image_layout(&value.attachment_type),
                image_initial_layout: get_image_initial_layout(&value.attachment_type),
                image_format: value.image_format.into(),
            },
            value.clear_value.into(),
        )
    }
}

pub type VulkanClearValue = ash::vk::ClearValue;
pub type VulkanClearColorValue = ash::vk::ClearColorValue;
pub type VulkanClearDepthValue = ash::vk::ClearDepthStencilValue;

crate::create_counter! {ID_COUNTER}
impl VulkanRenderpass {
    pub fn new(
        surface: &VulkanSurface,
        device: &VulkanDevice,
        attachments: &[(VulkanRenderpassAttachment, VulkanClearValue)],
    ) -> Result<VulkanRenderpass, VulkanRenderpassError> {
        if device.surface_id() != surface.id() {
            return Err(VulkanRenderpassError::SurfaceMismatch);
        }

        let mut color_attachment_refs = Vec::new();
        let mut depth_attachment_refs = Vec::new();
        let mut renderpass_attachments = Vec::new();
        let mut depth_id = 0;
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
                    depth_id = id;
                    depth_attachment_refs.push(
                        ash::vk::AttachmentReference {
                            attachment: id as u32,
                            layout: ash::vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                        }
                    );
                },
            };
            renderpass_attachments.push(attachment.clone().into());
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
                .get_device_raw()
                .create_render_pass(&renderpass_create_info, None)
        } {
            Ok(val) => val,
            Err(err) => return Err(VulkanRenderpassError::CreationError(err)),
        };

        Ok(VulkanRenderpass {
            id: ID_COUNTER.next(),
            device_id: device.id(),
            color_attachment_count: color_attachment_refs.len() as u32,
            has_depth_attachment: depth_attachment_ref.is_some(),
            depth_attachment_id: depth_id as u32,
            attachments: attachments.iter().map(|x| x.0.clone()).collect(),
            clear_values,
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

    pub fn color_attachment_count(&self) -> u32 { self.color_attachment_count }

    pub fn has_depth_attachment(&self) -> bool { self.has_depth_attachment }

    pub fn clear_values(&self) -> &[VulkanClearValue] { &self.attachments.iter().map(f) }

    pub fn depth_attachment_id(&self) -> u32 { self.depth_attachment_id }

    pub fn attachments(&self) -> &[VulkanRenderpassAttachment] {
        &self.attachments
    }
}
