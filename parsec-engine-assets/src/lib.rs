use std::marker::PhantomData;

use parsec_engine_ecs::world::World;

pub mod assets;

pub struct AssetHandle<T: Asset> {
    pub name: &'static str,
    _marker: PhantomData<T>,
}

pub struct AssetLibrary {}

pub trait Asset {
    type Cooked: serde::Serialize + serde::de::DeserializeOwned + 'static;

    const ASSET_TYPE: &'static str;
    const EXTENSIONS: &'static [&'static str];

    fn cook(data: &[u8], extension: &str) -> Self::Cooked;
    fn load(cooked: Self::Cooked, world: &World) -> Self;
}
