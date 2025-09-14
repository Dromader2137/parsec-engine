use keys::Keys;

use crate::input::key::{KeyCode, KeyState};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputEvent {
    key: KeyCode,
    state: KeyState
}

impl InputEvent {
    pub fn new(key: KeyCode, state: KeyState) -> InputEvent {
        InputEvent { key, state }
    }
}
