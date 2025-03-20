use keys::Keys;

pub mod key;
pub mod keys;

#[derive(Debug)]
pub struct Input {
    pub keys: Keys,
}

impl Input {
    pub fn new() -> Input {
        Input { keys: Keys::new() }
    }
}
