use crate::{arena::handle::{Handle, WeakHandle}, graphics::{vulkan::{VulkanBackend, physical_device::VulkanPhysicalDevice, surface::VulkanSurface}, window::Window}};

pub struct VulkanInstance {
    id: u32,
    entry: ash::Entry,
    instance: ash::Instance,
    physical_devices: Vec<Handle<VulkanPhysicalDevice>>,
    pub surfaces: Vec<Handle<VulkanSurface>>
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

crate::create_counter! {ID_COUNTER}
impl VulkanInstance {
    pub fn new(
        arenas: &mut VulkanBackend,
        window: &Window
    ) -> Result<Handle<VulkanInstance>, VulkanInstanceError> {
        let entry = match unsafe { ash::Entry::load() } {
            Ok(val) => val,
            Err(err) => return Err(VulkanInstanceError::EntryError(err)),
        };

        let app_info = ash::vk::ApplicationInfo::default()
            .api_version(ash::vk::make_api_version(0, 1, 1, 0));

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

        let layer_names = match cfg!(debug_assertions) {
            true => vec![c"VK_LAYER_KHRONOS_validation"],
            false => vec![],
        };
        let layers_names_raw: Vec<*const std::ffi::c_char> = layer_names
            .iter()
            .map(|raw_name| raw_name.as_ptr())
            .collect();

        let create_info = ash::vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_layer_names(&layers_names_raw)
            .enabled_extension_names(&extension_names);

        let instance = match unsafe {
            entry.create_instance(&create_info, None)
        } {
            Ok(val) => val,
            Err(err) => {
                return Err(VulkanInstanceError::InstanceCreationError(err));
            },
        };
        
        let ret = VulkanInstance {
            id: ID_COUNTER.next(),
            entry,
            instance,
            physical_devices: Vec::new(),
            surfaces: Vec::new()
        };

        Ok(arenas.instances.add(ret))
    }

    pub fn raw_instance(&self) -> &ash::Instance { &self.instance }

    pub fn raw_entry(&self) -> &ash::Entry { &self.entry }

    pub fn id(&self) -> u32 { self.id }
}

impl Drop for VulkanInstance {
    fn drop(&mut self) {
        unsafe { self.instance.destroy_instance(None) };
    }
}
