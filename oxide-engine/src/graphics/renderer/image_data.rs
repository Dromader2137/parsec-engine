use std::sync::Arc;

use crate::{
    graphics::vulkan::{
        VulkanError,
        framebuffer::Framebuffer,
        image::{
            ImageAspectFlags, ImageFormat, ImageInfo, ImageUsage, ImageView,
            OwnedImage,
        },
        renderpass::Renderpass,
        swapchain::Swapchain,
    },
    resources::Resources,
};

#[allow(unused)]
pub struct SwapchainViews(Vec<Arc<ImageView>>);
#[allow(unused)]
pub struct DepthImage(Arc<OwnedImage>);
#[allow(unused)]
pub struct DepthView(Arc<ImageView>);

pub fn init_renderer_images(
    renderpass: Arc<Renderpass>,
) -> Result<Arc<Swapchain>, VulkanError> {
    let swapchain = Swapchain::new(
        renderpass.surface.clone(),
        renderpass.device.clone(),
        None,
    )?;

    let swapchain_images = &swapchain.swapchain_images;
    let swapchain_format = renderpass.surface.format().into();
    let swapchain_views = {
        let mut out = Vec::new();
        for image in swapchain_images.iter() {
            let view = ImageView::from_image(
                renderpass.device.clone(),
                image.clone(),
                swapchain_format,
                ImageAspectFlags::COLOR,
            )?;
            out.push(view);
        }
        out
    };

    let depth_image = OwnedImage::new(renderpass.device.clone(), ImageInfo {
        format: ImageFormat::D16_UNORM,
        size: (
            renderpass.surface.window.width(),
            renderpass.surface.window.height(),
        ),
        usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
    })?;
    let depth_view = ImageView::from_image(
        renderpass.device.clone(),
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
            )?);
        }
        out
    };

    drop(renderpass);

    Resources::add(swapchain.clone()).unwrap();
    Resources::add(SwapchainViews(swapchain_views)).unwrap();
    Resources::add(DepthImage(depth_image)).unwrap();
    Resources::add(DepthView(depth_view)).unwrap();
    Resources::add(framebuffers).unwrap();

    Ok(swapchain)
}

pub fn recreate_renderer_images(
    renderpass: Arc<Renderpass>,
    old_swapchain: Arc<Swapchain>,
) -> Result<(), VulkanError> {
    let swapchain = Swapchain::new(
        renderpass.surface.clone(),
        renderpass.device.clone(),
        Some(old_swapchain.clone()),
    )?;

    drop(old_swapchain);

    let swapchain_images = &swapchain.swapchain_images;
    let swapchain_format = renderpass.surface.format().into();
    let swapchain_views = {
        let mut out = Vec::new();
        for image in swapchain_images.iter() {
            let view = ImageView::from_image(
                renderpass.device.clone(),
                image.clone(),
                swapchain_format,
                ImageAspectFlags::COLOR,
            )?;
            out.push(view);
        }
        out
    };

    let depth_image = OwnedImage::new(renderpass.device.clone(), ImageInfo {
        format: ImageFormat::D16_UNORM,
        size: (
            renderpass.surface.window.width(),
            renderpass.surface.window.height(),
        ),
        usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
    })?;
    let depth_view = ImageView::from_image(
        renderpass.device.clone(),
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
            )?);
        }
        out
    };

    drop(renderpass);

    Resources::add_or_change(swapchain.clone());
    Resources::add_or_change(SwapchainViews(swapchain_views));
    Resources::add_or_change(DepthImage(depth_image));
    Resources::add_or_change(DepthView(depth_view));
    Resources::add_or_change(framebuffers);

    Ok(())
}
