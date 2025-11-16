use std::cell::RefCell;

use crate::{ecs::world::component::Component, math::vec::Vec3f};

#[derive(Component, Clone)]
pub struct Transform {
    pub id: u32,
    pub position: Vec3f,
    pub scale: Vec3f,
    pub rotation: Vec3f,
}

impl Transform {
    const ID_COUNTER: RefCell<u32> = RefCell::new(0);

    pub fn new(position: Vec3f, scale: Vec3f, rotation: Vec3f) -> Transform {
        let id_counter = Self::ID_COUNTER;
        let mut borrowed = id_counter.borrow_mut();
        let id = *borrowed;
        *borrowed += 1;
        Transform {
            id,
            position,
            scale,
            rotation,
        }
    }
}
