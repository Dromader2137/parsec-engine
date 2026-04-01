use std::fmt::{Display, Debug};

pub struct ParsecError {
    error: Box<dyn std::error::Error + Send + Sync + 'static>,
}

impl Debug for ParsecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParsecError: ")
            .field("error: ", &self.error)
            .finish()
    }
}

impl Display for ParsecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self as &dyn std::fmt::Debug).fmt(f)
    }
}

impl<E> From<E> for ParsecError
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(value: E) -> Self {
        Self {
            error: Box::new(value),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub struct StrError(pub &'static str);

impl Display for StrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

pub trait OptionExt<T> {
    fn none_err(self) -> std::result::Result<T, ParsecError>;
}

impl<T> OptionExt<T> for Option<T> {
    fn none_err(self) -> std::result::Result<T, ParsecError> {
        self.ok_or(StrError("failed to unwrap None").into())
    }
}
