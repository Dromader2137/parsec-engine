use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy)]
pub struct AssetDescription {
    pub subresource: &'static str,
    pub source_file: &'static Path,
    pub asset_type: &'static str,
}

impl AssetDescription {
    pub fn new(
        subresource: &'static str,
        source_file: &'static Path,
        asset_type: &'static str,
    ) -> Self {
        let source_file = Path::new(source_file);
        Self {
            subresource,
            source_file,
            asset_type,
        }
    }

    pub fn from_name(asset_name: &'static str) -> Option<Self> {
        extract_from_asset_name(asset_name)
    }

    pub fn into_name(self) -> String {
        combine_into_asset_name(
            self.subresource,
            self.source_file,
            self.asset_type,
        )
    }
}

pub(super) fn combine_into_asset_name(
    subresource: &str,
    source_file: &Path,
    asset_type: &str,
) -> String {
    format!(
        "{}@{}#{}",
        subresource,
        source_file.to_string_lossy(),
        asset_type
    )
}

fn extract_from_asset_name(
    asset_name: &'static str,
) -> Option<AssetDescription> {
    let (subresource, other) = asset_name.split_once("@")?;
    let (filename, asset_type) = other.split_once("#")?;
    let source_file = Path::new(filename);
    Some(AssetDescription::new(subresource, source_file, asset_type))
}

fn six_bytes_to_char(sb: u8) -> char {
    match sb {
        0..=25 => ('A' as u8 + sb) as char,
        26..=51 => ('a' as u8 + (sb - 26)) as char,
        52..=61 => ('0' as u8 + (sb - 52)) as char,
        62 => '-',
        63 => '_',
        _ => unreachable!(),
    }
}

pub(super) fn cooked_path(name: &str) -> PathBuf {
    let bytes = name.as_bytes();
    let base64 = bytes
        .chunks(3)
        .map(|chunk| {
            let a = chunk.get(0).map(|x| *x).unwrap_or_default();
            let b = chunk.get(1).map(|x| *x).unwrap_or_default();
            let c = chunk.get(2).map(|x| *x).unwrap_or_default();
            let b1 = six_bytes_to_char((a & 0xFC) >> 2);
            let b2 = six_bytes_to_char(((a & 0x03) << 4) | ((b & 0xF0) >> 4));
            if chunk.get(1).is_none() {
                return [b1, b2, '=', '='];
            }
            let b3 = six_bytes_to_char(((b & 0x0F) << 2) | ((c & 0xC0) >> 6));
            if chunk.get(2).is_none() {
                return [b1, b2, b3, '='];
            }
            let b4 = six_bytes_to_char(c & 0x3F);
            [b1, b2, b3, b4]
        })
        .flatten()
        .collect::<String>();
    Path::new("assets").join(format!("{base64}.asset"))
}
