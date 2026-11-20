//! Module responsible for handling user input.

use keys::Keys;

use crate::{
    ctx::Ctx,
    ecs::system::{SystemBundle, SystemTrigger, Systems},
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

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum InputError {
    #[error("input resource not found")]
    InputNotFound,
    #[error("window resource not found")]
    WindowNotFound,
    #[error("keyboard input event resource not found")]
    KeyboardInputEventNotFound,
    #[error("mouse input event resource not found")]
    MouseInputEventNotFound,
}

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
    pub fn new() -> Input { Input { keys: Keys::new(), mouse: Mouse::new() } }
}

/*
 *   INPUT SYSTEMS
 */

fn input_start(ctx: Ctx) { ctx.resources.add(Input::new()); }

fn input_clear(ctx: Ctx) -> Result<(), ParsecError> {
    let mut input = ctx.resources.get_mut::<Input>().ok_or(InputError::InputNotFound)?;
    input.keys.clear();
    input.mouse.clear();
    Ok(())
}

fn input_clear_all(ctx: Ctx) -> Result<(), ParsecError> {
    let mut input = ctx.resources.get_mut::<Input>().ok_or(InputError::InputNotFound)?;
    input.keys.clear_all();
    input.mouse.clear();
    Ok(())
}

fn input_keyboard_event(ctx: Ctx) -> Result<(), ParsecError> {
    let window = ctx.resources.get::<Window>().ok_or(InputError::WindowNotFound)?;
    if !window.focused() {
        return Ok(());
    }
    let input_event = ctx
        .resources
        .get::<KeyboardInputEvent>()
        .ok_or(InputError::KeyboardInputEventNotFound)?
        .clone();
    let mut input = ctx.resources.get_mut::<Input>().ok_or(InputError::InputNotFound)?;
    input.keys.process_input_event(input_event);
    Ok(())
}

fn input_mouse_movement(ctx: Ctx) -> Result<(), ParsecError> {
    let window = ctx.resources.get::<Window>().ok_or(InputError::WindowNotFound)?;
    if !window.focused() {
        return Ok(());
    }
    let movement_event = ctx
        .resources
        .get::<MouseMovementEvent>()
        .ok_or(InputError::MouseInputEventNotFound)?
        .clone();
    let mut input = ctx.resources.get_mut::<Input>().ok_or(InputError::InputNotFound)?;
    input.mouse.process_movement(movement_event);
    Ok(())
}

fn input_mouse_button(ctx: Ctx) -> Result<(), ParsecError> {
    let window = ctx.resources.get::<Window>().ok_or(InputError::WindowNotFound)?;
    if !window.focused() {
        return Ok(());
    }
    let button_event =
        ctx.resources.get::<MouseButtonEvent>().ok_or(InputError::MouseInputEventNotFound)?.clone();
    let mut input = ctx.resources.get_mut::<Input>().ok_or(InputError::InputNotFound)?;
    input.mouse.process_button_event(button_event);
    Ok(())
}

fn input_mouse_wheel(ctx: Ctx) -> Result<(), ParsecError> {
    let window = ctx.resources.get::<Window>().ok_or(InputError::WindowNotFound)?;
    if !window.focused() {
        return Ok(());
    }
    let wheel_event =
        ctx.resources.get::<MouseWheelEvent>().ok_or(InputError::MouseInputEventNotFound)?.clone();
    let mut input = ctx.resources.get_mut::<Input>().ok_or(InputError::InputNotFound)?;
    input.mouse.process_wheel_event(wheel_event);
    Ok(())
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
