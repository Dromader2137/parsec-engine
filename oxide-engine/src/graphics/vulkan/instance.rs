use super::context::VulkanContextError;

pub struct Instance {
    entry: ash::Entry,
    instance: ash::Instance,
}

#[derive(Debug)]
pub enum InstanceError {
    EntryError(ash::LoadingError),
    InstanceCreationError(ash::vk::Result),
}

impl From<InstanceError> for VulkanContextError {
    fn from(value: InstanceError) -> Self {
        VulkanContextError::InstanceError(value)
    }
}

impl Instance {
    pub fn new() -> Result<Instance, InstanceError> {
        let entry = match unsafe { ash::Entry::load() } {
            Ok(val) => val,
            Err(err) => return Err(InstanceError::EntryError(err)),
        };

        let app_info =
            ash::vk::ApplicationInfo::default().api_version(ash::vk::make_api_version(0, 1, 0, 0));

        let create_info = ash::vk::InstanceCreateInfo::default().application_info(&app_info);

        let instance = match unsafe { entry.create_instance(&create_info, None) } {
            Ok(val) => val,
            Err(err) => return Err(InstanceError::InstanceCreationError(err)),
        };

        Ok(Instance { entry, instance })
    }
}
