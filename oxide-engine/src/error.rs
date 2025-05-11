use crate::{assets::AssetError, graphics::GraphicsError, world::WorldError};

#[derive(Debug)]
pub enum EngineError {
    GraphicsError(GraphicsError),
    WorldError(WorldError),
    AssetError(AssetError),
}

pub fn error(error: EngineError) {
    println!("Error: {:?}", error);
}
