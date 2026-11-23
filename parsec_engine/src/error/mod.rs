use std::fmt::Debug;

use crate::{
    app::AppError,
    assets::AssetError, ecs::EcsError, graphics::GraphicsError, input::InputError,
    renderer::RendererError, time::TimeError, utils::UtilsError,
};

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ParsecError {
    #[error(transparent)] Asset(#[from] AssetError),
    #[error(transparent)] Ecs(#[from] EcsError),
    #[error(transparent)] Graphics(#[from] GraphicsError),
    #[error(transparent)] Input(#[from] InputError),
    #[error(transparent)] Renderer(#[from] RendererError),
    #[error(transparent)] Utils(#[from] UtilsError),
    #[error(transparent)] Time(#[from] TimeError),
    #[error(transparent)] App(#[from] AppError),
}
