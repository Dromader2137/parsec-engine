//! Things that don't fit anywhere else / are used everywhere.

use crate::utils::borrowing::AccessError;

pub type IdType = u32;

pub mod borrowing;
pub mod id_counter;
pub mod identifiable;
pub mod small_vec;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum UtilsError {
    #[error("failed to access value: {0}")]
    Access(#[from] AccessError),
}
