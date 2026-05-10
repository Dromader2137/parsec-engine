use std::fmt::{Debug, Display};

pub struct ParsecError {
    error: Box<dyn std::error::Error + Send + Sync + 'static>,
}

impl Debug for ParsecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}", self.error))
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

pub trait OptionNoneErr<T> {
    fn none_err(self) -> Result<T, ParsecError>;
}

impl<T> OptionNoneErr<T> for Option<T> {
    fn none_err(self) -> Result<T, ParsecError> {
        self.ok_or(StrError("failed to unwrap None").into())
    }
}
