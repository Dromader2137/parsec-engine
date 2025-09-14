use crate::{assets::AssetError, ecs::world::WorldError, graphics::GraphicsError, resources::ResourceError};

#[derive(Debug)]
pub enum EngineError {
    GraphicsError(GraphicsError),
    WorldError(WorldError),
    AssetError(AssetError),
    ResourceError(ResourceError)
}

pub fn error(error: EngineError) {
    println!("Error: {:?}", error);
}
