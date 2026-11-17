use crate::{assets::library::AssetLibraryError, ecs::resources::Resources};

pub mod core;
pub mod library;
pub mod manifest;
pub mod source_file;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AssetError {
    #[error("failed to load asset: {0}")] 
    Load(#[from] AssetLoadError),
    #[error("failed to resolve asset: {0}")] 
    Resolve(#[from] AssetResolveError),
    #[error("failed to add asset: {0}")] 
    Add(#[from] AssetAddError),
    #[error("failed to cook source file: {0}")] 
    Cook(#[from] AssetCookError),
}

/// Assets get cooked offline and then loaded from cooked on runtime.
pub trait Asset: 'static {
    type Input: 'static;
    type Cooked: serde::Serialize + serde::de::DeserializeOwned + 'static;

    const ASSET_TYPE: &'static str;

    fn cook(input_data: Self::Input) -> Self::Cooked;
    fn load(cooked: Self::Cooked, resources: &mut Resources) -> Self;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssetId(pub(super) u64);
