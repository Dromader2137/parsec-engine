use parsec_engine_graphics::window::Window;

use crate::{
    image::VulkanImageFormat, instance::VulkanInstance,
    physical_device::VulkanPhysicalDevice,
};

pub struct VulkanInitialSurface {
    surface_loader: ash::khr::surface::Instance,
    surface: ash::vk::SurfaceKHR,
}

pub struct VulkanSurface {
    surface: ash::vk::SurfaceKHR,
    surface_loader: ash::khr::surface::Instance,
    surface_format: VulkanImageFormat,
    surface_capabilities: ash::vk::SurfaceCapabilitiesKHR,
}

#[derive(Debug, thiserror::Error)]
pub enum VulkanSurfaceError {
    #[error("Surface not supported by physical device: {0}")]
    SupportError(ash::vk::Result),
    #[error("Failed to create surface: {0}")]
    CreationError(ash::vk::Result),
    #[error("Failed to get diplay handle: {0}")]
    DisplayHandleError(winit::raw_window_handle::HandleError),
    #[error("Failed to get window handle: {0}")]
    WindowHandleError(winit::raw_window_handle::HandleError),
    #[error("Failed to get surface formats: {0}")]
    FormatsError(ash::vk::Result),
    #[error("Failed to get surface capabilities: {0}")]
    CapabilitiesError(ash::vk::Result),
    #[error("No surface format available")]
    NoSurfaceFormatsAvailable,
}

impl VulkanInitialSurface {
    pub fn new(
        instance: &VulkanInstance,
        window: &Window,
    ) -> Result<VulkanInitialSurface, VulkanSurfaceError> {
        let display_handle = window
            .raw_display_handle()
            .map_err(VulkanSurfaceError::DisplayHandleError)?;
        let window_handle = window
            .raw_window_handle()
            .map_err(VulkanSurfaceError::WindowHandleError)?;

        let surface = unsafe {
            ash_window::create_surface(
                instance.raw_entry(),
                instance.raw_handle(),
                display_handle.as_raw(),
                window_handle.as_raw(),
                None,
            )
            .map_err(VulkanSurfaceError::CreationError)?
        };

        let surface_loader = ash::khr::surface::Instance::new(
            instance.raw_entry(),
            instance.raw_handle(),
        );

        Ok(VulkanInitialSurface {
            surface,
            surface_loader,
        })
    }

    pub fn check_surface_support(
        &self,
        physical_device: ash::vk::PhysicalDevice,
        queue_family_index: u32,
    ) -> Result<bool, VulkanSurfaceError> {
        unsafe {
            self.surface_loader
                .get_physical_device_surface_support(
                    physical_device,
                    queue_family_index,
                    self.surface,
                )
                .map_err(VulkanSurfaceError::SupportError)
        }
    }

    pub fn surface_loader_raw(&self) -> &ash::khr::surface::Instance {
        &self.surface_loader
    }

    pub fn surface_raw(&self) -> &ash::vk::SurfaceKHR { &self.surface }
}

impl VulkanSurface {
    pub fn from_initial_surface(
        initial_surface: VulkanInitialSurface,
        physical_device: &VulkanPhysicalDevice,
    ) -> Result<VulkanSurface, VulkanSurfaceError> {
        let VulkanInitialSurface {
            surface_loader,
            surface,
        } = initial_surface;

        let surface_formats = unsafe {
            surface_loader
                .get_physical_device_surface_formats(
                    *physical_device.raw_handle(),
                    surface,
                )
                .map_err(VulkanSurfaceError::FormatsError)?
        };

        if surface_formats.is_empty() {
            return Err(VulkanSurfaceError::NoSurfaceFormatsAvailable);
        }

        let surface_capabilities = unsafe {
            surface_loader
                .get_physical_device_surface_capabilities(
                    *physical_device.raw_handle(),
                    surface,
                )
                .map_err(VulkanSurfaceError::CapabilitiesError)?
        };

        let preferred_formats =
            [VulkanImageFormat::RGBA8SRGB, VulkanImageFormat::BGRA8SRGB];

        let surface_format = *preferred_formats
            .iter()
            .find_map(|preffered_format| {
                let found = surface_formats.iter().find(|format| {
                    format.format == preffered_format.raw_image_format()
                });
                if found.is_some() {
                    Some(preffered_format)
                } else {
                    None
                }
            })
            .ok_or(VulkanSurfaceError::NoSurfaceFormatsAvailable)?;

        Ok(VulkanSurface {
            surface,
            surface_loader,
            surface_format,
            surface_capabilities,
        })
    }

    pub fn destroy(&self) {
        unsafe { self.surface_loader.destroy_surface(self.surface, None) }
    }

    pub fn raw_handle(&self) -> &ash::vk::SurfaceKHR { &self.surface }

    pub fn min_image_count(&self) -> u32 {
        self.surface_capabilities.min_image_count
    }

    pub fn max_image_count(&self) -> u32 {
        self.surface_capabilities.max_image_count
    }

    pub fn raw_supported_transforms(
        &self,
    ) -> ash::vk::SurfaceTransformFlagsKHR {
        self.surface_capabilities.supported_transforms
    }

    pub fn raw_current_transform(&self) -> ash::vk::SurfaceTransformFlagsKHR {
        self.surface_capabilities.current_transform
    }

    pub fn format(&self) -> VulkanImageFormat { self.surface_format }
}
