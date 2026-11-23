use std::{backtrace::Backtrace, error::Error, fmt::Debug};

use crate::{
    assets::AssetError, ecs::world::WorldError, input::InputError, renderer::RendererError,
    utils::UtilsError,
};

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ParsecError {
    #[error(transparent)] Asset(#[from] AssetError),
    #[error(transparent)] World(#[from] WorldError),
    #[error(transparent)] Graphics(#[from] GraphicsError),
    #[error(transparent)] Input(#[from] InputError),
    #[error(transparent)] Renderer(#[from] RendererError),
    #[error(transparent)] Utils(#[from] UtilsError),
}

#[derive(Debug)]
enum ParsecErrorKind {
    Fatal,
    Error,
    Warning,
    Info,
}

impl ParsecError {
    fn new(
        kind: ParsecErrorKind,
        source: Option<Box<dyn Error + Send + Sync + 'static>>,
        message: Option<String>,
        backtrace: Option<Backtrace>,
    ) -> Self {
        Self { kind, message, source, backtrace }
    }

    pub fn is_fatal(&self) -> bool { matches!(self.kind, ParsecErrorKind::Fatal) }
}

pub trait ParsecErrorMsg<T>
where
    Self: Sized,
{
    fn msg(self, msg: String) -> Self;
}

impl<T> ParsecErrorMsg<T> for Result<T, ParsecError> {
    fn msg(self, msg: String) -> Self {
        self.map_err(|mut err| {
            err.message = Some(msg);
            return err;
        })
    }
}

pub trait IntoParsecError<T> {
    fn fatal(self) -> Result<T, ParsecError>;
    fn error(self) -> Result<T, ParsecError>;
    fn warn(self) -> Result<T, ParsecError>;
    fn info(self) -> Result<T, ParsecError>;
}

impl<T> IntoParsecError<T> for Result<T, Box<dyn Error + Send + Sync + 'static>> {
    fn fatal(self) -> Result<T, ParsecError> {
        self.map_err(|err| ParsecError::new(ParsecErrorKind::Fatal, None, Some(err), Some(err)))
    }

    fn error(self) -> Result<T, ParsecError> {
        self.map_err(|err| ParsecError::new(ParsecErrorKind::Error, Some(err), None))
    }

    fn warn(self) -> Result<T, ParsecError> {
        self.map_err(|err| ParsecError::new(ParsecErrorKind::Warning, Some(err), None))
    }

    fn info(self) -> Result<T, ParsecError> {
        self.map_err(|err| ParsecError::new(ParsecErrorKind::Info, Some(err), None))
    }
}

impl<T> IntoParsecError<T> for Option<T> {
    fn fatal(self) -> Result<T, ParsecError> {
        self.ok_or(ParsecError::new(ParsecErrorKind::Fatal, None, None))
    }

    fn error(self) -> Result<T, ParsecError> {
        self.ok_or(ParsecError::new(ParsecErrorKind::Error, None, None))
    }

    fn warn(self) -> Result<T, ParsecError> {
        self.ok_or(ParsecError::new(ParsecErrorKind::Warning, None, None))
    }

    fn info(self) -> Result<T, ParsecError> {
        self.ok_or(ParsecError::new(ParsecErrorKind::Info, None, None))
    }
}
