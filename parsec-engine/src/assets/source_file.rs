//! [AssetSourceFileHandler] takes in files and gets all the information about
//! contained assets. For example an .obj file can contain multiple meshes.

use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use crate::assets::AssetDescription;

pub struct AssetSink<'a> {
    assets: &'a mut Vec<AssetDescription>,
}

pub trait AssetSourceFileHandler {
    const EXTENSION: &'static str;

    fn cook(asset_source_path: &Path, asset_sink: AssetSink);
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct AssetSourceFile {
    pub filepath: PathBuf,
    pub asset_ids: BTreeSet<usize>,
}
