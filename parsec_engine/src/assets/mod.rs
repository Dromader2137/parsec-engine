use crate::ecs::resources::Resources;

pub mod core;
pub mod library;
pub mod manifest;
pub mod source_file;

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
