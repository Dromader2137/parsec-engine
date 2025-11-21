use crate::{
    ecs::world::component::Component, graphics::renderer::camera_data::create_camera_data,
};

#[derive(Component)]
pub struct Camera {
    pub vertical_fov: f32,
    pub near_clipping_plane: f32,
    pub far_clipping_plane: f32,
    pub data_id: u32,
}

impl Camera {
    pub fn new(vertical_fov: f32, near_clipping_plane: f32, far_clipping_plane: f32) -> Camera {
        let data_id =
            create_camera_data(vertical_fov, near_clipping_plane, far_clipping_plane).unwrap();
        Camera {
            vertical_fov,
            near_clipping_plane,
            far_clipping_plane,
            data_id,
        }
    }
}
