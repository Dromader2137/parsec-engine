//! Module responsible for handling user input.

use keys::Keys;

use crate::{
    ecs::system::{System, SystemBundle, SystemTrigger, system},
    input::key::{KeyCode, KeyState},
    resources::{Resource, Resources},
};

pub mod key;
pub mod keys;

/// Contains all input data.
#[derive(Debug)]
pub struct Input {
    pub keys: Keys,
}

impl Input {
    pub fn new() -> Input { Input { keys: Keys::new() } }
}

/// A keybord input event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputEvent {
    key: KeyCode,
    state: KeyState,
}

impl InputEvent {
    pub fn new(key: KeyCode, state: KeyState) -> InputEvent { InputEvent { key, state } }
}

#[system]
fn input_start() { Resources::add(Input::new()).unwrap(); }

#[system]
fn input_clear(mut input: Resource<Input>) { input.keys.clear(); }

#[system]
fn input_clear_all(mut input: Resource<Input>) { input.keys.clear_all(); }

#[system]
fn input_keyboard_event(mut input: Resource<Input>, input_event: Resource<InputEvent>) {
    input.keys.process_input_event(*input_event);
}

#[derive(Default)]
pub struct InputBundle {}
impl SystemBundle for InputBundle {
    fn systems(self) -> Vec<(SystemTrigger, Box<dyn System>)> {
        vec![
            (SystemTrigger::Start, InputStart::new()),
            (SystemTrigger::Render, InputClear::new()),
            (SystemTrigger::WindowCursorLeft, InputClearAll::new()),
            (SystemTrigger::KeyboardInput, InputKeyboardEvent::new()),
        ]
    }
}
