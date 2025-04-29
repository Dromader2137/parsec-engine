use std::borrow::Cow;

use crate::graphics::window::WindowWrapper;

use super::context::VulkanError;

pub struct Instance {
    entry: ash::Entry,
    instance: ash::Instance,
    _debug_utils_loader: ash::ext::debug_utils::Instance,
    _debug_call_back: ash::vk::DebugUtilsMessengerEXT,
}

#[derive(Debug)]
pub enum InstanceError {
    EntryError(ash::LoadingError),
    InstanceCreationError(ash::vk::Result),
    PhysicalDeviceEnumerationError(ash::vk::Result),
    DisplayHandleError(winit::raw_window_handle::HandleError),
    ExtensionEnumerationError(ash::vk::Result),
    DebugCreationError(ash::vk::Result)
}

impl From<InstanceError> for VulkanError {
    fn from(value: InstanceError) -> Self {
        VulkanError::InstanceError(value)
    }
}

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: ash::vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: ash::vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const ash::vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    _user_data: *mut std::os::raw::c_void,
) -> ash::vk::Bool32 { unsafe {
    let callback_data = *p_callback_data;
    let message_id_number = callback_data.message_id_number;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        std::ffi::CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        std::ffi::CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    println!(
        "{message_severity:?}:\n{message_type:?} [{message_id_name} ({message_id_number})] : {message}\n",
    );

    ash::vk::FALSE
}}


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

        let mut extension_names = match ash_window::enumerate_required_extensions(display_handle.as_raw()) {
            Ok(val) => val,
            Err(err) => return Err(InstanceError::ExtensionEnumerationError(err))
        }.to_vec();
        extension_names.push(ash::ext::debug_utils::NAME.as_ptr());


        let layer_names = [c"VK_LAYER_KHRONOS_validation"];
        let layers_names_raw: Vec<*const std::ffi::c_char> = layer_names
            .iter()
            .map(|raw_name| raw_name.as_ptr())
            .collect();

        let create_info = ash::vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_layer_names(&layers_names_raw)
            .enabled_extension_names(&extension_names);

        let instance = match unsafe { entry.create_instance(&create_info, None) } {
            Ok(val) => val,
            Err(err) => return Err(InstanceError::InstanceCreationError(err)),
        };

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

        let debug_utils_loader = ash::ext::debug_utils::Instance::new(&entry, &instance);
        let debug_call_back = match unsafe { debug_utils_loader.create_debug_utils_messenger(&debug_info, None) } {
            Ok(val) => val,
            Err(err) => return Err(InstanceError::DebugCreationError(err))
        };

        Ok(Instance { entry, instance, _debug_utils_loader: debug_utils_loader, _debug_call_back: debug_call_back })
    }

    pub fn get_instance_raw(&self) -> &ash::Instance {
        &self.instance
    }

    pub fn get_entry_raw(&self) -> &ash::Entry {
        &self.entry
    }
}
