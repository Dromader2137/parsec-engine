use crate::graphics::{
    vulkan::{image::VulkanImageFormat, instance::VulkanInstance, physical_device::VulkanPhysicalDevice},
    window::Window,
};

pub struct VulkanInitialSurface {
    window_id: u32,
    surface_loader: ash::khr::surface::Instance,
    surface: ash::vk::SurfaceKHR,
}

pub struct VulkanSurface {
    id: u32,
    window_id: u32,
    physical_device_id: u32,
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
            .map_err(|err| VulkanSurfaceError::DisplayHandleError(err))?;
        let window_handle = window
            .raw_window_handle()
            .map_err(|err| VulkanSurfaceError::WindowHandleError(err))?;

        let surface = unsafe {
            ash_window::create_surface(
                instance.raw_entry(),
                instance.raw_instance(),
                display_handle.as_raw(),
                window_handle.as_raw(),
                None,
            )
            .map_err(|err| VulkanSurfaceError::CreationError(err))?
        };

        let surface_loader = ash::khr::surface::Instance::new(
            instance.raw_entry(),
            instance.raw_instance(),
        );

        Ok(VulkanInitialSurface {
            window_id: window.id(),
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
                .map_err(|err| VulkanSurfaceError::SupportError(err))
        }
    }

    pub fn surface_loader_raw(&self) -> &ash::khr::surface::Instance {
        &self.surface_loader
    }

    pub fn surface_raw(&self) -> &ash::vk::SurfaceKHR { &self.surface }
}

crate::create_counter! {ID_COUNTER}
impl VulkanSurface {
    pub fn from_initial_surface(
        initial_surface: VulkanInitialSurface,
        physical_device: &VulkanPhysicalDevice,
    ) -> Result<VulkanSurface, VulkanSurfaceError> {
        let VulkanInitialSurface {
            surface_loader,
            surface,
            window_id,
            ..
        } = initial_surface;

        let surface_formats = unsafe {
            surface_loader
                .get_physical_device_surface_formats(
                    *physical_device.raw_physical_device(),
                    initial_surface.surface,
                )
                .map_err(|err| VulkanSurfaceError::FormatsError(err))?
        };

        if surface_formats.is_empty() {
            return Err(VulkanSurfaceError::NoSurfaceFormatsAvailable);
        }

        let surface_capabilities = unsafe {
            surface_loader
                .get_physical_device_surface_capabilities(
                    *physical_device.raw_physical_device(),
                    initial_surface.surface,
                )
                .map_err(|err| VulkanSurfaceError::CapabilitiesError(err))?
        };

        let preferred_formats = [
            VulkanImageFormat::RGBA8SRGB,
            VulkanImageFormat::BGRA8SRGB,
        ];

        let surface_format = *preferred_formats
            .iter()
            .find_map(|preffered_format| {
                let found = surface_formats
                    .iter()
                    .find(|format| format.format == preffered_format.raw_image_format());
                if found.is_some() { Some(preffered_format) }
                else { None }
            })
            .ok_or(VulkanSurfaceError::NoSurfaceFormatsAvailable)?;

        Ok(VulkanSurface {
            id: ID_COUNTER.next(),
            window_id,
            physical_device_id: physical_device.id(),
            surface,
            surface_loader,
            surface_format,
            surface_capabilities,
        })
    }

    pub fn raw_surface_loader(&self) -> &ash::khr::surface::Instance {
        &self.surface_loader
    }

    pub fn raw_surface(&self) -> &ash::vk::SurfaceKHR { &self.surface }

    pub fn min_image_count(&self) -> u32 {
        self.surface_capabilities.min_image_count
    }

    pub fn max_image_count(&self) -> u32 {
        self.surface_capabilities.max_image_count
    }

    pub fn raw_supported_transforms(&self) -> ash::vk::SurfaceTransformFlagsKHR {
        self.surface_capabilities.supported_transforms
    }

    pub fn raw_current_transform(&self) -> ash::vk::SurfaceTransformFlagsKHR {
        self.surface_capabilities.current_transform
    }

    pub fn format(&self) -> VulkanImageFormat { self.surface_format }

    pub fn id(&self) -> u32 { self.id }

    pub fn physical_device_id(&self) -> u32 { self.physical_device_id }

    pub fn window_id(&self) -> u32 { self.window_id }
}
