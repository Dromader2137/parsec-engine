use std::{any::Any, collections::HashMap, marker::PhantomData, path::Path, time::SystemTime};

use crate::{
    assets::{
        Asset, AssetError, AssetId,
        manifest::{AssetDescription, cooked_path},
        source_file::{AssetCookContext, AssetSourceFile, AssetSourceFileHandler, AssetSourceId},
    },
    ecs::resources::Resources,
    error::ParsecError,
};

#[derive(Debug, PartialEq, Eq)]
pub struct AssetHandle<T: Asset> {
    asset_id: AssetId,
    _marker: PhantomData<T>,
}

impl<T: Asset> Clone for AssetHandle<T> {
    fn clone(&self) -> Self { Self { asset_id: self.asset_id, _marker: PhantomData } }
}
impl<T: Asset> Copy for AssetHandle<T> {}
impl<T: Asset> AssetHandle<T> {
    pub(crate) fn new(asset_id: AssetId) -> Self { Self { asset_id, _marker: PhantomData } }

    pub fn resolve<'lib>(&self, library: &'lib AssetLibrary) -> Option<&'lib T> {
        match library.assets.get(&self.asset_id)? {
            (_, AssetSlot::Loaded(asset_any)) => asset_any.downcast_ref::<T>(),
            _ => None,
        }
    }
}

