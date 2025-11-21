use std::sync::Arc;

use crate::{
    graphics::vulkan::{
        VulkanError,
        framebuffer::Framebuffer,
        image::{ImageAspectFlags, ImageFormat, ImageInfo, ImageUsage, ImageView, OwnedImage},
        renderpass::Renderpass,
        swapchain::Swapchain,
    },
    resources::Rsc,
};

#[allow(unused)]
pub struct SwapchainViews(Vec<Arc<ImageView>>);
#[allow(unused)]
pub struct DepthImage(Arc<OwnedImage>);
#[allow(unused)]
pub struct DepthView(Arc<ImageView>);

pub fn init_renderer_images() -> Result<(), VulkanError> {
    let renderpass = Rsc::<Arc<Renderpass>>::get().unwrap();

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

    Rsc::add(swapchain.clone()).unwrap();
    Rsc::add(SwapchainViews(swapchain_views)).unwrap();
    Rsc::add(DepthImage(depth_image)).unwrap();
    Rsc::add(DepthView(depth_view)).unwrap();
    Rsc::add(framebuffers).unwrap();

    Ok(())
}

pub fn recreate_renderer_images() -> Result<(), VulkanError> {
    let renderpass = Rsc::<Arc<Renderpass>>::get().unwrap();
    let old_swapchain = Rsc::<Arc<Swapchain>>::get().unwrap();

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

    Rsc::add_overwrite(swapchain.clone()).unwrap();
    Rsc::add_overwrite(SwapchainViews(swapchain_views)).unwrap();
    Rsc::add_overwrite(DepthImage(depth_image)).unwrap();
    Rsc::add_overwrite(DepthView(depth_view)).unwrap();
    Rsc::add_overwrite(framebuffers).unwrap();

    Ok(())
}
