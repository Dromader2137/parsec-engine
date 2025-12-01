//! Key storage.

use std::collections::HashSet;

use crate::input::{
    key::{KeyCode, KeyState, StorageKeyCode}, KeyboardInputEvent
};

/// Stores keys that currently are in different states.
#[derive(Debug)]
pub struct Keys {
    pressed: HashSet<StorageKeyCode>,
    down: HashSet<StorageKeyCode>,
    up: HashSet<StorageKeyCode>,
}

impl Keys {
    pub fn new() -> Keys {
        Keys {
            pressed: HashSet::new(),
            down: HashSet::new(),
            up: HashSet::new(),
        }
    }

    /// Takes an [InputEvent] and updated `self` accordingly.
    pub fn process_input_event(&mut self, event: KeyboardInputEvent) {
        match event.state {
            KeyState::Pressed => self.press(event.key),
            KeyState::Released => self.lift(event.key),
        }
    }

    fn press(&mut self, key: StorageKeyCode) {
        if !self.down.contains(&key) {
            self.pressed.insert(key.clone());
        }
        self.down.insert(key);
    }

    fn lift(&mut self, key: StorageKeyCode) {
        self.down.remove(&key);
        self.up.insert(key);
    }

    /// Clears pressed and up state.
    pub fn clear(&mut self) {
        self.pressed.clear();
        self.up.clear();
    }

    /// Clears all keys state.
    pub fn clear_all(&mut self) {
        self.pressed.clear();
        self.down.clear();
        self.up.clear();
    }

    /// Checks if the `key` is pressed.
    pub fn is_pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key.into())
    }

    /// Checks if the `key` is down.
    pub fn is_down(&self, key: KeyCode) -> bool { self.down.contains(&key.into()) }

    /// Checks if the `key` is up.
    pub fn is_up(&self, key: KeyCode) -> bool { self.up.contains(&key.into()) }
}
