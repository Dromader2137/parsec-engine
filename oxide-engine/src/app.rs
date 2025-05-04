use crate::{
    graphics::{graphics_data::GraphicsData, Graphics},
    input::Input, world::World,
};

#[allow(unused)]
pub struct App {
    world: World,
    graphics: Graphics,
    input: Input,
}

impl App {
    pub fn new() -> App {
        App {
            world: World::new(),
            graphics: Graphics::new(),
            input: Input::new(),
        }
    }

    pub fn run(&mut self) {
        let event_loop = winit::event_loop::EventLoop::new().expect("Valid event loop");
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        event_loop
            .run_app(self)
            .expect("Correctly working event loop");
        if let Some(graphics) = self.graphics.data.as_mut() {
            if let Err(err) = graphics.renderer.cleanup(&graphics.vulkan_context) {
                println!("Error: {:?}", err);
            }
            if let Err(err) = graphics.vulkan_context.cleanup() {
                println!("Error: {:?}", err);
            }
        }
    }
}

impl winit::application::ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.graphics.data = match GraphicsData::new(event_loop) {
            Ok(val) => Some(val),
            Err(err) => {
                println!("Error: {:?}", err);
                None
            }
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
            } => if let winit::keyboard::PhysicalKey::Code(key_code) = physical_key { match state {
                winit::event::ElementState::Pressed => {
                    self.input.keys.press(key_code.into());
                }
                winit::event::ElementState::Released => {
                    self.input.keys.lift(key_code.into());
                }
            } },
            winit::event::WindowEvent::CursorLeft { device_id } => {
                let _ = device_id;
                self.input.keys.clear_all();
            }
            winit::event::WindowEvent::Resized(_) => {
                if let Some(graphics) = self.graphics.data.as_mut() {
                    if let Err(err) = graphics.renderer.handle_resize() {
                        println!("Error: {:?}", err);
                    }
                }
            }
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            winit::event::WindowEvent::RedrawRequested => {
                if let Some(graphics) = self.graphics.data.as_mut() {
                    if let Err(err) = graphics
                        .renderer
                        .render(&graphics.vulkan_context, &graphics.window)
                    {
                        println!("Error: {:?}", err);
                    }
                }
                self.input.keys.clear();
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(graphics) = self.graphics.data.as_ref() {
            graphics.window.request_redraw();
        }
    }
}
