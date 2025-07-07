use crate::{
    assets::library::AssetLibrary, error::error, graphics::Graphics, input::Input, ecs::world::World,
};

#[allow(unused)]
pub struct App {
    name: String,
    world: World,
    systems: ,
    input: Input,
    assets: AssetLibrary,
    graphics: Graphics,
}

impl App {
    pub fn new(name: String) -> App {
        App {
            name,
            world: World::new(),
            input: Input::new(),
            assets: AssetLibrary::new(),
            graphics: Graphics::new(),
        }
    }

    pub fn run(&mut self) {
        let event_loop = winit::event_loop::EventLoop::new().expect("Valid event loop");
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        event_loop
            .run_app(self)
            .expect("Correctly working event loop");
    }
}

impl winit::application::ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Err(err) = self.graphics.init(event_loop, &self.name) {
            error(err.into());
        };
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
                    match state {
                        winit::event::ElementState::Pressed => {
                            self.input.keys.press(key_code.into())
                        }
                        winit::event::ElementState::Released => {
                            self.input.keys.lift(key_code.into())
                        }
                    }
                }
            }
            winit::event::WindowEvent::CursorLeft { device_id } => {
                let _ = device_id;
                self.input.keys.clear_all();
            }
            winit::event::WindowEvent::Resized(_) => {
                if let Err(err) = self.graphics.resize() {
                    error(err.into());
                }
            }
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            winit::event::WindowEvent::RedrawRequested => {
                if let Err(err) = self.graphics.render() {
                    error(err.into());
                }
                self.input.keys.clear();
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Err(err) = self.graphics.request_redraw() {
            error(err.into());
        }
    }
}
