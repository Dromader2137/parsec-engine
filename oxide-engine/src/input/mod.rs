//! Module responsible for handling user input.

use keys::Keys;

use crate::{
    ecs::system::{System, SystemBundle, SystemTrigger, system},
    graphics::window::Window,
    input::{
        keys::KeyboardInputEvent,
        mouse::{Mouse, MouseButtonEvent, MouseMovementEvent, MouseWheelEvent},
    },
    resources::{Resource, Resources},
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
fn input_keyboard_event(
    mut input: Resource<Input>,
    input_event: Resource<KeyboardInputEvent>,
    window: Resource<Window>,
) {
    if !window.focused() {
        return;
    }
    input.keys.process_input_event((*input_event).clone());
}

#[system]
fn input_mouse_movement(
    mut input: Resource<Input>,
    movement_event: Resource<MouseMovementEvent>,
    window: Resource<Window>,
) {
    if !window.focused() {
        return;
    }
    input.mouse.process_movement(*movement_event);
}

#[system]
fn input_mouse_button(
    mut input: Resource<Input>,
    button_event: Resource<MouseButtonEvent>,
    window: Resource<Window>,
) {
    if !window.focused() {
        return;
    }
    input.mouse.process_button_event(*button_event);
}

#[system]
fn input_mouse_wheel(
    mut input: Resource<Input>,
    wheel_event: Resource<MouseWheelEvent>,
    window: Resource<Window>,
) {
    if !window.focused() {
        return;
    }
    input.mouse.process_wheel_event(*wheel_event);
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
            (SystemTrigger::MouseButton, InputMouseButton::new()),
            (SystemTrigger::MouseWheel, InputMouseWheel::new()),
        ]
    }
}
