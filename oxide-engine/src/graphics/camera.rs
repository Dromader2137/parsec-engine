use crate::{graphics::{renderer::VulkanRenderer, vulkan::VulkanError}, math::mat::Matrix4f};

pub struct Camera {
    pub projection: Matrix4f,
    buffer_id: u32,
}

impl Camera {
    pub fn new(renderer: &mut VulkanRenderer, fov: f32, near: f32, far: f32) -> Result<Camera, VulkanError> {
        let projection = Matrix4f::perspective(fov, renderer.get_aspect_ratio(), near, far);
        Ok(Camera {
            projection,
            buffer_id: renderer.create_buffer(vec![projection])?,
        })
    }

    pub fn projection_buffer_id(&self) -> u32 {
        self.buffer_id
    }
}
