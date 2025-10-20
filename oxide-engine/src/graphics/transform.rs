use crate::{graphics::{renderer::VulkanRenderer, vulkan::VulkanError}, math::mat::Matrix4f};

#[derive(Debug)]
pub struct TransformData {
    pub transform_matrix: Matrix4f,
    buffer_id: u32
}

impl TransformData {
    pub fn new(renderer: &mut VulkanRenderer) -> Result<TransformData, VulkanError> {
        Ok(TransformData { 
            transform_matrix: Matrix4f::indentity(), 
            buffer_id: renderer.create_buffer(vec![Matrix4f::indentity()])?
        })
    }
}
