use crate::{
    graphics::{
        vulkan::{
            VulkanError,
            device::Device,
            framebuffer::Framebuffer,
            image::{
                ImageAspectFlags, ImageFormat, ImageInfo, ImageUsage,
                ImageView, OwnedImage, SwapchainImage,
            },
            instance::Instance,
            physical_device::PhysicalDevice,
            renderpass::Renderpass,
            surface::Surface,
            swapchain::Swapchain,
        },
        window::WindowWrapper,
    },
    resources::Resources,
};

#[allow(unused)]
pub struct SwapchainViews(Vec<ImageView>);
#[allow(unused)]
pub struct DepthImage(OwnedImage);
#[allow(unused)]
pub struct DepthView(ImageView);

pub fn init_renderer_images(
    instance: &Instance,
    physical_device: &PhysicalDevice,
    window: &WindowWrapper,
    surface: &Surface,
    device: &Device,
    renderpass: &Renderpass,
) -> Result<(Swapchain, Vec<SwapchainImage>), VulkanError> {
    let (swapchain, swapchain_images) = Swapchain::new(
        instance,
        physical_device,
        window,
        surface,
        device,
        None,
    )?;

    let swapchain_format = surface.format().into();
    let swapchain_views = {
        let mut out = Vec::new();
        for image in swapchain_images.iter() {
            let view = ImageView::from_image(
                device,
                image,
                swapchain_format,
                ImageAspectFlags::COLOR,
            )?;
            out.push(view);
        }
        out
    };

    let depth_image = OwnedImage::new(physical_device, device, ImageInfo {
        format: ImageFormat::D16_UNORM,
        size: (window.width(), window.height()),
        usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
    })?;
    let depth_view = ImageView::from_image(
        device,
        &depth_image,
        ImageFormat::D16_UNORM,
        ImageAspectFlags::DEPTH,
    )?;

    let framebuffers = {
        let mut out = Vec::new();
        for image_view in swapchain_views.iter() {
            out.push(Framebuffer::new(
                window,
                device,
                image_view,
                &depth_view,
                renderpass,
            )?);
        }
        out
    };

    Resources::add(SwapchainViews(swapchain_views)).unwrap();
    Resources::add(DepthImage(depth_image)).unwrap();
    Resources::add(DepthView(depth_view)).unwrap();
    Resources::add(framebuffers).unwrap();

    Ok((swapchain, swapchain_images))
}

pub fn recreate_renderer_images(
    instance: &Instance,
    physical_device: &PhysicalDevice,
    window: &WindowWrapper,
    surface: &Surface,
    device: &Device,
    renderpass: &Renderpass,
    old_swapchain: &Swapchain,
) -> Result<(Swapchain, Vec<SwapchainImage>), VulkanError> {
    let (swapchain, swapchain_images) = Swapchain::new(
        instance,
        physical_device,
        window,
        surface,
        device,
        Some(old_swapchain),
    )?;

    let swapchain_format = surface.format().into();
    let swapchain_views = {
        let mut out = Vec::new();
        for image in swapchain_images.iter() {
            let view = ImageView::from_image(
                device,
                image,
                swapchain_format,
                ImageAspectFlags::COLOR,
            )?;
            out.push(view);
        }
        out
    };

    let depth_image = OwnedImage::new(physical_device, device, ImageInfo {
        format: ImageFormat::D32_SFLOAT,
        size: (window.width(), window.height()),
        usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
    })?;
    let depth_view = ImageView::from_image(
        device,
        &depth_image,
        ImageFormat::D16_UNORM,
        ImageAspectFlags::DEPTH,
    )?;

    let framebuffers = {
        let mut out = Vec::new();
        for image_view in swapchain_views.iter() {
            out.push(Framebuffer::new(
                window,
                device,
                image_view,
                &depth_view,
                renderpass,
            )?);
        }
        out
    };

    Resources::add_or_change(SwapchainViews(swapchain_views));
    Resources::add_or_change(DepthImage(depth_image));
    Resources::add_or_change(DepthView(depth_view));
    Resources::add_or_change(framebuffers);

    Ok((swapchain, swapchain_images))
}
