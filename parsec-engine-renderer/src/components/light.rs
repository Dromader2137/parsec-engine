use parsec_engine_ecs::world::component::Component;
use parsec_engine_math::vec::Vec3f;
use parsec_engine_utils::create_counter;

#[derive(Debug, Component)]
pub struct Light {
    light_id: u32,
    pub direction: Vec3f,
    pub up: Vec3f,
    pub color: Vec3f,
}

create_counter! {ID_COUNTER}
impl Light {
    pub fn new(direction: Vec3f, up: Vec3f, color: Vec3f) -> Self {
        Self {
            light_id: ID_COUNTER.next(),
            direction,
            up,
            color,
        }
    }

    pub fn light_id(&self) -> u32 { self.light_id }
}
