use std::{
    any::Any,
    collections::HashMap,
    marker::PhantomData,
    ops::Deref,
    path::Path,
    sync::{Arc, RwLock},
    time::SystemTime,
};

use crate::{
    assets::{
        Asset, AssetId,
        manifest::{AssetDescription, cooked_path},
        source_file::{AssetCookContext, AssetSourceFile, AssetSourceFileHandler, AssetSourceId},
    },
    ecs::resources::Resources,
    error::ParsecError,
};

#[derive(Debug, Clone)]
pub struct AssetHandle<T: Asset> {
    asset_slot: Arc<RwLock<AssetSlot>>,
    _marker: PhantomData<T>,
}

impl<T: Asset> Deref for AssetHandle<T> {
    type Target = Option<&T>;
}

impl<T: Asset> AssetHandle<T> {
    pub(crate) fn new(asset_slot: Arc<RwLock<AssetSlot>>) -> Self {
        Self { asset_slot, _marker: PhantomData }
    }

    fn get(&self, resources: &mut Resources) -> Option<&T> {
        let lock = self.asset_slot.read().expect("lock shouldn't be poisoned here");
        if let AssetSlot::Loaded(asset) = &*lock { asset.downcast_ref::<T>() } else { None }
    }
}

#[derive(Debug, Default)]
enum AssetSlot {
    #[default]
    Unloaded,
    Loading,
    Loaded(Box<dyn Any>),
    Failed(AssetError),
}

#[derive(Debug, thiserror::Error)]
pub enum AssetError {
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
    assets: HashMap<AssetId, (AssetDescription, Arc<RwLock<AssetSlot>>)>,
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

    pub fn add_asset(&mut self, asset_name: &'static str) -> Result<(), AssetError> {
        if self.names.contains_key(asset_name) {
            return Err(AssetError::DuplicateName(asset_name));
        }
        let description = AssetDescription::from_name(asset_name)
            .ok_or(AssetError::InvalidAssetName(asset_name))?;
        let id = AssetId(self.assets.len() as u64);
        let slot = AssetSlot::Unloaded;
        self.assets.insert(id, (description, Arc::new(RwLock::new(slot))));
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

    fn cook_source_file(&self, path: &'static Path) -> Result<(), ParsecError> {
        let extension =
            path.extension().and_then(|x| x.to_str()).ok_or(AssetError::InvalidExtension(path))?;
        let handler = self
            .source_file_handlers
            .get(extension)
            .ok_or(AssetError::HandlerNotFound(extension))?;
        if !path.is_file() {
            return Err(AssetError::FileNotFound(path).into());
        }
        handler(path, AssetCookContext::new(path))
    }

    pub fn load<T: Asset>(
        &self,
        asset_name: &'static str,
        resources: &mut Resources,
    ) -> Result<AssetHandle<T>, ParsecError> {
        let asset_id =
            *self.names.get(asset_name).ok_or_else(|| AssetError::AssetNotFound(asset_name))?;

        let source_file = {
            let (desc, slot) = self.assets.get_mut(&asset_id).unwrap();
            if desc.asset_type != T::ASSET_TYPE {
                return Err(AssetError::TypeMismatch(asset_name).into());
            }
            desc.source_file
        };

        let source_file_meta = std::fs::metadata(source_file)?;
        let source_modified = source_file_meta.modified()?;
        let source_modified_duration =
            source_modified.duration_since(SystemTime::UNIX_EPOCH)?.as_millis();

        let cooked_path = cooked_path(asset_name);
        let cooked =
            if cooked_path.is_file() { Ok(()) } else { self.cook_source_file(&source_file) }
                .and_then(|()| std::fs::read(&cooked_path).map_err(ParsecError::from))
                .and_then(|bytes| {
                    postcard::from_bytes::<(u128, T::Cooked)>(&bytes).map_err(ParsecError::from)
                });

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
                let asset = Box::new(T::load(cooked, resources));
                let lock = slot.write().unwrap();
                
                *slot = AssetSlot::Loaded(asset);
                if let AssetSlot::Loaded(asset) = slot {
                    return Ok((
                        asset.downcast_ref::<T>().expect("types should match at this point"),
                        AssetHandle::new(asset_id),
                    ));
                }
                Err(AssetError::LoadFailed("internal error").into())
            },
            Err(err) => {
                let (_, slot) = self.assets.get_mut(&asset_id).unwrap();
                *slot = AssetSlot::Failed(AssetError::LoadFailed(asset_name));
                Err(err)
            },
        }
    }

    pub fn get<T: Asset>(&self, handle: AssetHandle<T>) -> Option<&T> {
        match self.assets.get(&handle.asset_id)? {
            (_, AssetSlot::Loaded(asset_any)) => asset_any.downcast_ref::<T>(),
            (_, AssetSlot::Failed(err)) => {
                println!("ERROR: {}", err);
                None
            },
            _ => None,
        }
    }

    pub fn cook_all(&self) -> Result<(), ParsecError> {
        for (_, source_file) in self.source_files() {
            self.cook_source_file(&source_file.filepath)?;
        }
        Ok(())
    }
}
