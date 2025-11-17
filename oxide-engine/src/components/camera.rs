use crate::{ecs::world::component::Component, resources::ResourceCollection};

#[derive(Component, Clone)]
pub struct Camera {
    pub id: u32,
    pub vfov: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new(
        resources: &ResourceCollection,
        vertical_fov: f32,
        near_clipping_plane: f32,
        far_clipping_plane: f32,
    ) -> Camera {
        let mut camera_controller = resources.get_mut::<CameraController>().unwrap();
        let id = camera_controller.id_counter;
        camera_controller.id_counter += 1;
        camera_controller.just_added.push(id);
        Camera {
            id,
            vfov: vertical_fov,
            near: near_clipping_plane,
            far: far_clipping_plane,
        }
    }
}

#[derive(Default)]
pub struct CameraController {
    id_counter: u32,
    pub just_added: Vec<u32>,
}
