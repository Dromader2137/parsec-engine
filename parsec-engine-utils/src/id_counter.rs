use std::sync::atomic::{AtomicU32, Ordering};

use crate::IdType;

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
        static $name: ::std::sync::LazyLock<
            ::parsec_engine_utils::id_counter::IdCounter,
        > = ::std::sync::LazyLock::new(|| {
            ::parsec_engine_utils::id_counter::IdCounter::default()
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
