use crate::{
    assets::AssetLibrary,
    ecs::{resources::Resources, world::World},
};

pub struct Ctx<'a> {
    pub world: &'a mut World,
    pub resources: &'a mut Resources,
    pub assets: &'a mut AssetLibrary,
}
