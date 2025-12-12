use std::sync::atomic::{AtomicU32, Ordering};

use crate::graphics::{
    vulkan::{
        VulkanError, instance::VulkanInstance,
        physical_device::VulkanPhysicalDevice,
    },
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
    surface_format: ash::vk::SurfaceFormatKHR,
    surface_capabilities: ash::vk::SurfaceCapabilitiesKHR,
}

#[derive(Debug)]
pub enum VulkanSurfaceError {
    SupportError(ash::vk::Result),
    CreationError(ash::vk::Result),
    DisplayHandleError(winit::raw_window_handle::HandleError),
    WindowHandleError(winit::raw_window_handle::HandleError),
    FormatsError(ash::vk::Result),
    CapabilitiesError(ash::vk::Result),
    NoSurfaceFormatsAvailable,
    InitialSurfaceBorrowedMoreThanOnce,
}

impl From<VulkanSurfaceError> for VulkanError {
    fn from(value: VulkanSurfaceError) -> Self {
        VulkanError::VulkanSurfaceError(value)
    }
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
                instance.get_entry_raw(),
                instance.get_instance_raw(),
                display_handle.as_raw(),
                window_handle.as_raw(),
                None,
            )
            .map_err(|err| VulkanSurfaceError::CreationError(err))?
        };

        let surface_loader = ash::khr::surface::Instance::new(
            instance.get_entry_raw(),
            instance.get_instance_raw(),
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

    pub fn get_surface_loader(&self) -> &ash::khr::surface::Instance {
        &self.surface_loader
    }

    pub fn get_surface_raw(&self) -> &ash::vk::SurfaceKHR { &self.surface }
}

impl VulkanSurface {
    const ID_COUNTER: AtomicU32 = AtomicU32::new(0);

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
                    *physical_device.get_physical_device_raw(),
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
                    *physical_device.get_physical_device_raw(),
                    initial_surface.surface,
                )
                .map_err(|err| VulkanSurfaceError::CapabilitiesError(err))?
        };

        let id = Self::ID_COUNTER.load(Ordering::Acquire);
        Self::ID_COUNTER.store(id + 1, Ordering::Release);

        Ok(VulkanSurface {
            id,
            window_id,
            physical_device_id: physical_device.id(),
            surface,
            surface_loader,
            surface_format: surface_formats[0],
            surface_capabilities,
        })
    }

    pub fn get_surface_loader_raw(&self) -> &ash::khr::surface::Instance {
        &self.surface_loader
    }

    pub fn get_surface_raw(&self) -> &ash::vk::SurfaceKHR { &self.surface }

    pub fn min_image_count(&self) -> u32 {
        self.surface_capabilities.min_image_count
    }

    pub fn max_image_count(&self) -> u32 {
        self.surface_capabilities.max_image_count
    }

    pub fn supported_transforms(&self) -> ash::vk::SurfaceTransformFlagsKHR {
        self.surface_capabilities.supported_transforms
    }

    pub fn current_transform(&self) -> ash::vk::SurfaceTransformFlagsKHR {
        self.surface_capabilities.current_transform
    }

    pub fn format(&self) -> ash::vk::Format { self.surface_format.format }

    pub fn color_space(&self) -> ash::vk::ColorSpaceKHR {
        self.surface_format.color_space
    }

    pub fn id(&self) -> u32 { self.id }

    pub fn physical_device_id(&self) -> u32 { self.physical_device_id }

    pub fn window_id(&self) -> u32 { self.window_id }
}
