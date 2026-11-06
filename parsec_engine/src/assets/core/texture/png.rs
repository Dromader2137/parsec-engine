use std::path::Path;

use image::EncodableLayout;

use crate::{
    assets::{
        core::texture::{Texture, TextureInput},
        source_file::{AssetCookContext, AssetSourceFileHandler},
    },
    error::ParsecError,
};

pub struct PngFileHandler;

impl AssetSourceFileHandler for PngFileHandler {
    const EXTENSION: &'static str = "png";

    fn cook(asset_source_path: &Path, ctx: AssetCookContext) -> Result<(), ParsecError> {
        let image = image::open(asset_source_path)?;
        let texture_input = TextureInput {
            width: image.width(),
            height: image.height(),
            pixel_data: image.into_rgba8().as_raw().as_bytes().to_vec(),
        };
        ctx.add_asset::<Texture>("", texture_input)
    }
}
