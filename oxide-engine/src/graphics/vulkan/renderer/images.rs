use crate::graphics::{
    vulkan::{
        VulkanError,
        context::VulkanContext,
        device::Device,
        framebuffer::Framebuffer,
        image::{
            Image, ImageAspectFlags, ImageFormat, ImageInfo, ImageUsage, ImageView, OwnedImage,
        },
        renderpass::Renderpass,
        swapchain::Swapchain,
    },
    window::WindowWrapper,
};

pub struct VulkanRendererFrameData {
    pub depth_image: OwnedImage,
    pub depth_view: ImageView,
    pub swapchain_images: Vec<Image>,
    pub swapchain_views: Vec<ImageView>,
    pub swapchain: Swapchain,
    pub framebuffers: Vec<Framebuffer>,
}

impl VulkanRendererFrameData {
    pub fn new(
        context: &VulkanContext,
        window: &WindowWrapper,
        renderpass: &Renderpass,
    ) -> Result<VulkanRendererFrameData, VulkanError> {
        let swapchain = Swapchain::new(
            &context.instance,
            &context.surface,
            &context.physical_device,
            &context.device,
            window,
        )?;

        let swapchain_images = swapchain.get_images()?;
        let swapchain_format = context.surface.format().into();
        let swapchain_views = {
            let mut out = Vec::new();
            for image in swapchain_images.iter() {
                let view = ImageView::from_image(
                    &context.device,
                    image,
                    swapchain_format,
                    ImageAspectFlags::COLOR,
                )?;
                out.push(view);
            }
            out
        };

        let depth_image = OwnedImage::new(
            &context.instance,
            &context.physical_device,
            &context.device,
            ImageInfo {
                format: ImageFormat::D16_UNORM,
                size: (window.get_width(), window.get_height()),
                usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
            },
        )?;
        let depth_view = ImageView::from_image(
            &context.device,
            &depth_image,
            ImageFormat::D16_UNORM,
            ImageAspectFlags::DEPTH,
        )?;

        let framebuffers = {
            let mut out = Vec::new();
            for image_view in swapchain_views.iter() {
                out.push(Framebuffer::new(
                    &context.surface,
                    &context.device,
                    image_view,
                    &depth_view,
                    renderpass,
                    window,
                )?);
            }
            out
        };

        Ok(VulkanRendererFrameData {
            depth_image,
            depth_view,
            swapchain_images,
            swapchain_views,
            swapchain,
            framebuffers,
        })
    }

    pub fn recreate(
        &mut self,
        context: &VulkanContext,
        window: &WindowWrapper,
        renderpass: &Renderpass,
    ) -> Result<(), VulkanError> {
        self.framebuffers
            .iter()
            .for_each(|x| x.cleanup(&context.device));
        self.swapchain_views
            .iter()
            .for_each(|x| x.cleanup(&context.device));
        self.depth_view.cleanup(&context.device);
        self.depth_image.cleanup(&context.device);
        self.swapchain.cleanup();

        let depth_image = OwnedImage::new(
            &context.instance,
            &context.physical_device,
            &context.device,
            ImageInfo {
                format: ImageFormat::D16_UNORM,
                size: (window.get_width(), window.get_height()),
                usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
            },
        )?;
        let depth_view = ImageView::from_image(
            &context.device,
            &depth_image,
            ImageFormat::D16_UNORM,
            ImageAspectFlags::DEPTH,
        )?;
        let swapchain = Swapchain::new(
            &context.instance,
            &context.surface,
            &context.physical_device,
            &context.device,
            window,
        )?;
        let swapchain_images = swapchain.get_images()?;
        let swapchain_format = context.surface.format().into();
        let swapchain_image_views = {
            let mut out = Vec::new();
            for img in swapchain_images.iter() {
                let view = ImageView::from_image(
                    &context.device,
                    img,
                    swapchain_format,
                    ImageAspectFlags::COLOR,
                )?;
                out.push(view);
            }
            out
        };
        let framebuffers = {
            let mut out = Vec::new();
            for image_view in swapchain_image_views.iter() {
                out.push(Framebuffer::new(
                    &context.surface,
                    &context.device,
                    image_view,
                    &depth_view,
                    renderpass,
                    window,
                )?);
            }
            out
        };

        self.swapchain = swapchain;
        self.swapchain_images = swapchain_images;
        self.swapchain_views = swapchain_image_views;
        self.depth_image = depth_image;
        self.depth_view = depth_view;
        self.framebuffers = framebuffers;

        Ok(())
    }

    pub fn cleanup(&self, device: &Device) {
        self.framebuffers.iter().for_each(|x| x.cleanup(device));
        self.swapchain_views.iter().for_each(|x| x.cleanup(device));
        self.swapchain.cleanup();
        self.depth_view.cleanup(device);
        self.depth_image.cleanup(device);
    }

    pub fn clamp_frames_in_flight(&self, fif: usize) -> usize {
        fif.min(self.swapchain_images.len()).max(1)
    }
}
