use crate::error::EngineError;

pub mod library;

pub trait Asset: 'static {
  fn on_load() -> Result<(), AssetError> {
    Ok(())
  }
  fn on_unload() -> Result<(), AssetError> {
    Ok(())
  }
}

#[derive(Debug)]
pub enum AssetError {
  AssetLibraryError,
}

impl From<AssetError> for EngineError {
  fn from(value: AssetError) -> Self {
    EngineError::AssetError(value)
  }
}
