use std::{cell::RefCell, ptr::NonNull};

use crate::{
    assets::library::AssetLibrary,
    ecs::{
        system::{SystemInput, SystemTrigger, Systems},
        world::World,
    },
    input::InputEvent,
    resources::Rsc,
};

#[allow(unused)]
pub struct App {
    world: World,
    pub systems: Systems,
    assets: AssetLibrary,
}

impl App {
    pub fn new() -> App {
        App {
            world: World::new(),
            systems: Systems::new(),
            assets: AssetLibrary::new(),
        }
    }

    pub fn run(&mut self) {
        self.systems
            .execute_type(SystemTrigger::Start, SystemInput {
                world: &mut self.world,
                assets: &mut self.assets,
            });

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

        self.systems
            .execute_type(SystemTrigger::LateStart, SystemInput {
                world: &mut self.world,
                assets: &mut self.assets,
            });

        ACTIVE_EVENT_LOOP.with_borrow_mut(|x| *x = None);
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
                    Rsc::add(InputEvent::new(key_code.into(), state.into())).unwrap();
                }

                self.systems
                    .execute_type(SystemTrigger::KeyboardInput, SystemInput {
                        world: &mut self.world,
                        assets: &mut self.assets,
                    });

                if let winit::keyboard::PhysicalKey::Code(_) = physical_key {
                    Rsc::<InputEvent>::remove().unwrap();
                }
            },
            winit::event::WindowEvent::CursorLeft { device_id: _ } => {
                self.systems
                    .execute_type(SystemTrigger::WindowCursorLeft, SystemInput {
                        world: &mut self.world,
                        assets: &mut self.assets,
                    });
            },
            winit::event::WindowEvent::Resized(_) => {
                self.systems
                    .execute_type(SystemTrigger::WindowResized, SystemInput {
                        world: &mut self.world,
                        assets: &mut self.assets,
                    });
            },
            winit::event::WindowEvent::CloseRequested => {
                self.systems.execute_type(SystemTrigger::End, SystemInput {
                    world: &mut self.world,
                    assets: &mut self.assets,
                });
                event_loop.exit();
            },
            winit::event::WindowEvent::RedrawRequested => {
                self.systems
                    .execute_type(SystemTrigger::Render, SystemInput {
                        world: &mut self.world,
                        assets: &mut self.assets,
                    });
            },
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        self.systems
            .execute_type(SystemTrigger::Update, SystemInput {
                world: &mut self.world,
                assets: &mut self.assets,
            });
    }
}
