//! Module responsible for the ECS subsystem.

pub mod entity;
pub mod resources;
pub mod system;
pub mod world;

use crate::ecs::{resources::ResourceError, world::WorldError};

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum EcsError {
    #[error("world error: {0}")]
    World(#[from] WorldError),
    #[error("resource error: {0}")]
    Resource(#[from] ResourceError),
}
