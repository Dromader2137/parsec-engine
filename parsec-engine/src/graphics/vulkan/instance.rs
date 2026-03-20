use crate::graphics::window::Window;

pub struct VulkanInstance {
    entry: ash::Entry,
    instance: ash::Instance,
}

#[derive(Debug, thiserror::Error)]
pub enum VulkanInstanceError {
    #[error("Failed to load Vulkan entry: {0}")]
    EntryError(ash::LoadingError),
    #[error("Failed to create instance: {0}")]
    InstanceCreationError(ash::vk::Result),
    #[error("Failed to enumarate physical devices: {0}")]
    PhysicalDeviceEnumerationError(ash::vk::Result),
    #[error("Failed to get display error: {0}")]
    DisplayHandleError(winit::raw_window_handle::HandleError),
    #[error("Failed to enumarate vulkan extension: {0}")]
    ExtensionEnumerationError(ash::vk::Result),
    #[error("Failed to create vulkan debug callback: {0}")]
    DebugCreationError(ash::vk::Result),
}

impl VulkanInstance {
    pub fn new(window: &Window) -> Result<VulkanInstance, VulkanInstanceError> {
        let entry = match unsafe { ash::Entry::load() } {
            Ok(val) => val,
            Err(err) => return Err(VulkanInstanceError::EntryError(err)),
        };

        let app_info = ash::vk::ApplicationInfo::default()
            .api_version(ash::vk::make_api_version(0, 1, 0, 0));

        let display_handle = match window.raw_display_handle() {
            Ok(val) => val,
            Err(err) => {
                return Err(VulkanInstanceError::DisplayHandleError(err));
            },
        };

        let mut extension_names =
            match ash_window::enumerate_required_extensions(
                display_handle.as_raw(),
            ) {
                Ok(val) => val,
                Err(err) => {
                    return Err(
                        VulkanInstanceError::ExtensionEnumerationError(err),
                    );
                },
            }
            .to_vec();
        extension_names.push(ash::ext::debug_utils::NAME.as_ptr());

        let layer_names_raw = [];

        let create_info = ash::vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_layer_names(&layer_names_raw)
            .enabled_extension_names(&extension_names);

        let instance = match unsafe {
            entry.create_instance(&create_info, None)
        } {
            Ok(val) => val,
            Err(err) => {
                return Err(VulkanInstanceError::InstanceCreationError(err));
            },
        };

        Ok(VulkanInstance { entry, instance })
    }

    pub fn raw_handle(&self) -> &ash::Instance { &self.instance }

    pub fn raw_entry(&self) -> &ash::Entry { &self.entry }
}

impl Drop for VulkanInstance {
    fn drop(&mut self) { unsafe { self.instance.destroy_instance(None) }; }
}
