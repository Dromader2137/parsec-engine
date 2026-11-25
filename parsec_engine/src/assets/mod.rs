use crate::{
    assets::library::{AssetAddError, AssetCookError, AssetLoadError},
    ecs::resources::Resources,
};

pub mod core;
pub mod library;
mod description;
pub mod source_file;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AssetError {
    #[error("failed to load asset: {0}")]
    Load(#[from] AssetLoadError),
    #[error("failed to cook source file: {0}")]
    Cook(#[from] AssetCookError),
    #[error("failed to add asset: {0}")]
    Add(#[from] AssetAddError),
}

/// Assets get cooked offline and then loaded from cooked on runtime.
pub trait Asset: Sized + 'static {
    type Input: 'static;
    type Cooked: serde::Serialize + serde::de::DeserializeOwned + 'static;
    type CookError: std::error::Error + Send + Sync + 'static;
    type LoadError: std::error::Error + Send + Sync + 'static;

    const ASSET_TYPE: &'static str;

    fn cook(input_data: Self::Input) -> Result<Self::Cooked, Self::CookError>;
    fn load(cooked: Self::Cooked, resources: &mut Resources) -> Result<Self, Self::LoadError>;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssetId(pub(super) u64);
