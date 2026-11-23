use std::sync::{Mutex, PoisonError};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Access {
    #[default]
    ReadWrite,
    Read,
    None,
}

#[derive(Debug, Default)]
pub struct BorrowingStats {
    access: Mutex<Access>,
    borrow_count: Mutex<usize>,
}

impl BorrowingStats {
    pub fn borrow(&self) -> Result<(), AccessError> {
        let mut access = self.access.lock().unwrap_or_else(PoisonError::into_inner);
        let mut count = self.borrow_count.lock().unwrap_or_else(PoisonError::into_inner);
        match *access {
            Access::ReadWrite => {
                *access = Access::Read;
                *count = 1;
                Ok(())
            },
            Access::Read => {
                *access = Access::Read;
                *count += 1;
                Ok(())
            },
            Access::None => Err(AccessError::AlreadyBorrowedMut),
        }
    }

    pub fn borrow_mut(&self) -> Result<(), AccessError> {
        let mut access = self.access.lock().unwrap_or_else(PoisonError::into_inner);
        match *access {
            Access::ReadWrite => {
                *access = Access::None;
                Ok(())
            },
            Access::Read => Err(AccessError::AlreadyBorrowed),
            Access::None => Err(AccessError::AlreadyBorrowedMut),
        }
    }

    pub fn free(&self) {
        let mut access = self.access.lock().unwrap_or_else(PoisonError::into_inner);
        let mut count = self.borrow_count.lock().unwrap_or_else(PoisonError::into_inner);
        match *access {
            Access::Read => {
                *count -= 1;
                if *count == 0 {
                    *access = Access::ReadWrite;
                }
            },
            Access::None => *access = Access::ReadWrite,
            _ => (),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum AccessError {
    #[error("already borrowed immutably")]
    AlreadyBorrowed,
    #[error("already borrowed mutably")]
    AlreadyBorrowedMut,
}
