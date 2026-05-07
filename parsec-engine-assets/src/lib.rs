use std::{fs::File, marker::PhantomData};

pub mod assets;

pub struct AssetHandle<T: Asset> {
    pub name: &'static str,
    _marker: PhantomData<T>,
}

pub struct AssetLibrary {}

pub trait Asset {
    type Cooked: serde::Serialize + 'static;

    const ASSET_TYPE: &'static str;
    const EXTENSIONS: &'static [&'static str];

    fn cook(file: File) -> Self::Cooked;
    fn load(cooked: Self::Cooked) -> Self;
}
