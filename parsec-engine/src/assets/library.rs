use std::{
    any::Any,
    collections::{BTreeSet, HashMap},
    marker::PhantomData,
    path::{Path, PathBuf},
};

use crate::{
    assets::{
        Asset, AssetId,
        manifest::{AssetDescription, cooked_path},
        source_file::{
            AssetCookContext, AssetSourceFile, AssetSourceFileHandler,
            AssetSourceId,
        },
    },
    ecs::resources::Resources,
    error::ParsecError
};

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

#[derive(Debug, Default)]
enum AssetSlot {
    #[default]
    Unloaded,
    Loading,
    Loaded(Box<dyn Any>),
    Failed(ParsecError),
}

#[derive(Debug, thiserror::Error)]
pub enum AssetTrackingError {
    #[error("asset source file {0:?} has no valid extension")]
    InvalidExtension(PathBuf),
    #[error("no asset source file handler registered for extension {0:?}")]
    HandlerNotFound(String),
    #[error("asset source file {0:?} doesn't exist")]
    FileNotFound(PathBuf),
    #[error("asset name {0:?} is already used by a different asset")]
    DuplicateName(String),
    #[error("asset library doesn't contain asset {0:?}")]
    AssetNotFound(String),
    #[error("asset {0:?} has a different type than requested")]
    TypeMismatch(String),
    #[error("failed to load asset {0:?}")]
    LoadFailed(String),
}

type SourceFileHandlerFn =
    fn(&Path, AssetCookContext) -> Result<(), ParsecError>;

#[derive(Debug)]
pub struct AssetLibrary {
    assets: HashMap<AssetId, (AssetDescription, AssetSlot)>,
    names: HashMap<String, AssetId>,
    files: HashMap<AssetSourceId, AssetSourceFile>,
    source_file_handlers: HashMap<&'static str, SourceFileHandlerFn>,
}

impl AssetLibrary {
    pub fn new() -> AssetLibrary {
        AssetLibrary {
            assets: HashMap::new(),
            names: HashMap::new(),
            files: HashMap::new(),
            source_file_handlers: HashMap::new(),
        }
    }

    pub fn add_asset(
        &mut self,
        asset_desc: AssetDescription,
    ) -> Result<(), AssetTrackingError> {
        let asset_id = AssetId(self.assets.len() as u64);
        let name = asset_desc.name();
        if self.names.contains_key(&name) {
            return Err(AssetTrackingError::DuplicateName(name));
        }
        self.names.insert(name, asset_id);
        let source_id = self
            .source_file_by_path(&asset_desc.source_file)
            .unwrap_or(AssetSourceId(self.files.len() as u64));
        self.files
            .entry(source_id)
            .or_insert_with(|| AssetSourceFile {
                filepath: asset_desc.source_file.clone(),
                asset_ids: BTreeSet::new(),
            })
            .asset_ids
            .insert(asset_id);
        self.assets
            .insert(asset_id, (asset_desc, AssetSlot::Unloaded));
        Ok(())
    }

    pub fn register_source_file_handler<H: AssetSourceFileHandler>(&mut self) {
        self.source_file_handlers.insert(H::EXTENSION, H::cook);
    }

    pub fn source_file_by_path(&self, path: &Path) -> Option<AssetSourceId> {
        self.files
            .iter()
            .find(|(_, file)| file.filepath == path)
            .map(|(&id, _)| id)
    }

    pub fn source_file(
        &self,
        source_id: AssetSourceId,
    ) -> Option<&AssetSourceFile> {
        self.files.get(&source_id)
    }

    pub fn source_files(
        &self,
    ) -> impl Iterator<Item = (AssetSourceId, &AssetSourceFile)> {
        self.files.iter().map(|(&id, file)| (id, file))
    }

    pub fn asset_description(
        &self,
        asset_id: AssetId,
    ) -> Option<&AssetDescription> {
        self.assets.get(&asset_id).map(|(desc, _)| desc)
    }

    fn cook_source_file(&self, path: &Path) -> Result<(), ParsecError> {
        let extension = path
            .extension()
            .and_then(|x| x.to_str())
            .ok_or(AssetTrackingError::InvalidExtension(path.to_path_buf()))?;
        let handler = self.source_file_handlers.get(extension).ok_or(
            AssetTrackingError::HandlerNotFound(extension.to_string()),
        )?;
        if !path.is_file() {
            return Err(
                AssetTrackingError::FileNotFound(path.to_path_buf()).into()
            );
        }
        handler(path, AssetCookContext::new(path))
    }

    pub fn load<T: Asset>(
        &mut self,
        asset_name: &str,
        resources: &mut Resources,
    ) -> Result<AssetHandle<T>, ParsecError> {

        let asset_id = *self
            .names
            .get(asset_name)
            .ok_or_else(|| {
                AssetTrackingError::AssetNotFound(asset_name.to_string())
            })?;

        let source_file = {
            let (desc, slot) = self.assets.get_mut(&asset_id).unwrap();
            if desc.asset_type != T::ASSET_TYPE {
                return Err(AssetTrackingError::TypeMismatch(
                    asset_name.to_string(),
                )
                .into());
            }
            if matches!(slot, AssetSlot::Loaded(_)) {
                return Ok(AssetHandle::new(asset_id));
            }
            *slot = AssetSlot::Loading;
            desc.source_file.clone()
        };

        let cooked_path = cooked_path(asset_name);
        let cooked = if cooked_path.is_file() {
            Ok(())
        } else {
            self.cook_source_file(&source_file)
        }
        .and_then(|()| std::fs::read(&cooked_path).map_err(ParsecError::from))
        .and_then(|bytes| {
            postcard::from_bytes::<T::Cooked>(&bytes).map_err(ParsecError::from)
        });
        let (_, slot) = self.assets.get_mut(&asset_id).unwrap();
        match cooked {
            Ok(cooked) => {
                *slot = AssetSlot::Loaded(Box::new(T::load(cooked, resources)));
                Ok(AssetHandle::new(asset_id))
            },
            Err(err) => {
                *slot = AssetSlot::Failed(
                    AssetTrackingError::LoadFailed(asset_name.to_string())
                        .into(),
                );
                Err(err)
            },
        }
    }

    pub fn get<T: Asset>(&self, handle: AssetHandle<T>) -> Option<&T> {
        match self.assets.get(&handle.asset_id)? {
            (_, AssetSlot::Loaded(asset_any)) => asset_any.downcast_ref::<T>(),
            _ => None,
        }
    }
}