#[derive(Debug, Default)]
enum AssetSlot {
    #[default]
    Unloaded,
    Loaded(Box<dyn Any>),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AssetLoadError {
    #[error("asset '{0}' not found")]
    NotFound(&'static str),
    #[error("cooked asset file for '{0}' not found")]
    CookedFileNotFound(&'static str),
    #[error("asset '{0}' is of a different type than requested")]
    TypeMismatch(&'static str),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Time(#[from] std::time::SystemTimeError),
    #[error(transparent)]
    Parse(#[from] postcard::Error),
    #[error(transparent)]
    AssetInternal(Box<dyn std::error::Error + Send + Sync + 'static>),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AssetCookError {
    #[error("source file '{0}' has no valid extension")]
    InvalidExtension(&'static Path),
    #[error("source file handler '{0}' not found")]
    HandlerNotFound(&'static str),
    #[error("source file '{0}' not found")]
    FileNotFound(&'static Path),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Time(#[from] std::time::SystemTimeError),
    #[error(transparent)]
    Parse(#[from] postcard::Error),
    #[error(transparent)]
    AssetInternal(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error(transparent)]
    HandlerInternal(Box<dyn std::error::Error + Send + Sync + 'static>),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AssetAddError {
    #[error("duplicated name '{0}'")]
    Duplicate(&'static str),
    #[error("invalid name '{0}'")]
    InvalidName(&'static str),
}

type SourceFileHandlerFn = fn(&Path, AssetCookContext) -> Result<(), AssetCookError>;

#[derive(Debug)]
pub struct AssetLibrary {
    assets: HashMap<AssetId, (AssetDescription, AssetSlot)>,
    names: HashMap<&'static str, AssetId>,
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

    pub fn add_asset(&mut self, asset_name: &'static str) -> Result<(), AssetAddError> {
        if self.names.contains_key(asset_name) {
            Err(AssetAddError::Duplicate(asset_name))?;
        }
        let description = AssetDescription::from_name(asset_name)
            .ok_or(AssetAddError::InvalidName(asset_name))?;
        let id = AssetId(self.assets.len() as u64);
        let slot = AssetSlot::Unloaded;
        self.assets.insert(id, (description, slot));
        self.names.insert(asset_name, id);
        Ok(())
    }

    pub fn add_source_file_handler<H: AssetSourceFileHandler>(&mut self) {
        self.source_file_handlers.insert(H::EXTENSION, H::cook);
    }

    pub fn source_file_by_path(&self, path: &Path) -> Option<AssetSourceId> {
        self.files.iter().find(|(_, file)| file.filepath == path).map(|(&id, _)| id)
    }

    pub fn source_file(&self, source_id: AssetSourceId) -> Option<&AssetSourceFile> {
        self.files.get(&source_id)
    }

    pub fn source_files(&self) -> impl Iterator<Item = (AssetSourceId, &AssetSourceFile)> {
        self.files.iter().map(|(&id, file)| (id, file))
    }

    pub fn asset_description(&self, asset_id: AssetId) -> Option<&AssetDescription> {
        self.assets.get(&asset_id).map(|(desc, _)| desc)
    }

    pub fn asset_descriptions(&self) -> impl Iterator<Item = (AssetId, &AssetDescription)> {
        self.assets.iter().map(|(&id, (desc, _))| (id, desc))
    }

    fn asset_by_name(&self, asset_name: &'static str) -> Option<AssetId> {
        self.names.get(asset_name).cloned()
    }

    fn cook_source_file(&self, path: &'static Path) -> Result<(), AssetCookError> {
        let extension = path
            .extension()
            .and_then(|x| x.to_str())
            .ok_or(AssetCookError::InvalidExtension(path))?;
        let handler = self
            .source_file_handlers
            .get(extension)
            .ok_or(AssetCookError::HandlerNotFound(extension))?;
        if !path.is_file() {
            Err(AssetCookError::FileNotFound(path))?;
        }
        handler(path, AssetCookContext::new(path))
    }

    fn read_cooked_asset<T: Asset>(
        &self,
        asset_name: &'static str,
    ) -> Result<(u128, T::Cooked), AssetLoadError> {
        let cooked_path = cooked_path(asset_name);
        if !cooked_path.is_file() {
            Err(AssetLoadError::CookedFileNotFound(asset_name))?
        }
        let bytes = std::fs::read(&cooked_path)?;
        let deserialized = postcard::from_bytes(&bytes)?;
        Ok(deserialized)
    }

    fn check_last_modification(&self, source_file: &'static Path) -> Result<u128, AssetLoadError> {
        let source_file_meta = std::fs::metadata(source_file)?;
        let source_modified = source_file_meta.modified()?;
        let source_modified_duration =
            source_modified.duration_since(SystemTime::UNIX_EPOCH)?.as_millis();
        Ok(source_modified_duration)
    }

    pub fn load<T: Asset>(
        &mut self,
        asset_name: &'static str,
        resources: &mut Resources,
    ) -> Result<AssetHandle<T>, AssetError> {
        let asset_id =
            self.asset_by_name(asset_name).ok_or(AssetLoadError::NotFound(asset_name))?;

        let (desc, slot) =
            self.assets.get_mut(&asset_id).ok_or(AssetLoadError::NotFound(asset_name))?;
        let source_file = desc.source_file;

        if desc.asset_type != T::ASSET_TYPE {
            Err(AssetLoadError::TypeMismatch(asset_name))?;
        }

        if matches!(slot, AssetSlot::Loaded(_)) {
            return Ok(AssetHandle::new(asset_id));
        }

        let last_modification = self.check_last_modification(source_file)?;
        let (_, cooked_asset) = match self.read_cooked_asset::<T>(asset_name) {
            Err(AssetLoadError::CookedFileNotFound(_)) => {
                self.cook_source_file(source_file)?;
                self.read_cooked_asset::<T>(asset_name)
            },
            Err(err) => Err(err),
            Ok((ts, val)) => {
                if ts == last_modification {
                    Ok((ts, val))
                } else {
                    self.cook_source_file(source_file)?;
                    self.read_cooked_asset::<T>(asset_name)
                }
            },
        }?;

        let (_, slot) =
            self.assets.get_mut(&asset_id).ok_or(AssetLoadError::NotFound(asset_name))?;
        *slot = AssetSlot::Loaded(Box::new(T::load(cooked_asset, resources)?));
        Ok(AssetHandle::new(asset_id))
    }

    pub fn cook_all(&self) -> Result<(), ParsecError> {
        for (_, source_file) in self.source_files() {
            self.cook_source_file(&source_file.filepath)?;
        }
        Ok(())
    }
}
