use crate::graphics::window::WindowWrapper;

use super::{context::VulkanError, physical_device::PhysicalDevice};

pub struct Instance {
    entry: ash::Entry,
    instance: ash::Instance,
}

#[derive(Debug)]
pub enum InstanceError {
    EntryError(ash::LoadingError),
    InstanceCreationError(ash::vk::Result),
    PhysicalDeviceEnumerationError(ash::vk::Result),
    DisplayHandleError(winit::raw_window_handle::HandleError),
    ExtensionEnumerationError(ash::vk::Result)
}

impl From<InstanceError> for VulkanError {
    fn from(value: InstanceError) -> Self {
        VulkanError::InstanceError(value)
    }
}

impl Instance {
    pub fn new(window: &WindowWrapper) -> Result<Instance, InstanceError> {
        let entry = match unsafe { ash::Entry::load() } {
            Ok(val) => val,
            Err(err) => return Err(InstanceError::EntryError(err)),
        };

        let app_info =
            ash::vk::ApplicationInfo::default().api_version(ash::vk::make_api_version(0, 1, 0, 0));
        
        let display_handle = match window.get_display_handle() {
            Ok(val) => val,
            Err(err) => return Err(InstanceError::DisplayHandleError(err))
        };

        let extension_names = match ash_window::enumerate_required_extensions(display_handle.as_raw()) {
            Ok(val) => val,
            Err(err) => return Err(InstanceError::ExtensionEnumerationError(err))
        };

        let create_info = ash::vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(extension_names);

        let instance = match unsafe { entry.create_instance(&create_info, None) } {
            Ok(val) => val,
            Err(err) => return Err(InstanceError::InstanceCreationError(err)),
        };

        Ok(Instance { entry, instance })
    }

    pub fn enumerate_physical_devices(&self) -> Result<Vec<ash::vk::PhysicalDevice>, InstanceError> {
        match unsafe { self.instance.enumerate_physical_devices() } {
            Ok(val) => Ok(val),
            Err(err) => Err(InstanceError::PhysicalDeviceEnumerationError(err))
        }
    }
    
    pub fn get_physical_device_queue_families_properties(&self, physical_device: ash::vk::PhysicalDevice) -> Vec<ash::vk::QueueFamilyProperties> {
        unsafe { self.instance.get_physical_device_queue_family_properties(physical_device) }
    }

    pub fn create_device(&self, physical_device: &PhysicalDevice, create_info: &ash::vk::DeviceCreateInfo) -> Result<ash::Device, ash::vk::Result> {
        unsafe { self.instance.create_device(*physical_device.get_physical_device_raw(), create_info, None) }
    }

    pub fn get_instance_raw(&self) -> &ash::Instance {
        &self.instance
    }

    pub fn get_entry(&self) -> &ash::Entry {
        &self.entry
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe { self.instance.destroy_instance(None) }
    }
}
