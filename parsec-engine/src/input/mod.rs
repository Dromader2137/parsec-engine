//! Module responsible for handling user input.

use keys::Keys;

use crate::{
    ecs::{
        system::{SystemBundle, SystemTrigger, Systems},
        world::World,
    },
    error::ParsecError,
    graphics::window::Window,
    input::{
        keys::KeyboardInputEvent,
        mouse::{Mouse, MouseButtonEvent, MouseMovementEvent, MouseWheelEvent},
    },
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

impl Default for Input {
    fn default() -> Self { Self::new() }
}

impl Input {
    pub fn new() -> Input {
        Input {
            keys: Keys::new(),
            mouse: Mouse::new(),
        }
    }
}

fn input_start(world: &mut World) { world.resources.add(Input::new()); }

fn input_clear(world: &World) -> Result<(), ParsecError> {
    let mut input = world.resources.get::<Input>();
    input.keys.clear();
    input.mouse.clear();
    Ok(())
}

fn input_clear_all(world: &World) {
    let mut input = world.resources.get::<Input>();
    input.keys.clear_all();
    input.mouse.clear();
}

fn input_keyboard_event(world: &World) {
    let window = world.resources.get::<Window>();
    if !window.focused() {
        return;
    }
    let input_event = world.resources.get::<KeyboardInputEvent>();
    world
        .resources.get::<Input>()
        .keys
        .process_input_event((*input_event).clone());
}

fn input_mouse_movement(world: &World) {
    let window = world.resources.get::<Window>();
    if !window.focused() {
        return;
    }
    let movement_event = world.resources.get::<MouseMovementEvent>();
    world
        .resources.get::<Input>()
        .mouse
        .process_movement(*movement_event);
}

fn input_mouse_button(world: &World) {
    let window = world.resources.get::<Window>();
    if !window.focused() {
        return;
    }
    let button_event = world.resources.get::<MouseButtonEvent>();
    world
        .resources.get::<Input>()
        .mouse
        .process_button_event(*button_event);
}

fn input_mouse_wheel(world: &World) {
    let window = world.resources.get::<Window>();
    if !window.focused() {
        return;
    }
    let wheel_event = world.resources.get::<MouseWheelEvent>();
    world
        .resources.get::<Input>()
        .mouse
        .process_wheel_event(*wheel_event);
}

pub struct InputBundle;
impl SystemBundle for InputBundle {
    fn insert(self, systems: &mut Systems) {
        systems.add(SystemTrigger::Start, input_start);
        systems.add(SystemTrigger::LateUpdate, input_clear);
        systems.add(SystemTrigger::WindowCursorLeft, input_clear_all);
        systems.add(SystemTrigger::KeyboardInput, input_keyboard_event);
        systems.add(SystemTrigger::MouseMovement, input_mouse_movement);
        systems.add(SystemTrigger::MouseButton, input_mouse_button);
        systems.add(SystemTrigger::MouseWheel, input_mouse_wheel);
    }
}
