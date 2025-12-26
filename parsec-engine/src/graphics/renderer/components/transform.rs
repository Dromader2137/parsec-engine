use crate::{
    ecs::world::component::Component,
    math::{quat::Quat, vec::Vec3f},
};

#[derive(Debug, Component)]
pub struct Transform {
    pub position: Vec3f,
    pub scale: Vec3f,
    pub rotation: Quat,
}

impl Transform {
    pub fn new(position: Vec3f, scale: Vec3f, rotation: Quat) -> Transform {
        Transform {
            position,
            scale,
            rotation,
        }
    }
}
