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
    ecs::world::World,
    error::{ParsecError, StrError},
};

pub mod assets;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AssetHandle<T: Asset> {
    name: &'static str,
    _marker: PhantomData<T>,
}

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
    manifest: Manifest,
    assets: HashMap<TypeId, Vec<(&'static str, Box<dyn Any>)>>,
}

impl AssetLibrary {
    pub fn new() -> AssetLibrary {
        AssetLibrary {
            manifest: Manifest::load(),
            assets: HashMap::new(),
        }
    }

    pub fn load<T: Asset>(
        &mut self,
        name: &'static str,
        world: &World,
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

        let asset = T::load(cooked, world);
        let asset_vec =
            self.assets.entry(TypeId::of::<T>()).or_insert(Vec::new());
        asset_vec.push((name, Box::new(asset) as Box<dyn Any>));
        Ok(AssetHandle::new(name))
    }

    pub fn get<T: Asset>(&self, handle: AssetHandle<T>) -> &T {
        let name = handle.name;
        let asset_vec = self.assets.get(&TypeId::of::<T>()).unwrap();
        let (_, asset_any) =
            asset_vec.iter().find(|(n, _)| *n == name).unwrap();
        asset_any.downcast_ref::<T>().unwrap()
    }
}

pub trait Asset: 'static {
    type Cooked: serde::Serialize + serde::de::DeserializeOwned + 'static;

    const ASSET_TYPE: &'static str;
    const EXTENSIONS: &'static [&'static str];

    fn cook(data: &[u8], extension: &str) -> Self::Cooked;
    fn load(cooked: Self::Cooked, world: &World) -> Self;
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AssetDescription {
    pub name: String,
    pub path: PathBuf,
    pub last_cooked: Option<SystemTime>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Manifest {
    pub verison: u8,
    pub assets: HashMap<String, AssetDescription>,
}

impl Manifest {
    pub fn new() -> Self {
        Self {
            verison: 0,
            assets: HashMap::new(),
        }
    }

    pub fn load() -> Manifest {
        let try_file = File::options().read(true).open("./assets.json");
        if matches!(
            try_file.as_ref().map_err(|err| err.kind()),
            Err(std::io::ErrorKind::NotFound)
        ) {
            Manifest::new()
        } else {
            let file = try_file.expect("Failed to open assets.json");
            let reader = BufReader::new(file);
            let manifest = serde_json::from_reader(reader)
                .expect("Failed to parse assets.json");
            manifest
        }
    }
}
