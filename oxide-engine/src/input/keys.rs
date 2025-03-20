use std::collections::HashSet;

use super::key::KeyCode;

#[derive(Debug)]
pub struct Keys {
    pressed: HashSet<KeyCode>,
    down: HashSet<KeyCode>,
    up: HashSet<KeyCode>,
}

impl Keys {
    pub fn new() -> Keys {
        Keys {
            pressed: HashSet::new(),
            down: HashSet::new(),
            up: HashSet::new(),
        }
    }

    pub fn press(&mut self, key: KeyCode) {
        if !self.down.contains(&key) {
            self.pressed.insert(key);
        }
        self.down.insert(key);
    }

    pub fn lift(&mut self, key: KeyCode) {
        self.down.remove(&key);
        self.up.insert(key);
    }

    pub fn clear(&mut self) {
        self.pressed.clear();
        self.up.clear();
    }

    pub fn clear_all(&mut self) {
        self.pressed.clear();
        self.down.clear();
        self.up.clear();
    }

    pub fn is_pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key)
    }

    pub fn is_down(&self, key: KeyCode) -> bool {
        self.down.contains(&key)
    }

    pub fn is_up(&self, key: KeyCode) -> bool {
        self.up.contains(&key)
    }
}
