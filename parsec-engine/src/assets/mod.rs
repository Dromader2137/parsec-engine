use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fs::File,
    io::BufReader,
    marker::PhantomData,
    path::PathBuf,
    time::SystemTime,
};

use crate::{
    ecs::resources::Resources,
    error::{ParsecError, StrError},
};

pub mod core;

#[derive(Debug, PartialEq, Eq)]
pub struct AssetHandle<T: Asset> {
    name: &'static str,
    _marker: PhantomData<T>,
}

impl<T: Asset> Clone for AssetHandle<T> {
    fn clone(&self) -> Self {
        Self {
            name: self.name,
            _marker: PhantomData,
        }
    }
}
impl<T: Asset> Copy for AssetHandle<T> {}
impl<T: Asset> AssetHandle<T> {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            _marker: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct AssetLibrary {
    manifest: AssetsManifest,
    assets: HashMap<TypeId, Vec<(&'static str, Box<dyn Any>)>>,
}

impl AssetLibrary {
    pub fn new() -> AssetLibrary {
        AssetLibrary {
            manifest: AssetsManifest::load(),
            assets: HashMap::new(),
        }
    }

    pub fn get_handle<T: Asset>(
        &mut self,
        name: &'static str,
        resources: &mut Resources,
    ) -> Result<AssetHandle<T>, ParsecError> {
        if !self.manifest.assets.contains_key(name) {
            return Err(
                StrError("Asset library doesn't contain this asset").into()
            );
        }

        let bytes = std::fs::read(
            PathBuf::new()
                .join("assets")
                .join(name)
                .with_extension("asset"),
        )?;
        let cooked = postcard::from_bytes::<T::Cooked>(&bytes)?;

        let asset = T::load(cooked, resources);
        let asset_vec =
            self.assets.entry(TypeId::of::<T>()).or_insert(Vec::new());
        asset_vec.push((name, Box::new(asset) as Box<dyn Any>));
        Ok(AssetHandle::new(name))
    }

    pub fn get_data<T: Asset>(&self, handle: AssetHandle<T>) -> &T {
        let name = handle.name;
        let asset_vec = self
            .assets
            .get(&TypeId::of::<T>())
            .expect("Asset handles are always valid");
        let (_, asset_any) = asset_vec
            .iter()
            .find(|(n, _)| *n == name)
            .expect("Asset handles are always valid");
        asset_any
            .downcast_ref::<T>()
            .expect("Asset handles are always valid")
    }
}

pub trait Asset: Send + Sync + 'static {
    type Cooked: serde::Serialize + serde::de::DeserializeOwned + 'static;

    const ASSET_TYPE: &'static str;
    const EXTENSIONS: &'static [&'static str];

    fn cook(data: &[u8], extension: &str) -> Self::Cooked;
    fn load(cooked: Self::Cooked, resources: &mut Resources) -> Self;
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AssetDescription {
    pub name: String,
    pub path: PathBuf,
    pub last_cooked: Option<SystemTime>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AssetsManifest {
    pub verison: u8,
    pub assets: HashMap<String, AssetDescription>,
}

impl AssetsManifest {
    pub fn new() -> Self {
        Self {
            verison: 0,
            assets: HashMap::new(),
        }
    }

    pub fn load() -> AssetsManifest {
        let try_file = File::options().read(true).open("./assets.json");
        if matches!(
            try_file.as_ref().map_err(|err| err.kind()),
            Err(std::io::ErrorKind::NotFound)
        ) {
            AssetsManifest::new()
        } else {
            let file = try_file.expect("Failed to open assets.json");
            let reader = BufReader::new(file);
            let manifest = serde_json::from_reader(reader)
                .expect("Failed to parse assets.json");
            manifest
        }
    }
}
