use std::{any::Any, collections::HashMap, marker::PhantomData, path::Path, time::SystemTime};

use crate::{
    assets::{
        Asset, AssetId,
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
    Loading,
    Loaded(Box<dyn Any>),
    #[allow(unused)]
    Failed(AssetLibraryError),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AssetLoadError {
    #[error("asset '{0}' not found")]
    NotFound(&'static str),
    #[error("asset '{0}' is of a different type than requested")]
    TypeMismatch(&'static str),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Time(#[from] std::time::SystemTimeError),
    #[error(transparent)]
    Parse(#[from] postcard::Error),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AssetLibraryError {
    #[error("asset source file {0:?} has no valid extension")]
    InvalidExtension(&'static Path),
    #[error("no asset source file handler registered for extension {0:?}")]
    HandlerNotFound(&'static str),
    #[error("asset source file {0:?} doesn't exist")]
    FileNotFound(&'static Path),
    #[error("asset name {0:?} is already used by a different asset")]
    DuplicateName(&'static str),
    #[error("asset library doesn't contain asset {0:?}")]
    AssetNotFound(&'static str),
    #[error("asset {0:?} has a different type than requested")]
    TypeMismatch(&'static str),
    #[error("failed to load asset {0:?}")]
    LoadFailed(&'static str),
    #[error("invalid asset name {0:?}")]
    InvalidAssetName(&'static str),
}

type SourceFileHandlerFn = fn(&Path, AssetCookContext) -> Result<(), ParsecError>;

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

    pub fn add_asset(&mut self, asset_name: &'static str) -> Result<(), AssetLibraryError> {
        if self.names.contains_key(asset_name) {
            return Err(AssetLibraryError::DuplicateName(asset_name));
        }
        let description = AssetDescription::from_name(asset_name)
            .ok_or(AssetLibraryError::InvalidAssetName(asset_name))?;
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

    fn cook_source_file<T: Asset>(&self, path: &'static Path) -> Result<(u128, T), AssetCookError> {
        let extension = path
            .extension()
            .and_then(|x| x.to_str())
            .ok_or(AssetLibraryError::InvalidExtension(path))?;
        let handler = self
            .source_file_handlers
            .get(extension)
            .ok_or(AssetLibraryError::HandlerNotFound(extension))?;
        if !path.is_file() {
            return Err(AssetLibraryError::FileNotFound(path).into());
        }
        handler(path, AssetCookContext::new(path))?;
        let bytes = std::fs::read(path)?;
        let deserialized = postcard::from_bytes(&bytes)?;
        Ok(deserialized)
    }

    pub fn load<T: Asset>(
        &mut self,
        asset_name: &'static str,
        resources: &mut Resources,
    ) -> Result<AssetHandle<T>, AssetLoadError> {
        use AssetLoadError::*;

        let asset_id = self.asset_by_name(asset_name).ok_or(NotFound(asset_name))?;

        let (desc, slot) = self.assets.get_mut(&asset_id).ok_or(NotFound(asset_name))?;

        if desc.asset_type != T::ASSET_TYPE {
            return Err(TypeMismatch(asset_name));
        }

        if matches!(slot, AssetSlot::Loaded(_)) {
            return Ok(AssetHandle::new(asset_id));
        }

        *slot = AssetSlot::Loading;

        let source_file_meta = std::fs::metadata(desc.source_file)?;
        let source_modified = source_file_meta.modified()?;
        let source_modified_duration =
            source_modified.duration_since(SystemTime::UNIX_EPOCH)?.as_millis();

        let cooked_path = cooked_path(asset_name);
        let (cook_timestamp, cooked) = 
            if cooked_path.is_file() { Ok() } else { self.cook_source_file(&desc.source_file) }

        match cooked {
            Ok((cook_timestamp, mut cooked)) => {
                if source_modified_duration != cook_timestamp {
                    let (_, new_cooked) = self
                        .cook_source_file(&source_file)
                        .and_then(|()| std::fs::read(&cooked_path).map_err(ParsecError::from))
                        .and_then(|bytes| {
                            postcard::from_bytes::<(u128, T::Cooked)>(&bytes)
                                .map_err(ParsecError::from)
                        })?;
                    cooked = new_cooked;
                }
                let (_, slot) = self.assets.get_mut(&asset_id).unwrap();
                *slot = AssetSlot::Loaded(Box::new(T::load(cooked, resources)));
                Ok(AssetHandle::new(asset_id))
            },
            Err(err) => {
                let (_, slot) = self.assets.get_mut(&asset_id).unwrap();
                *slot = AssetSlot::Failed(AssetLibraryError::LoadFailed(asset_name));
                Err(err)
            },
        }
    }

    pub fn cook_all(&self) -> Result<(), ParsecError> {
        for (_, source_file) in self.source_files() {
            self.cook_source_file(&source_file.filepath)?;
        }
        Ok(())
    }
}
