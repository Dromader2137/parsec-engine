use std::{cell::RefCell, ptr::NonNull, sync::Mutex};

use crate::{
    ecs::{
        system::{SystemTrigger, Systems},
        world::World,
    },
    input::InputEvent,
    resources::Resources,
};

#[allow(unused)]
pub struct App {
    pub systems: Systems,
}

impl App {
    pub fn new() -> App {
        App {
            systems: Systems::new(),
        }
    }

    pub fn run(&mut self) {
        self.systems.execute_type(SystemTrigger::Start);

        Resources::add(Mutex::new(World::new())).unwrap();

        let event_loop = winit::event_loop::EventLoop::new().expect("Valid event loop");
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
    fn new(event_loop: &winit::event_loop::ActiveEventLoop) -> ActiveEventLoopStore {
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
        ACTIVE_EVENT_LOOP.with_borrow_mut(|x| *x = Some(ActiveEventLoopStore::new(event_loop)));

        self.systems.execute_type(SystemTrigger::LateStart);
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
                        physical_key,
                        state,
                        ..
                    },
                ..
            } => {
                if let winit::keyboard::PhysicalKey::Code(key_code) = physical_key {
                    Resources::add(InputEvent::new(key_code.into(), state.into())).unwrap();
                }

                self.systems.execute_type(SystemTrigger::KeyboardInput);

                if let winit::keyboard::PhysicalKey::Code(_) = physical_key {
                    Resources::remove::<InputEvent>().unwrap();
                }
            },
            winit::event::WindowEvent::CursorLeft { device_id: _ } => {
                self.systems.execute_type(SystemTrigger::WindowCursorLeft);
            },
            winit::event::WindowEvent::Resized(_) => {
                self.systems.execute_type(SystemTrigger::WindowResized);
            },
            winit::event::WindowEvent::CloseRequested => {
                ACTIVE_EVENT_LOOP.with_borrow_mut(|x| *x = None);
                self.systems.execute_type(SystemTrigger::End);
                event_loop.exit();
            },
            winit::event::WindowEvent::RedrawRequested => {
                self.systems.execute_type(SystemTrigger::Render);
            },
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        self.systems.execute_type(SystemTrigger::Update);
    }
}
