use std::sync::Mutex;

#[derive(Debug)]
pub struct IdCounter {
    id: Mutex<u32>,
}

impl IdCounter {
    pub fn new(id: u32) -> IdCounter { IdCounter { id: Mutex::new(id) } }

    pub fn next(&self) -> u32 {
        let mut lock = self.id.lock().unwrap();
        let ret = *lock;
        *lock += 1;
        ret
    }
}
