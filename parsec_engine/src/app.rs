//! Modele responsible managing the lifecycle of an application.

use parsec_engine_math::vec::Vec2f;

use crate::{
    assets::library::AssetLibrary, ecs::{
        EcsError, resources::{ResourceError, ResourceMarker, Resources}, system::{SystemTrigger, Systems}, world::World,
    }, error::ParsecError, graphics::ActiveEventLoop, input::{
        key::StorageKeyCode,
        keys::KeyboardInputEvent,
        mouse::{MouseButtonEvent, MouseMovementEvent, MouseWheelEvent},
    },
};

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AppError {
    #[error("event loop error: {0}")]
    EventLoop(#[from] winit::error::EventLoopError),
    #[error("failed to remove event resource: {0}")]
    Resource(#[from] ResourceError),
}

#[allow(unused)]
pub struct App {
    pub systems: Systems,
    pub assets: AssetLibrary,
    world: World,
    resources: Resources,
}

impl App {
    pub fn new() -> App {
        App {
            systems: Systems::new(),
            world: World::new(),
            resources: Resources::new(),
            assets: AssetLibrary::new(),
        }
    }

    pub fn execute(&mut self) -> Result<(), ParsecError> {
        if option_env!("PARSEC_COOK_ASSETS").is_some() {
            self.assets.cook_all()?;
            return Ok(());
        }

        self.execute_system(SystemTrigger::Start);

        let event_loop = winit::event_loop::EventLoop::new().map_err(AppError::from)?;
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        event_loop.run_app(self).map_err(AppError::from)?;
        Ok(())
    }

    pub fn execute_system(&mut self, system_trigger: SystemTrigger) {
        if let Err(err) = self.systems.fire_trigger(
            system_trigger,
            &mut self.world,
            &mut self.resources,
            &mut self.assets,
        ) {
            println!("{}", err);
        }
    }

    fn remove_resource<R: ResourceMarker>(&mut self) {
        if let Err(err) = self.resources.remove::<R>() {
            let perr = ParsecError::Ecs(EcsError::Resource(err));
            println!("{}", perr);
        }
    }
}

impl winit::application::ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.resources.add(ActiveEventLoop::new(event_loop));
        self.execute_system(SystemTrigger::LateStart);
    }

    fn device_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        if let winit::event::DeviceEvent::MouseMotion { delta } = event {
            self.resources
                .add(MouseMovementEvent::delta(Vec2f::new(delta.0 as f32, delta.1 as f32)));

            self.execute_system(SystemTrigger::MouseMovement);

            self.remove_resource::<MouseMovementEvent>();
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
                event: winit::event::KeyEvent { state, logical_key, .. },
                ..
            } => {
                let key_code = match logical_key {
                    winit::keyboard::Key::Named(named) => StorageKeyCode::Noncharacter(named),
                    winit::keyboard::Key::Character(char) => {
                        StorageKeyCode::Character(char.to_lowercase().into())
                    },
                    _ => return,
                };

                self.resources.add(KeyboardInputEvent::new(key_code, state));

                self.execute_system(SystemTrigger::KeyboardInput);

                self.remove_resource::<KeyboardInputEvent>();
            },
            winit::event::WindowEvent::CursorLeft { device_id: _ } => {
                self.execute_system(SystemTrigger::WindowCursorLeft);
            },
            winit::event::WindowEvent::Resized(_) => {
                self.execute_system(SystemTrigger::WindowResized);
            },
            winit::event::WindowEvent::CursorMoved { device_id: _, position } => {
                self.resources.add(MouseMovementEvent::position(Vec2f::new(
                    position.x as f32,
                    position.y as f32,
                )));

                self.execute_system(SystemTrigger::MouseMovement);

                self.remove_resource::<MouseMovementEvent>();
            },
            winit::event::WindowEvent::MouseInput { device_id: _, state, button } => {
                self.resources.add(MouseButtonEvent::new(button, state));

                self.execute_system(SystemTrigger::MouseButton);

                self.remove_resource::<MouseButtonEvent>();
            },
            winit::event::WindowEvent::MouseWheel { device_id: _, delta, phase: _ } => {
                let processed_delta = match delta {
                    winit::event::MouseScrollDelta::PixelDelta(winit::dpi::PhysicalPosition {
                        x,
                        y,
                    }) => Vec2f::new(x as f32, y as f32),
                    winit::event::MouseScrollDelta::LineDelta(x, y) => Vec2f::new(x, y),
                };

                self.resources.add(MouseWheelEvent::new(processed_delta));

                self.execute_system(SystemTrigger::MouseWheel);

                self.remove_resource::<MouseWheelEvent>();
            },
            winit::event::WindowEvent::CloseRequested => {
                self.remove_resource::<ActiveEventLoop>();
                self.execute_system(SystemTrigger::End);
                event_loop.exit();
            },
            winit::event::WindowEvent::RedrawRequested => {
                self.execute_system(SystemTrigger::Render);
            },
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        self.execute_system(SystemTrigger::EarlyUpdate);
        self.execute_system(SystemTrigger::Update);
        self.execute_system(SystemTrigger::LateUpdate);
    }
}
