use std::path::Path;

use crate::{
    assets::{
        core::shader::{InputShader, Shader},
        source_file::{AssetCookContext, AssetSourceFileHandler},
    },
    error::ParsecError,
};

fn cook_shader(
    path: &Path,
    extension: &str,
    ctx: AssetCookContext,
) -> Result<(), ParsecError> {
    let shader_code = std::fs::read_to_string(path)?;
    ctx.add_asset::<Shader>("main", InputShader {
        shader_extension: extension.to_string(),
        shader_code,
    })
}

pub struct VertexShaderFileHandler;

impl AssetSourceFileHandler for VertexShaderFileHandler {
    const EXTENSION: &'static str = "vert";

    fn cook(
        asset_source_path: &Path,
        ctx: AssetCookContext,
    ) -> Result<(), ParsecError> {
        cook_shader(asset_source_path, Self::EXTENSION, ctx)
    }
}

pub struct FragmentShaderFileHandler;

impl AssetSourceFileHandler for FragmentShaderFileHandler {
    const EXTENSION: &'static str = "frag";

    fn cook(
        asset_source_path: &Path,
        ctx: AssetCookContext,
    ) -> Result<(), ParsecError> {
        cook_shader(asset_source_path, Self::EXTENSION, ctx)
    }
}
