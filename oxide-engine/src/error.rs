use crate::{assets::AssetError, ecs::world::WorldError, graphics::GraphicsError};

#[derive(Debug)]
pub enum EngineError {
    GraphicsError(GraphicsError),
    WorldError(WorldError),
    AssetError(AssetError),
}

pub fn error(error: EngineError) {
    println!("Error: {:?}", error);
}
