use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use crate::{
    assets::{
        Asset, AssetId,
        manifest::{asset_name, cooked_path},
    },
    error::ParsecError,
};

pub struct AssetCookContext<'a> {
    source_file: &'a Path,
}

impl<'a> AssetCookContext<'a> {
    pub(super) fn new(source_file: &'a Path) -> Self { Self { source_file } }

    pub fn add_asset<T: Asset>(
        &self,
        subresource: &str,
        input: T::Input,
    ) -> Result<(), ParsecError> {
        let path = cooked_path(&asset_name(
            subresource,
            self.source_file,
            T::ASSET_TYPE,
        ));
        let bytes = postcard::to_stdvec(&T::cook(input))?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(path, bytes)?;
        Ok(())
    }
}

pub trait AssetSourceFileHandler {
    const EXTENSION: &'static str;

    fn cook(
        asset_source_path: &Path,
        ctx: AssetCookContext,
    ) -> Result<(), ParsecError>;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssetSourceId(pub(super) u64);

#[derive(Debug)]
pub struct AssetSourceFile {
    pub filepath: PathBuf,
    pub asset_ids: BTreeSet<AssetId>,
}
