use std::sync::Mutex;

use crate::utils::IdType;

#[derive(Debug)]
pub struct IdCounter {
    id: Mutex<IdType>,
}

impl IdCounter {
    fn new() -> IdCounter { IdCounter { id: Mutex::new(0) } }

    pub fn next(&self) -> IdType {
        let mut lock = self.id.lock().unwrap();
        let ret = *lock;
        *lock += 1;
        ret
    }
}

impl Default for IdCounter {
    fn default() -> Self {
        IdCounter::new()        
    }
}

#[macro_export]
macro_rules! create_counter {
    ($name:ident) => {
        static $name: once_cell::sync::Lazy<crate::utils::id_counter::IdCounter> = 
            once_cell::sync::Lazy::new(|| crate::utils::id_counter::IdCounter::default());
    };
}
