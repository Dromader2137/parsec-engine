use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct AssetDescription {
    pub subresource: String,
    pub source_file: PathBuf,
    pub asset_type: &'static str,
}

impl AssetDescription {
    pub fn new(subresource: &str, source_file: impl AsRef<Path>, asset_type: &'static str) -> Self {
        let source_file = source_file.as_ref().to_path_buf();
        Self { subresource: subresource.to_string(), source_file, asset_type }
    }

    pub fn name(&self) -> String {
        asset_name(&self.subresource, &self.source_file, self.asset_type)
    }
}

pub(super) fn asset_name(
    subresource: &str,
    source_file: &Path,
    asset_type: &str,
) -> String {
    let filename = source_file.file_name().unwrap_or_default().to_string_lossy();
    format!("{subresource}@{filename}.{asset_type}")
}

pub(super) fn cooked_path(name: &str) -> PathBuf {
    Path::new("assets").join(format!("{name}.asset"))
}
