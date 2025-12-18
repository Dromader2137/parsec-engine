use std::borrow::Cow;

use crate::{graphics::window::Window, utils::id_counter::IdCounter};

pub struct VulkanInstance {
    id: u32,
    entry: ash::Entry,
    instance: ash::Instance,
    _debug_utils_loader: ash::ext::debug_utils::Instance,
    _debug_call_back: Option<ash::vk::DebugUtilsMessengerEXT>,
}

#[derive(Debug, thiserror::Error)]
pub enum VulkanInstanceError {
    #[error("Failed to load vulkan entry: {0}")]
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

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: ash::vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: ash::vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const ash::vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    _user_data: *mut std::os::raw::c_void,
) -> ash::vk::Bool32 {
    unsafe {
        let callback_data = *p_callback_data;
        let message_id_number = callback_data.message_id_number;

        let message_id_name = if callback_data.p_message_id_name.is_null() {
            std::borrow::Cow::from("")
        } else {
            std::ffi::CStr::from_ptr(callback_data.p_message_id_name)
                .to_string_lossy()
        };

        let message = if callback_data.p_message.is_null() {
            Cow::from("")
        } else {
            std::ffi::CStr::from_ptr(callback_data.p_message).to_string_lossy()
        };

        println!(
            "{message_severity:?}: {message_type:?} [{message_id_name} \
             ({message_id_number})] : {message} ",
        );

        ash::vk::FALSE
    }
}

static ID_COUNTER: once_cell::sync::Lazy<IdCounter> =
    once_cell::sync::Lazy::new(|| IdCounter::new(0));
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

        let debug_utils_loader =
            ash::ext::debug_utils::Instance::new(&entry, &instance);
        let debug_call_back = match cfg!(debug_assertions) && false {
            true => {
                let debug_info = ash::vk::DebugUtilsMessengerCreateInfoEXT::default()
                    .message_severity(
                        ash::vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                            | ash::vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                            | ash::vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
                    )
                    .message_type(
                        ash::vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                            | ash::vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                            | ash::vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
                    )
                    .pfn_user_callback(Some(vulkan_debug_callback));

                match unsafe {
                    debug_utils_loader
                        .create_debug_utils_messenger(&debug_info, None)
                } {
                    Ok(val) => Some(val),
                    Err(err) => {
                        return Err(VulkanInstanceError::DebugCreationError(
                            err,
                        ));
                    },
                }
            },
            false => None,
        };

        Ok(VulkanInstance {
            id: ID_COUNTER.next(),
            entry,
            instance,
            _debug_utils_loader: debug_utils_loader,
            _debug_call_back: debug_call_back,
        })
    }

    pub fn get_instance_raw(&self) -> &ash::Instance { &self.instance }

    pub fn get_entry_raw(&self) -> &ash::Entry { &self.entry }

    pub fn id(&self) -> u32 { self.id }
}

impl Drop for VulkanInstance {
    fn drop(&mut self) {
        if let Some(messanger) = self._debug_call_back {
            unsafe {
                self._debug_utils_loader
                    .destroy_debug_utils_messenger(messanger, None)
            };
        }
        unsafe { self.instance.destroy_instance(None) };
    }
}
