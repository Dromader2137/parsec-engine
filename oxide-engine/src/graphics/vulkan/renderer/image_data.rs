use std::sync::Arc;

use crate::graphics::{
    vulkan::{
        VulkanError,
        context::VulkanContext,
        framebuffer::Framebuffer,
        image::{
            ImageAspectFlags, ImageFormat, ImageInfo, ImageUsage, ImageView, OwnedImage,
        },
        renderpass::Renderpass,
        swapchain::Swapchain,
    },
    window::WindowWrapper,
};

pub struct VulkanRendererImageData {
    pub depth_image: Arc<OwnedImage>,
    pub depth_view: Arc<ImageView>,
    pub swapchain_views: Vec<Arc<ImageView>>,
    pub swapchain: Arc<Swapchain>,
    pub framebuffers: Vec<Arc<Framebuffer>>,
}

impl VulkanRendererImageData {
    pub fn new(
        context: Arc<VulkanContext>,
        renderpass: Arc<Renderpass>,
        window: Arc<WindowWrapper>,
    ) -> Result<VulkanRendererImageData, VulkanError> {
        let swapchain = Swapchain::new(
            context.surface.clone(),
            context.device.clone(),
            window.clone(),
            None
        )?;

        let swapchain_images = &swapchain.swapchain_images;
        let swapchain_format = context.surface.format().into();
        let swapchain_views = {
            let mut out = Vec::new();
            for image in swapchain_images.iter() {
                let view = ImageView::from_image(
                    context.device.clone(),
                    image.clone(),
                    swapchain_format,
                    ImageAspectFlags::COLOR,
                )?;
                out.push(view);
            }
            out
        };

        let depth_image = OwnedImage::new(
            context.device.clone(),
            ImageInfo {
                format: ImageFormat::D16_UNORM,
                size: (window.get_width(), window.get_height()),
                usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
            },
        )?;
        let depth_view = ImageView::from_image(
            context.device.clone(),
            depth_image.clone(),
            ImageFormat::D16_UNORM,
            ImageAspectFlags::DEPTH,
        )?;

        let framebuffers = {
            let mut out = Vec::new();
            for image_view in swapchain_views.iter() {
                out.push(Framebuffer::new(
                    image_view.clone(),
                    depth_view.clone(),
                    renderpass.clone(),
                    window.clone(),
                )?);
            }
            out
        };

        Ok(VulkanRendererImageData {
            depth_image,
            depth_view,
            swapchain_views,
            swapchain,
            framebuffers,
        })
    }

    pub fn recreate(
        &mut self,
        context: Arc<VulkanContext>,
        renderpass: Arc<Renderpass>,
        window: Arc<WindowWrapper>,
    ) -> Result<(), VulkanError> {
        let swapchain = Swapchain::new(
            context.surface.clone(),
            context.device.clone(),
            window.clone(),
            Some(self.swapchain.clone())
        )?;

        let swapchain_images = &swapchain.swapchain_images;
        let swapchain_format = context.surface.format().into();
        let swapchain_views = {
            let mut out = Vec::new();
            for image in swapchain_images.iter() {
                let view = ImageView::from_image(
                    context.device.clone(),
                    image.clone(),
                    swapchain_format,
                    ImageAspectFlags::COLOR,
                )?;
                out.push(view);
            }
            out
        };

        let depth_image = OwnedImage::new(
            context.device.clone(),
            ImageInfo {
                format: ImageFormat::D16_UNORM,
                size: (window.get_width(), window.get_height()),
                usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
            },
        )?;
        let depth_view = ImageView::from_image(
            context.device.clone(),
            depth_image.clone(),
            ImageFormat::D16_UNORM,
            ImageAspectFlags::DEPTH,
        )?;

        let framebuffers = {
            let mut out = Vec::new();
            for image_view in swapchain_views.iter() {
                out.push(Framebuffer::new(
                    image_view.clone(),
                    depth_view.clone(),
                    renderpass.clone(),
                    window.clone(),
                )?);
            }
            out
        };

        self.swapchain = swapchain;
        self.swapchain_views = swapchain_views;
        self.depth_image = depth_image;
        self.depth_view = depth_view;
        self.framebuffers = framebuffers;

        Ok(())
    }

    pub fn clamp_frames_in_flight(&self, fif: usize) -> usize {
        fif.min(self.swapchain.swapchain_images.len()).max(1)
    }
}
