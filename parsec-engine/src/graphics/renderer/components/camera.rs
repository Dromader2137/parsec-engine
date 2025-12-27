use crate::ecs::world::component::Component;

#[derive(Debug, Component)]
pub struct Camera {
    camera_id: u32,
    pub vertical_fov: f32,
    pub near_clipping_plane: f32,
    pub far_clipping_plane: f32,
}

crate::create_counter!{ID_COUNTER}
impl Camera {
    pub fn new(
        vertical_fov: f32,
        near_clipping_plane: f32,
        far_clipping_plane: f32,
    ) -> Camera {
        Camera {
            camera_id: ID_COUNTER.next(),
            vertical_fov,
            near_clipping_plane,
            far_clipping_plane,
        }
    }

    pub fn camera_id(&self) -> u32 {
        self.camera_id
    }
}
