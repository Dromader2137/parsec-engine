use keys::Keys;

use crate::{
    ecs::system::{System, SystemBundle, SystemInput, SystemTrigger},
    input::key::{KeyCode, KeyState},
    resources::{Rsc, RscMut},
};

pub mod key;
pub mod keys;

#[derive(Debug)]
pub struct Input {
    pub keys: Keys,
}

impl Input {
    pub fn new() -> Input { Input { keys: Keys::new() } }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputEvent {
    key: KeyCode,
    state: KeyState,
}

impl InputEvent {
    pub fn new(key: KeyCode, state: KeyState) -> InputEvent { InputEvent { key, state } }
}

#[derive(Default)]
pub struct InputBundle {}
impl SystemBundle for InputBundle {
    fn systems(self) -> Vec<System> {
        vec![
            System::new(SystemTrigger::Start, |SystemInput { .. }| {
                Rsc::add(Input::new()).unwrap();
            }),
            System::new(SystemTrigger::Render, |SystemInput { .. }| {
                let mut input = RscMut::<Input>::get().unwrap();
                input.keys.clear();
            }),
            System::new(SystemTrigger::WindowCursorLeft, |SystemInput { .. }| {
                let mut input = RscMut::<Input>::get().unwrap();
                input.keys.clear_all();
            }),
            System::new(SystemTrigger::KeyboardInput, |SystemInput { .. }| {
                let mut input = RscMut::<Input>::get().unwrap();
                let event = Rsc::<InputEvent>::get().unwrap();
                input.keys.process_input_event(*event);
            }),
        ]
    }
}
