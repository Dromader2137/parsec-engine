//! Module responsible for mouse interaction.

pub type MouseButton = winit::event::MouseButton;
pub type MouseButtonState = winit::event::ElementState;

use std::collections::HashSet;

use crate::math::vec::Vec2f;

/// A mouse movement event.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseMovementEvent {
    position: Vec2f,
}

impl MouseMovementEvent {
    pub fn new(position: Vec2f) -> MouseMovementEvent {
        MouseMovementEvent { position }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseButtonEvent {
    button: MouseButton,
    state: MouseButtonState
}

impl MouseButtonEvent {
    pub fn new(button: MouseButton, state: MouseButtonState) -> MouseButtonEvent {
        MouseButtonEvent { button, state }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseWheelEvent {
    delta: Vec2f
}

impl MouseWheelEvent {
    pub fn new(delta: Vec2f) -> MouseWheelEvent {
        MouseWheelEvent { delta }
    }
}

/// Stores mouse information.
#[derive(Debug)]
pub struct Mouse {
    position: Vec2f,
    prev_position: Vec2f,
    position_delta: Vec2f,
    wheel_delta: Vec2f,
    pressed: HashSet<MouseButton>,
    down: HashSet<MouseButton>,
    up: HashSet<MouseButton>,
}

impl Mouse {
    pub fn new() -> Mouse {
        Mouse {
            position: Vec2f::ZERO,
            prev_position: Vec2f::ZERO,
            position_delta: Vec2f::ZERO,
            wheel_delta: Vec2f::ZERO,
            pressed: HashSet::new(),
            down: HashSet::new(),
            up: HashSet::new(),
        }
    }

    pub fn positon_delta(&self) -> Vec2f { self.position_delta }
    
    pub fn wheel_delta(&self) -> Vec2f { self.wheel_delta }

    pub fn position(&self) -> Vec2f { self.position }

    fn set_position(&mut self, new_position: Vec2f) {
        self.prev_position = self.position;
        self.position = new_position;
        self.position_delta = self.position - self.prev_position;
    }

    pub fn clear(&mut self) { 
        self.position_delta = Vec2f::ZERO; 
        self.wheel_delta = Vec2f::ZERO; 
        self.pressed.clear();
        self.up.clear();
    }
    
    /// Clears all buttons state.
    pub fn clear_all(&mut self) {
        self.position_delta = Vec2f::ZERO; 
        self.wheel_delta = Vec2f::ZERO; 
        self.pressed.clear();
        self.down.clear();
        self.up.clear();
    }

    pub fn process_movement(&mut self, event: MouseMovementEvent) {
        self.set_position(event.position);
    }
    
    /// Takes an [InputEvent] and updated `self` accordingly.
    pub fn process_button_event(&mut self, event: MouseButtonEvent) {
        match event.state {
            MouseButtonState::Pressed => self.press(event.button),
            MouseButtonState::Released => self.lift(event.button),
        }
    }
    
    pub fn process_wheel_event(&mut self, event: MouseWheelEvent) {
        self.wheel_delta = event.delta;
    }   

    fn press(&mut self, button: MouseButton) {
        if !self.down.contains(&button) {
            self.pressed.insert(button.clone());
        }
        self.down.insert(button);
    }

    fn lift(&mut self, button: MouseButton) {
        self.down.remove(&button);
        self.up.insert(button);
    }

    /// Checks if the `button` is pressed.
    pub fn is_pressed(&self, button: MouseButton) -> bool {
        self.pressed.contains(&button)
    }

    /// Checks if the `button` is down.
    pub fn is_down(&self, button: MouseButton) -> bool {
        self.down.contains(&button)
    }

    /// Checks if the `button` is up.
    pub fn is_up(&self, button: MouseButton) -> bool {
        self.up.contains(&button)
    }
}
