//! Module responsible for top level error (TO BE DELETED).

use crate::{ecs::world::WorldError, graphics::GraphicsError, resources::ResourceError};

#[derive(Debug)]
pub enum EngineError {
    GraphicsError(GraphicsError),
    WorldError(WorldError),
    ResourceError(ResourceError),
}

pub fn error(error: EngineError) {
    println!("Error: {:?}", error);
}
