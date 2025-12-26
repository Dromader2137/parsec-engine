use crate::ecs::world::component::Component;

#[derive(Debug, Component)]
pub struct Camera {
    pub vertical_fov: f32,
    pub near_clipping_plane: f32,
    pub far_clipping_plane: f32,
}

impl Camera {
    pub fn new(
        vertical_fov: f32,
        near_clipping_plane: f32,
        far_clipping_plane: f32,
    ) -> Camera {
        Camera {
            vertical_fov,
            near_clipping_plane,
            far_clipping_plane,
        }
    }
}
