use crate::{ecs::world::World, error::EngineError};

pub mod library;

pub struct AssetLoadInput<'a> {
    pub world: &'a mut World,
}

pub trait Asset: 'static {
    fn on_load(&mut self, state: AssetLoadInput) -> Result<(), AssetError> {
        let _ = state;
        Ok(())
    }
    fn on_unload(&mut self, state: AssetLoadInput) -> Result<(), AssetError> {
        let _ = state;
        Ok(())
    }
}

#[derive(Debug)]
pub enum AssetError {
    AssetLibraryError,
}

impl From<AssetError> for EngineError {
    fn from(value: AssetError) -> Self { EngineError::AssetError(value) }
}
