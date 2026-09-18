use std::{
    any::{Any, TypeId},
    collections::HashMap,
    path::PathBuf,
};

use crate::{
    assets::{Asset, AssetHandle, AssetId, manifest::Manifest},
    ecs::resources::Resources,
    error::{ParsecError, StrError},
};

#[derive(Debug)]
pub struct AssetLibrary {
    manifest: Manifest,
    assets: HashMap<AssetId, (Box<dyn Any>, TypeId)>
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
        resources: &mut Resources,
    ) -> Result<AssetHandle<T>, ParsecError> {
        if self
            .manifest
            .assets
            .iter()
            .find(|x| &x.asset_name == name)
            .is_none()
        {
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

    pub fn get<T: Asset>(&self, handle: AssetHandle<T>) -> Option<&T> {
        let name = handle.asset_id;
        let asset_vec = self.assets.get(&TypeId::of::<T>())?;
        let (_, asset_any) = asset_vec.iter().find(|(n, _)| *n == name)?;
        asset_any.downcast_ref::<T>()
    }
}
