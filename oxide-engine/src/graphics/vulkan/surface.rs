use std::sync::Arc;

use crate::graphics::{
    vulkan::{VulkanError, instance::Instance, physical_device::PhysicalDevice},
    window::WindowWrapper,
};

pub struct InitialSurface {
    pub window: Arc<WindowWrapper>,
    pub instance: Arc<Instance>,
    surface: ash::vk::SurfaceKHR,
    surface_loader: ash::khr::surface::Instance,
}

pub struct Surface {
    pub window: Arc<WindowWrapper>,
    pub instance: Arc<Instance>,
    surface: ash::vk::SurfaceKHR,
    surface_loader: ash::khr::surface::Instance,
    surface_format: ash::vk::SurfaceFormatKHR,
    surface_capabilities: ash::vk::SurfaceCapabilitiesKHR,
}

#[derive(Debug)]
pub enum SurfaceError {
    SupportError(ash::vk::Result),
    CreationError(ash::vk::Result),
    DisplayHandleError(winit::raw_window_handle::HandleError),
    WindowHandleError(winit::raw_window_handle::HandleError),
    FormatsError(ash::vk::Result),
    CapabilitiesError(ash::vk::Result),
    NoSurfaceFormatsAvailable,
    InitialSurfaceBorrowedMoreThanOnce,
}

impl From<SurfaceError> for VulkanError {
    fn from(value: SurfaceError) -> Self {
        VulkanError::SurfaceError(value)
    }
}

impl InitialSurface {
    pub fn new(
        instance: Arc<Instance>,
        window: Arc<WindowWrapper>,
    ) -> Result<Arc<InitialSurface>, SurfaceError> {
        let display_handle = match window.raw_display_handle() {
            Ok(val) => val,
            Err(err) => return Err(SurfaceError::DisplayHandleError(err)),
        };

        let window_handle = match window.raw_window_handle() {
            Ok(val) => val,
            Err(err) => return Err(SurfaceError::WindowHandleError(err)),
        };

        let surface = match unsafe {
            ash_window::create_surface(
                instance.get_entry_raw(),
                instance.get_instance_raw(),
                display_handle.as_raw(),
                window_handle.as_raw(),
                None,
            )
        } {
            Ok(val) => val,
            Err(err) => return Err(SurfaceError::CreationError(err)),
        };

        let surface_loader =
            ash::khr::surface::Instance::new(instance.get_entry_raw(), instance.get_instance_raw());

        Ok(Arc::new(InitialSurface {
            window,
            instance,
            surface,
            surface_loader,
        }))
    }

    pub fn check_surface_support(
        &self,
        physical_device: ash::vk::PhysicalDevice,
        queue_family_index: u32,
    ) -> Result<bool, SurfaceError> {
        match unsafe {
            self.surface_loader.get_physical_device_surface_support(
                physical_device,
                queue_family_index,
                self.surface,
            )
        } {
            Ok(val) => Ok(val),
            Err(err) => Err(SurfaceError::SupportError(err)),
        }
    }

    pub fn get_surface_loader(&self) -> &ash::khr::surface::Instance {
        &self.surface_loader
    }

    pub fn get_surface_raw(&self) -> &ash::vk::SurfaceKHR {
        &self.surface
    }
}

impl Surface {
    pub fn from_initial_surface(
        initial_surface_arc: Arc<InitialSurface>,
        physical_device: Arc<PhysicalDevice>,
    ) -> Result<Arc<Surface>, SurfaceError> {
        let initial_surface = match Arc::into_inner(initial_surface_arc) {
            Some(val) => val,
            None => return Err(SurfaceError::InitialSurfaceBorrowedMoreThanOnce),
        };

        let surface_formats = match unsafe {
            initial_surface
                .surface_loader
                .get_physical_device_surface_formats(
                    *physical_device.get_physical_device_raw(),
                    initial_surface.surface,
                )
        } {
            Ok(val) => val,
            Err(err) => return Err(SurfaceError::FormatsError(err)),
        };

        if surface_formats.is_empty() {
            return Err(SurfaceError::NoSurfaceFormatsAvailable);
        }

        let surface_capabilities = match unsafe {
            initial_surface
                .surface_loader
                .get_physical_device_surface_capabilities(
                    *physical_device.get_physical_device_raw(),
                    initial_surface.surface,
                )
        } {
            Ok(val) => val,
            Err(err) => return Err(SurfaceError::CapabilitiesError(err)),
        };

        let InitialSurface {
            window,
            instance,
            surface,
            surface_loader,
        } = initial_surface;

        Ok(Arc::new(Surface {
            window,
            instance,
            surface,
            surface_loader,
            surface_format: surface_formats[0],
            surface_capabilities,
        }))
    }

    pub fn get_surface_loader_raw(&self) -> &ash::khr::surface::Instance {
        &self.surface_loader
    }

    pub fn get_surface_raw(&self) -> &ash::vk::SurfaceKHR {
        &self.surface
    }

    pub fn min_image_count(&self) -> u32 {
        self.surface_capabilities.min_image_count
    }

    pub fn max_image_count(&self) -> u32 {
        self.surface_capabilities.max_image_count
    }

    pub fn current_extent(&self) -> ash::vk::Extent2D {
        match self.surface_capabilities.current_extent.width {
            u32::MAX => ash::vk::Extent2D {
                width: self.window.width(),
                height: self.window.height(),
            },
            _ => self.surface_capabilities.current_extent,
        }
    }

    pub fn width(&self) -> u32 {
        self.current_extent().width
    }

    pub fn height(&self) -> u32 {
        self.current_extent().height
    }

    pub fn aspect_ratio(&self) -> f32 {
        if self.height() == 0 {
            return 1.0;
        }
        self.width() as f32 / self.height() as f32
    }

    pub fn supported_transforms(&self) -> ash::vk::SurfaceTransformFlagsKHR {
        self.surface_capabilities.supported_transforms
    }

    pub fn current_transform(&self) -> ash::vk::SurfaceTransformFlagsKHR {
        self.surface_capabilities.current_transform
    }

    pub fn format(&self) -> ash::vk::Format {
        self.surface_format.format
    }

    pub fn color_space(&self) -> ash::vk::ColorSpaceKHR {
        self.surface_format.color_space
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe { self.surface_loader.destroy_surface(self.surface, None) };
    }
}
