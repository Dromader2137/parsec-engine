//! Module responsible for mouse interaction.

use crate::math::vec::Vec2f;

/// Stores mouse information.
#[derive(Debug)]
pub struct Mouse {
    position: Vec2f,
    prev_position: Vec2f,
    delta: Vec2f
}

impl Mouse {
    pub fn new() -> Mouse {
        Mouse {
            position: Vec2f::ZERO,
            prev_position: Vec2f::ZERO,
            delta: Vec2f::ZERO
        }
    }

    pub fn delta(&self) -> Vec2f {
        self.delta
    }

    pub fn position(&self) -> Vec2f {
        self.position
    }

    pub fn set_position(&mut self, new_position: Vec2f) {
        self.prev_position = self.position;
        self.position = new_position;
        self.delta = self.position - self.prev_position;
    }

    pub fn clear(&mut self) {
        self.delta = Vec2f::ZERO;
    }
}
