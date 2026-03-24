use std::sync::atomic::{AtomicU32, Ordering};

use crate::utils::IdType;

#[derive(Debug)]
pub struct IdCounter {
    id: AtomicU32,
}

impl IdCounter {
    fn new() -> IdCounter {
        IdCounter {
            id: AtomicU32::new(0),
        }
    }

    pub fn next(&self) -> IdType {
        self.id.fetch_add(1, Ordering::SeqCst);
        self.id.load(Ordering::SeqCst)
    }
}

impl Default for IdCounter {
    fn default() -> Self { IdCounter::new() }
}

#[macro_export]
macro_rules! create_counter {
    ($name:ident) => {
        static $name: once_cell::sync::Lazy<
            crate::utils::id_counter::IdCounter,
        > = once_cell::sync::Lazy::new(|| {
            crate::utils::id_counter::IdCounter::default()
        });
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let id_counter = IdCounter::default();
        assert_eq!(id_counter.next(), 1);
        assert_eq!(id_counter.next(), 2);
        assert_eq!(id_counter.next(), 3);
    }
}
