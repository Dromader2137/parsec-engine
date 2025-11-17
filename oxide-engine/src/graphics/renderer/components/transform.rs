use crate::{
    ecs::world::component::Component, graphics::renderer::transform_data::create_transform_data,
    math::vec::Vec3f, resources::ResourceCollection,
};

#[derive(Component)]
pub struct Transform {
    pub position: Vec3f,
    pub scale: Vec3f,
    pub rotation: Vec3f,
    pub data_id: u32,
}

impl Transform {
    pub fn new(
        resources: &ResourceCollection,
        position: Vec3f,
        scale: Vec3f,
        rotation: Vec3f,
    ) -> Transform {
        let data_id = create_transform_data(resources, position, scale, rotation).unwrap();
        Transform {
            position,
            scale,
            rotation,
            data_id,
        }
    }
}
