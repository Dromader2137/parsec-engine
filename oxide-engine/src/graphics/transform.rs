use crate::{graphics::{renderer::create_buffer, vulkan::VulkanError}, math::mat::Matrix4f, resources::ResourceCollection, utils::id_vec::IdVec};

#[derive(Debug)]
pub struct TransformData {
    pub transform_matrix: Matrix4f,
    buffer_id: u32
}

impl TransformData {
}

pub fn create_transform_data(resources: &mut ResourceCollection) -> Result<u32, VulkanError> {
    let buffer_id = create_buffer(resources, vec![Matrix4f::indentity()])?;
    let mut transforms = resources.get_mut::<IdVec<TransformData>>().unwrap();
    let transform_data = TransformData {
        transform_matrix: Matrix4f::indentity(),
        buffer_id
    };
    Ok(transforms.push(transform_data))
}
