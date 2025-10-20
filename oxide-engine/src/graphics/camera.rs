use crate::{graphics::{renderer::VulkanRenderer, vulkan::VulkanError}, math::mat::Matrix4f};

pub struct CameraData {
    pub projection_matrix: Matrix4f,
    pub view_matrix: Matrix4f,
    buffer_id: u32,
}

impl CameraData {
    pub fn new(renderer: &mut VulkanRenderer) -> Result<CameraData, VulkanError> {
        let projection = Matrix4f::perspective(fov, renderer.get_aspect_ratio(), near, far);
        Ok(CameraData {
            projection_matrix: projection,
            view_matrix: Matrix4f::indentity(),
            buffer_id: renderer.create_buffer(vec![projection])?,
        })
    }

    pub fn projection_buffer_id(&self) -> u32 {
        self.buffer_id
    }
}
