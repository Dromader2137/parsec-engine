//! Module responsible for handling user input.

use std::sync::Arc;

use keys::Keys;

use crate::{
    ecs::system::{System, SystemBundle, SystemTrigger, system}, graphics::window::WindowWrapper, input::{
        key::{KeyCode, KeyState},
        mouse::Mouse,
    }, math::vec::Vec2f, resources::{Resource, Resources}
};

pub mod key;
pub mod keys;
pub mod mouse;

/// Contains all input data.
#[derive(Debug)]
pub struct Input {
    pub keys: Keys,
    pub mouse: Mouse,
}

impl Input {
    pub fn new() -> Input {
        Input {
            keys: Keys::new(),
            mouse: Mouse::new(),
        }
    }
}

/// A keybord input event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyboardInputEvent {
    key: KeyCode,
    state: KeyState,
}

impl KeyboardInputEvent {
    pub fn new(key: KeyCode, state: KeyState) -> KeyboardInputEvent {
        KeyboardInputEvent { key, state }
    }
}

/// A mouse movement event.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseMovementEvent {
    position: Vec2f,
}

impl MouseMovementEvent {
    pub fn new(position: Vec2f) -> MouseMovementEvent { MouseMovementEvent { position } }
}

#[system]
fn input_start() { Resources::add(Input::new()).unwrap(); }

#[system]
fn input_clear(mut input: Resource<Input>) {
    input.keys.clear();
    input.mouse.clear();
}

#[system]
fn input_clear_all(mut input: Resource<Input>) {
    input.keys.clear_all();
    input.mouse.clear();
}

#[system]
fn input_keyboard_event(mut input: Resource<Input>, input_event: Resource<KeyboardInputEvent>) {
    input.keys.process_input_event(*input_event);
}

#[system]
fn input_mouse_movement(mut input: Resource<Input>, mut window: Resource<Arc<WindowWrapper>>, movement_event: Resource<MouseMovementEvent>) {
    input.mouse.set_position(movement_event.position);
    window.
}

#[derive(Default)]
pub struct InputBundle {}
impl SystemBundle for InputBundle {
    fn systems(self) -> Vec<(SystemTrigger, Box<dyn System>)> {
        vec![
            (SystemTrigger::Start, InputStart::new()),
            (SystemTrigger::LateUpdate, InputClear::new()),
            (SystemTrigger::WindowCursorLeft, InputClearAll::new()),
            (SystemTrigger::KeyboardInput, InputKeyboardEvent::new()),
            (SystemTrigger::MouseMovement, InputMouseMovement::new()),
        ]
    }
}
