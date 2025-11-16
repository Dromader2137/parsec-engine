use std::cell::RefCell;

use crate::ecs::world::component::Component;

#[derive(Component, Clone)]
pub struct Camera {
    pub id: u32,
    pub vfov: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    const ID_COUNTER: RefCell<u32> = RefCell::new(0);

    pub fn new(vertical_fov: f32, near_clipping_plane: f32, far_clipping_plane: f32) -> Camera {
        let id_counter = Self::ID_COUNTER;
        let mut borrowed = id_counter.borrow_mut();
        let id = *borrowed;
        *borrowed += 1;
        Camera {
            id,
            vfov: vertical_fov,
            near: near_clipping_plane,
            far: far_clipping_plane,
        }
    }
}
