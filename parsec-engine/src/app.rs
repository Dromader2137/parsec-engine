//! Modele responsible managing the lifecycle of an application.

use parsec_engine_ecs::{
    resources::Resources,
    system::{SystemTrigger, Systems, requests::Requests},
    world::World,
};
use parsec_engine_graphics::ActiveEventLoop;
use parsec_engine_input::{
    key::StorageKeyCode,
    keys::KeyboardInputEvent,
    mouse::{MouseButtonEvent, MouseMovementEvent, MouseWheelEvent},
};
use parsec_engine_math::vec::Vec2f;

#[allow(unused)]
pub struct App {
    pub systems: Systems,
    world: World,
    resources: Resources,
}

impl Default for App {
    fn default() -> Self { Self::new() }
}

impl App {
    pub fn new() -> App {
        App {
            systems: Systems::new(),
            world: World::new(),
            resources: Resources::new(),
        }
    }

    pub fn run(&mut self) {
        self.resources.add(Requests::new(self.world.current_id));

        self.execute_system(SystemTrigger::Start);

        let event_loop =
            winit::event_loop::EventLoop::new().expect("Valid event loop");
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        event_loop.run_app(self).unwrap();
    }

    pub fn execute_system(&mut self, system_trigger: SystemTrigger) {
        if let Err(err) = self.systems.execute_type(
            system_trigger,
            &mut self.resources,
            &mut self.world,
        ) {
            panic!(
                "System triggered with {:?} returned: {}",
                system_trigger, err
            );
        }
    }
}

// pub struct ActiveEventLoopStore {
//     event_loop: NonNull<winit::event_loop::ActiveEventLoop>,
// }
//
// thread_local! {
// pub static ACTIVE_EVENT_LOOP: RefCell<Option<ActiveEventLoopStore>> = const { RefCell::new(None) };
// }
//
// impl ActiveEventLoopStore {
//     fn new(
//         event_loop: &winit::event_loop::ActiveEventLoop,
//     ) -> ActiveEventLoopStore {
//         ActiveEventLoopStore {
//             event_loop: NonNull::from_ref(event_loop),
//         }
//     }
//
//     pub fn get_event_loop(&self) -> &winit::event_loop::ActiveEventLoop {
//         unsafe { self.event_loop.as_ref() }
//     }
// }

impl winit::application::ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.resources.add(ActiveEventLoop::new(event_loop));
        // ACTIVE_EVENT_LOOP.with_borrow_mut(|x| {
        //     *x = Some(ActiveEventLoopStore::new(event_loop))
        // });
        self.execute_system(SystemTrigger::LateStart);
    }

    fn device_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        if let winit::event::DeviceEvent::MouseMotion { delta } = event {
            self.resources.add(MouseMovementEvent::delta(Vec2f::new(
                delta.0 as f32,
                delta.1 as f32,
            )));

            self.execute_system(SystemTrigger::MouseMovement);

            self.resources.remove::<MouseMovementEvent>().unwrap();
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

                self.resources.add(KeyboardInputEvent::new(key_code, state));

                self.execute_system(SystemTrigger::KeyboardInput);

                self.resources.remove::<KeyboardInputEvent>().unwrap();
            },
            winit::event::WindowEvent::CursorLeft { device_id: _ } => {
                self.execute_system(SystemTrigger::WindowCursorLeft);
            },
            winit::event::WindowEvent::Resized(_) => {
                self.execute_system(SystemTrigger::WindowResized);
            },
            winit::event::WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                self.resources.add(MouseMovementEvent::position(Vec2f::new(
                    position.x as f32,
                    position.y as f32,
                )));

                self.execute_system(SystemTrigger::MouseMovement);

                self.resources.remove::<MouseMovementEvent>().unwrap();
            },
            winit::event::WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => {
                self.resources.add(MouseButtonEvent::new(button, state));

                self.execute_system(SystemTrigger::MouseButton);

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

                self.resources.add(MouseWheelEvent::new(processed_delta));

                self.execute_system(SystemTrigger::MouseWheel);

                self.resources.remove::<MouseWheelEvent>().unwrap();
            },
            winit::event::WindowEvent::CloseRequested => {
                self.resources.remove::<ActiveEventLoop>().unwrap();
                self.execute_system(SystemTrigger::End);
                event_loop.exit();
            },
            winit::event::WindowEvent::RedrawRequested => {
                self.execute_system(SystemTrigger::Render);
            },
            _ => (),
        }
    }

    fn about_to_wait(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
    ) {
        self.execute_system(SystemTrigger::EarlyUpdate);
        self.execute_system(SystemTrigger::Update);
        self.execute_system(SystemTrigger::LateUpdate);
    }
}
