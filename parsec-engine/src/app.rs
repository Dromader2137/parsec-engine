//! Modele responsible managing the lifecycle of an application.

use std::{cell::RefCell, ptr::NonNull};

use crate::{
    ecs::{
        system::{SystemTrigger, Systems},
        world::World,
    },
    input::{
        key::StorageKeyCode,
        keys::KeyboardInputEvent,
        mouse::{MouseButtonEvent, MouseMovementEvent, MouseWheelEvent},
    },
    math::vec::Vec2f,
    resources::Resources,
};

#[allow(unused)]
pub struct App {
    pub systems: Systems,
    world: World,
    resources: Resources,
}

impl App {
    pub fn new() -> App {
        App {
            systems: Systems::new(),
            world: World::new(),
            resources: Resources::new()
        }
    }

    pub fn run(&mut self) {
        self.systems.execute_type(
            SystemTrigger::Start,
            &self.resources,
            &self.world,
        );

        let event_loop =
            winit::event_loop::EventLoop::new().expect("Valid event loop");
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        event_loop
            .run_app(self)
            .expect("Correctly working event loop");
    }
}

pub struct ActiveEventLoopStore {
    event_loop: NonNull<winit::event_loop::ActiveEventLoop>,
}

thread_local! {
pub static ACTIVE_EVENT_LOOP: RefCell<Option<ActiveEventLoopStore>> = RefCell::new(None);
}

impl ActiveEventLoopStore {
    fn new(
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) -> ActiveEventLoopStore {
        ActiveEventLoopStore {
            event_loop: NonNull::from_ref(event_loop),
        }
    }

    pub fn get_event_loop(&self) -> &winit::event_loop::ActiveEventLoop {
        unsafe { self.event_loop.as_ref() }
    }
}

impl winit::application::ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        ACTIVE_EVENT_LOOP.with_borrow_mut(|x| {
            *x = Some(ActiveEventLoopStore::new(event_loop))
        });

        self.systems.execute_type(
            SystemTrigger::LateStart,
            &self.resources,
            &self.world,
        );
    }

    fn device_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        match event {
            winit::event::DeviceEvent::MouseMotion { delta } => {
                self.resources
                    .add(MouseMovementEvent::delta(Vec2f::new(
                        delta.0 as f32,
                        delta.1 as f32,
                    )))
                    .unwrap();

                self.systems.execute_type(
                    SystemTrigger::MouseMovement,
                    &self.resources,
                    &self.world,
                );

                self.resources.remove::<MouseMovementEvent>().unwrap();
            },
            _ => (),
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        state, logical_key, ..
                    },
                ..
            } => {
                let key_code = match logical_key {
                    winit::keyboard::Key::Named(named) => {
                        StorageKeyCode::Noncharacter(named)
                    },
                    winit::keyboard::Key::Character(char) => {
                        StorageKeyCode::Character(char.to_lowercase().into())
                    },
                    _ => return,
                };

                self.resources
                    .add(KeyboardInputEvent::new(key_code.into(), state.into()))
                    .unwrap();

                self.systems.execute_type(
                    SystemTrigger::KeyboardInput,
                    &self.resources,
                    &self.world,
                );

                self.resources.remove::<KeyboardInputEvent>().unwrap();
            },
            winit::event::WindowEvent::CursorLeft { device_id: _ } => {
                self.systems.execute_type(
                    SystemTrigger::WindowCursorLeft,
                    &self.resources,
                    &self.world,
                );
            },
            winit::event::WindowEvent::Resized(_) => {
                self.systems.execute_type(
                    SystemTrigger::WindowResized,
                    &self.resources,
                    &self.world,
                );
            },
            winit::event::WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                self.resources
                    .add(MouseMovementEvent::position(Vec2f::new(
                        position.x as f32,
                        position.y as f32,
                    )))
                    .unwrap();

                self.systems.execute_type(
                    SystemTrigger::MouseMovement,
                    &self.resources,
                    &self.world,
                );

                self.resources.remove::<MouseMovementEvent>().unwrap();
            },
            winit::event::WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => {
                self.resources
                    .add(MouseButtonEvent::new(button, state))
                    .unwrap();

                self.systems.execute_type(
                    SystemTrigger::MouseButton,
                    &self.resources,
                    &self.world,
                );

                self.resources.remove::<MouseButtonEvent>().unwrap();
            },
            winit::event::WindowEvent::MouseWheel {
                device_id: _,
                delta,
                phase: _,
            } => {
                let processed_delta = match delta {
                    winit::event::MouseScrollDelta::PixelDelta(
                        winit::dpi::PhysicalPosition { x, y },
                    ) => Vec2f::new(x as f32, y as f32),
                    winit::event::MouseScrollDelta::LineDelta(x, y) => {
                        Vec2f::new(x, y)
                    },
                };

                self.resources
                    .add(MouseWheelEvent::new(processed_delta))
                    .unwrap();

                self.systems.execute_type(
                    SystemTrigger::MouseWheel,
                    &self.resources,
                    &self.world,
                );

                self.resources.remove::<MouseWheelEvent>().unwrap();
            },
            winit::event::WindowEvent::CloseRequested => {
                ACTIVE_EVENT_LOOP.with_borrow_mut(|x| *x = None);
                self.systems.execute_type(
                    SystemTrigger::End,
                    &self.resources,
                    &self.world,
                );
                event_loop.exit();
            },
            winit::event::WindowEvent::RedrawRequested => {
                self.systems.execute_type(
                    SystemTrigger::Render,
                    &self.resources,
                    &self.world,
                );
            },
            _ => (),
        }
    }

    fn about_to_wait(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
    ) {
        self.systems.execute_type(
            SystemTrigger::EarlyUpdate,
            &self.resources,
            &self.world,
        );
        self.systems.execute_type(
            SystemTrigger::Update,
            &self.resources,
            &self.world,
        );
        self.systems.execute_type(
            SystemTrigger::LateUpdate,
            &self.resources,
            &self.world,
        );
    }
}
