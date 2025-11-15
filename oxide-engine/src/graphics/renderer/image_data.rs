use std::sync::Arc;

use crate::{
    graphics::vulkan::{
        VulkanError,
        framebuffer::Framebuffer,
        image::{ImageAspectFlags, ImageFormat, ImageInfo, ImageUsage, ImageView, OwnedImage},
        renderpass::Renderpass,
        swapchain::Swapchain,
    },
    resources::ResourceCollection,
};

#[allow(unused)]
pub struct SwapchainViews(Vec<Arc<ImageView>>);
#[allow(unused)]
pub struct DepthImage(Arc<OwnedImage>);
#[allow(unused)]
pub struct DepthView(Arc<ImageView>);

pub fn init_renderer_images(resources: &mut ResourceCollection) -> Result<(), VulkanError> {
    let renderpass = resources.get::<Arc<Renderpass>>().unwrap();

    let swapchain = Swapchain::new(renderpass.surface.clone(), renderpass.device.clone(), None)?;

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

    resources.add(swapchain.clone()).unwrap();
    resources.add(SwapchainViews(swapchain_views)).unwrap();
    resources.add(DepthImage(depth_image)).unwrap();
    resources.add(DepthView(depth_view)).unwrap();
    resources.add(framebuffers).unwrap();

    Ok(())
}

pub fn recreate_renderer_images(resources: &mut ResourceCollection) -> Result<(), VulkanError> {
    let renderpass = resources.get::<Arc<Renderpass>>().unwrap();
    let old_swapchain = resources.get::<Arc<Swapchain>>().unwrap();

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

    resources.add_or_change(swapchain.clone()).unwrap();
    resources
        .add_or_change(SwapchainViews(swapchain_views))
        .unwrap();
    resources.add_or_change(DepthImage(depth_image)).unwrap();
    resources.add_or_change(DepthView(depth_view)).unwrap();
    resources.add_or_change(framebuffers).unwrap();

    Ok(())
}
