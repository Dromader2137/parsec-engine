//! Module responsible for handling user input.

use keys::Keys;

use crate::{
    ctx::Ctx,
    ecs::system::{SystemBundle, SystemTrigger, Systems},
    error::{OptionNoneErr, ParsecError},
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

fn input_start(ctx: Ctx) { ctx.resources.add(Input::new()); }

fn input_clear(ctx: Ctx) -> Result<(), ParsecError> {
    let mut input = ctx.resources.get_mut::<Input>().none_err()?;
    input.keys.clear();
    input.mouse.clear();
    Ok(())
}

fn input_clear_all(ctx: Ctx) -> Result<(), ParsecError> {
    let mut input = ctx.resources.get_mut::<Input>().none_err()?;
    input.keys.clear_all();
    input.mouse.clear();
    Ok(())
}

fn input_keyboard_event(ctx: Ctx) -> Result<(), ParsecError> {
    let window = ctx.resources.get::<Window>().none_err()?;
    if !window.focused() {
        return Ok(());
    }
    let input_event = ctx
        .resources
        .get::<KeyboardInputEvent>()
        .none_err()?
        .clone();
    ctx.resources
        .get_mut::<Input>()
        .none_err()?
        .keys
        .process_input_event(input_event);
    Ok(())
}

fn input_mouse_movement(ctx: Ctx) -> Result<(), ParsecError> {
    let window = ctx.resources.get::<Window>().none_err()?;
    if !window.focused() {
        return Ok(());
    }
    let movement_event = ctx
        .resources
        .get::<MouseMovementEvent>()
        .none_err()?
        .clone();
    ctx.resources
        .get_mut::<Input>()
        .none_err()?
        .mouse
        .process_movement(movement_event);
    Ok(())
}

fn input_mouse_button(ctx: Ctx) -> Result<(), ParsecError> {
    let window = ctx.resources.get::<Window>().none_err()?;
    if !window.focused() {
        return Ok(());
    }
    let button_event =
        ctx.resources.get::<MouseButtonEvent>().none_err()?.clone();
    ctx.resources
        .get_mut::<Input>()
        .none_err()?
        .mouse
        .process_button_event(button_event);
    Ok(())
}

fn input_mouse_wheel(ctx: Ctx) -> Result<(), ParsecError> {
    let window = ctx.resources.get::<Window>().none_err()?;
    if !window.focused() {
        return Ok(());
    }
    let wheel_event =
        ctx.resources.get::<MouseWheelEvent>().none_err()?.clone();
    ctx.resources
        .get_mut::<Input>()
        .none_err()?
        .mouse
        .process_wheel_event(wheel_event);
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
