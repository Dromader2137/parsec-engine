use std::path::Path;

use image::EncodableLayout;

use crate::assets::{
    core::texture::{Texture, TextureInput},
    library::AssetCookError,
    source_file::{AssetCookContext, AssetSourceFileHandler},
};

pub struct PngFileHandler;

impl AssetSourceFileHandler for PngFileHandler {
    const EXTENSION: &'static str = "png";

    fn cook(asset_source_path: &Path, ctx: AssetCookContext) -> Result<(), AssetCookError> {
        let image =
            image::open(asset_source_path).map_err(|err| AssetCookError::Other(Box::new(err)))?;
        let texture_input = TextureInput {
            width: image.width(),
            height: image.height(),
            pixel_data: image.into_rgba8().as_raw().as_bytes().to_vec(),
        };
        ctx.add_asset::<Texture>("", texture_input)
    }
}
