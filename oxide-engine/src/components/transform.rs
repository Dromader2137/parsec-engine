use crate::{ecs::world::component::Component, math::vec::Vec3f, resources::ResourceCollection};

#[derive(Component, Clone)]
pub struct Transform {
    pub id: u32,
    pub position: Vec3f,
    pub scale: Vec3f,
    pub rotation: Vec3f,
}

impl Transform {
    pub fn new(resources: &ResourceCollection, position: Vec3f, scale: Vec3f, rotation: Vec3f) -> Transform {
        let mut transform_controller = resources.get_mut::<TransformController>().unwrap();
        let id = transform_controller.id_counter;
        transform_controller.id_counter += 1;
        transform_controller.just_added.push(id);
        Transform {
            id,
            position,
            scale,
            rotation,
        }
    }
}

#[derive(Default)]
pub struct TransformController {
    id_counter: u32,
    pub just_added: Vec<u32>,
}
