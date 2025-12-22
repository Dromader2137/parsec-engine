use crate::{
    ecs::world::component::Component,
    math::{quat::Quat, vec::Vec3f},
};

#[derive(Component)]
pub struct Transform {
    pub position: Vec3f,
    pub scale: Vec3f,
    pub rotation: Quat,
    pub data_id: Option<u32>,
}

impl Transform {
    pub fn new(position: Vec3f, scale: Vec3f, rotation: Quat) -> Transform {
        Transform {
            position,
            scale,
            rotation,
            data_id: None,
        }
    }
}
