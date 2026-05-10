use parsec_engine_math::{quat::Quat, vec::Vec3f};

use crate::{create_counter, ecs::world::component::Component};

#[derive(Debug, Component)]
pub struct Transform {
    transform_id: u32,
    pub position: Vec3f,
    pub scale: Vec3f,
    pub rotation: Quat,
}

create_counter! {ID_COUNTER}
impl Transform {
    pub fn new(position: Vec3f, scale: Vec3f, rotation: Quat) -> Transform {
        Transform {
            transform_id: ID_COUNTER.next(),
            position,
            scale,
            rotation,
        }
    }

    pub fn transform_id(&self) -> u32 { self.transform_id }
}
