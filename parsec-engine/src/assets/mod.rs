use std::marker::PhantomData;

use crate::ecs::resources::Resources;

pub mod core;
pub mod library;
pub mod manifest;
pub mod source_file;

#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct AssetId(u64);

#[derive(Debug, PartialEq, Eq)]
pub struct AssetHandle<T: Asset> {
    asset_id: AssetId,
    _marker: PhantomData<T>,
}

impl<T: Asset> Clone for AssetHandle<T> {
    fn clone(&self) -> Self {
        Self {
            asset_id: self.asset_id,
            _marker: PhantomData,
        }
    }
}
impl<T: Asset> Copy for AssetHandle<T> {}
impl<T: Asset> AssetHandle<T> {
    pub(crate) fn new(asset_id: AssetId) -> Self {
        Self {
            asset_id,
            _marker: PhantomData,
        }
    }
}

/// Assets get cooked offline and then loaded from cooked on runtime.
pub trait Asset: 'static {
    type Input: 'static;
    type Cooked: serde::Serialize + serde::de::DeserializeOwned + 'static;

    const ASSET_TYPE: &'static str;

    fn cook(input_data: Self::Input) -> Self::Cooked;
    fn load(cooked: Self::Cooked, resources: &mut Resources) -> Self;
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AssetDescription {
    pub asset_id: AssetId,
    pub asset_name: String,
    pub asset_type: String,
    pub source_file_id: u64,
    pub source_subresource: String,
}
