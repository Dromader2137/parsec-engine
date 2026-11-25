use std::{collections::BTreeSet, path::Path, time::SystemTime};

use crate::assets::{
    Asset, AssetId,
    library::AssetCookError,
    description::{combine_into_asset_name, cooked_path},
};

pub struct AssetCookContext<'a> {
    source_path: &'a Path,
}

impl<'a> AssetCookContext<'a> {
    pub(super) fn new(source_path: &'a Path) -> Self { Self { source_path } }

    pub fn add_asset<T: Asset>(
        &self,
        subresource: &str,
        input: T::Input,
    ) -> Result<(), AssetCookError> {
        let path =
            cooked_path(&combine_into_asset_name(subresource, self.source_path, T::ASSET_TYPE));
        let source_file = std::fs::metadata(self.source_path)?;
        let source_modified = source_file.modified()?;
        let source_modified_duration =
            source_modified.duration_since(SystemTime::UNIX_EPOCH)?.as_millis();
        let bytes = postcard::to_stdvec(&(
            source_modified_duration,
            T::cook(input).map_err(|err| AssetCookError::Other(Box::new(err)))?,
        ))?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(path, bytes)?;
        Ok(())
    }
}

pub trait AssetSourceFileHandler {
    const EXTENSION: &'static str;

    fn cook(asset_source_path: &Path, ctx: AssetCookContext) -> Result<(), AssetCookError>;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct AssetSourceId(pub(super) u64);

#[derive(Debug)]
pub(super) struct AssetSourceFile {
    pub(super) filepath: &'static Path,
    #[allow(unused)]
    pub(super) asset_ids: BTreeSet<AssetId>,
}
