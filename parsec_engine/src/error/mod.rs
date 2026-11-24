use std::{error::Error, fmt::{Debug, Display}};

#[derive(Debug)]
pub struct SystemError {
    source: Box<dyn Error + Send + Sync + 'static>,
}

impl Display for SystemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.source)
    }
}

impl<T: Error + Send + Sync + 'static> From<T> for SystemError {
    fn from(value: T) -> Self {
        SystemError { source: Box::new(value) }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("tried to access a None option")]
pub struct NoneErr;

pub trait NoneToErr<T> where Self: Sized {
    fn to_err(self) -> Result<T, NoneErr>;
}

impl<T> NoneToErr<T> for Option<T> {
    fn to_err(self) -> Result<T, NoneErr> {
        self.ok_or(NoneErr)
    }
}
